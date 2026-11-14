use pastimer::{
    audit,
    engine::Engine,
    error::{Error, Result},
    model::{Action, Command, GameId, Player},
    store::Store,
    thousand::{self, Settings},
};
use serde_json::Value;
use uuid::Uuid;

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn human(i: u8) -> Player {
    Player {
        id: format!("human-{i}"),
        name: format!("Human {i}"),
    }
}
fn text(value: &Value) -> Result<&str> {
    value.as_str().ok_or_else(Error::internal)
}
async fn command(e: &Engine, code: &str, actor: &str, action: Action) -> Result<Value> {
    let view = e.read(code, actor).await?;
    e.command(
        code,
        actor,
        Command {
            version: view["version"].as_u64().ok_or_else(Error::internal)?,
            request_id: Uuid::new_v4().to_string(),
            action,
        },
    )
    .await
}
async fn setup(path: &str, seats: u8, humans: u8) -> Result<(Engine, String)> {
    let e = Engine::new(Store::open(path).await?);
    let view = e
        .create_with_computers(
            human(0),
            GameId::Thousand,
            Some(Settings {
                player_count: seats,
                barrel_threshold: 900,
            }),
            seats - humans,
        )
        .await?;
    let code = text(&view["code"])?.to_owned();
    assert_eq!(
        view["computers"].as_array().map(Vec::len),
        Some(usize::from(seats - humans))
    );
    for i in 1..humans {
        e.join(&code, human(i)).await?;
    }
    command(&e, &code, "human-0", Action::Start).await?;
    Ok((e, code))
}
fn choose_human(view: &Value) -> Result<Option<Action>> {
    let c = &view["table"]["controls"];
    let action = if c["shuffle"] == true {
        thousand::Action::Shuffle
    } else if c["pass"] == true {
        thousand::Action::Pass
    } else if c["takeStock"] == true {
        thousand::Action::TakeStock
    } else if let Some(recipients) = c["recipients"].as_array().filter(|r| !r.is_empty()) {
        thousand::Action::Distribute {
            transfers: recipients
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    Ok(thousand::Transfer {
                        player: text(&p["id"])?.into(),
                        card: text(&view["table"]["hand"][i])?.parse()?,
                        public: false,
                    })
                })
                .collect::<Result<_>>()?,
        }
    } else if c["order"] == true {
        thousand::Action::Order {
            amount: view["table"]["order"]
                .as_u64()
                .ok_or_else(Error::internal)? as u16,
        }
    } else if let Some(card) = c["legalCards"].as_array().and_then(|c| c.first()) {
        thousand::Action::Play {
            card: text(card)?.parse()?,
            announce: false,
        }
    } else {
        return Ok(None);
    };
    Ok(Some(Action::Thousand { action }))
}
async fn step(e: &Engine, code: &str, humans: u8) -> Result<Value> {
    let room = e.read(code, "human-0").await?;
    if room["table"]["phase"] == "closing" {
        e.advance_at(
            code,
            "human-0",
            room["table"]["closeAt"]
                .as_i64()
                .ok_or_else(Error::internal)?,
        )
        .await?;
        return e.read(code, "human-0").await;
    }
    if let Some(due) = room["computerDueAt"].as_i64() {
        e.advance_at(code, "human-0", due).await?;
        return e.read(code, "human-0").await;
    }
    if room["table"]["phase"] == "awaitShuffle"
        && room["computers"]
            .as_array()
            .is_some_and(|ids| ids.contains(&room["table"]["dealer"]))
    {
        return command(e, code, "human-0", Action::ComputerDeal).await;
    }
    for i in 0..humans {
        let actor = human(i).id;
        let own = e.read(code, &actor).await?;
        assert!(own.get("state").is_none());
        assert!(own["table"].get("hands").is_none());
        if let Some(action) = choose_human(&own)? {
            return command(e, code, &actor, action).await;
        }
    }
    Err(Error::bad(
        "Computer table stalled with no legal human action",
    ))
}

#[tokio::test]
async fn one_or_two_humans_complete_deals_with_legal_auditable_computers() -> TestResult {
    for seats in [3, 4] {
        for humans in [1, 2] {
            let (e, code) = setup(":memory:", seats, humans).await?;
            let mut room = e.read(&code, "human-0").await?;
            for _ in 0..500 {
                if room["table"]["auditDeals"].as_u64().unwrap_or(0) >= 4 {
                    break;
                }
                room = step(&e, &code, humans).await?;
                let exported = e.audit(&code, "human-0").await?;
                assert_eq!(
                    audit::verify(&exported)?["deals"],
                    room["table"]["auditDeals"]
                );
                if let Some(deck) = exported["decks"].as_array().and_then(|d| d.last())
                    && room["table"]["phase"] != "awaitShuffle"
                {
                    assert!(
                        deck.get("proof").is_none(),
                        "An unfinished computer deal must stay private"
                    );
                }
            }
            assert_eq!(room["table"]["auditDeals"], 4);
            assert_eq!(
                room["table"]["phase"], "awaitShuffle",
                "Keep settled results until a human asks for the next deal"
            );
            let export = e.audit(&code, "human-0").await?;
            assert_eq!(audit::verify(&export)?["deals"], 4);
            let computer_actions: Vec<_> = export["commands"]
                .as_array()
                .ok_or("commands")?
                .iter()
                .filter(|c| {
                    c["actor"]
                        .as_str()
                        .is_some_and(|id| id.starts_with("computer:"))
                })
                .collect();
            assert!(
                computer_actions
                    .iter()
                    .any(|c| c["action"]["action"]["type"] == "play")
            );
            assert!(computer_actions.iter().any(|c| matches!(
                c["action"]["action"]["type"].as_str(),
                Some("bid" | "pass" | "here" | "distribute")
            )));
            assert!(
                computer_actions
                    .iter()
                    .any(|c| c["action"]["action"]["type"] == "shuffle")
            );
            assert!(
                export["deals"]
                    .as_array()
                    .ok_or("deals")?
                    .iter()
                    .all(|d| d["tricks"].as_array().map(Vec::len) == Some(8))
            );
            let mut forged = export.clone();
            let request = forged["events"]
                .as_array_mut()
                .ok_or("events")?
                .iter_mut()
                .find(|e| e["type"] == "computer.deal.requested")
                .ok_or("computer request")?;
            request["actor"] = request["data"]["player"].clone();
            let mut previous = "0".repeat(64);
            for event in forged["events"].as_array_mut().ok_or("events")? {
                event["previousHash"] = Value::String(previous);
                event.as_object_mut().ok_or("event")?.remove("hash");
                previous = pastimer::crypto::hash(event)?;
                event["hash"] = Value::String(previous.clone());
            }
            assert!(
                audit::verify(&forged).is_err(),
                "A computer cannot impersonate the human request to begin a deal"
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn computer_configuration_capacity_and_human_authority_are_enforced() -> TestResult {
    let e = Engine::new(Store::open(":memory:").await?);
    assert!(
        e.create_with_computers(human(0), GameId::Thousand, None, 3)
            .await
            .is_err()
    );
    assert!(
        e.create_with_computers(human(0), GameId::HighCard, None, 1)
            .await
            .is_err()
    );
    let view = e
        .create_with_computers(human(0), GameId::Thousand, None, 2)
        .await?;
    let code = text(&view["code"])?;
    assert!(e.join(code, human(1)).await.is_err());
    assert!(
        command(&e, code, "human-0", Action::ComputerDeal)
            .await
            .is_err()
    );
    let before = e.read(code, "human-0").await?;
    assert!(e.read(code, "outsider").await.is_err());
    assert_eq!(before, e.read(code, "human-0").await?);
    // Old human-only snapshots decode with no computer seats.
    let mut conn = e.store.pool.acquire().await?;
    let room = Engine::load(&mut conn, code).await?;
    let mut stored = serde_json::to_value(room)?;
    stored
        .as_object_mut()
        .ok_or("snapshot")?
        .remove("computers");
    assert!(
        serde_json::from_value::<pastimer::model::Room>(stored)?
            .computers
            .is_empty()
    );
    Ok(())
}

#[tokio::test]
async fn computers_recover_after_restart_and_concurrent_deadline_resolution() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("computers.sqlite");
    let path = path.to_str().ok_or("path")?;
    let (e, code) = setup(path, 4, 1).await?;
    let mut room = e.read(&code, "human-0").await?;
    for _ in 0..70 {
        if room["table"]["phase"] == "closing" {
            break;
        }
        room = step(&e, &code, 1).await?;
    }
    assert_eq!(room["table"]["phase"], "closing");
    let deadline = room["table"]["closeAt"].as_i64().ok_or("deadline")?;
    let computers = room["computers"].clone();
    e.store.pool.close().await;
    let e = Engine::new(Store::open(path).await?);
    let (a, b) = tokio::join!(
        e.advance_at(&code, "human-0", deadline),
        e.advance_at(&code, "human-0", deadline)
    );
    a?;
    b?;
    let mut room = e.read(&code, "human-0").await?;
    assert_eq!(room["computers"], computers);
    let export = e.audit(&code, "human-0").await?;
    assert_eq!(
        export["events"]
            .as_array()
            .ok_or("events")?
            .iter()
            .filter(|e| e["type"] == "trick.closed")
            .count(),
        1
    );
    for _ in 0..100 {
        if room["table"]["auditDeals"] == 1 {
            break;
        }
        room = step(&e, &code, 1).await?;
    }
    assert_eq!(
        audit::verify(&e.audit(&code, "human-0").await?)?["deals"],
        1
    );
    if !computers
        .as_array()
        .is_some_and(|ids| ids.contains(&room["table"]["dealer"]))
    {
        for _ in 0..100 {
            if room["table"]["auditDeals"] == 2 {
                break;
            }
            room = step(&e, &code, 1).await?;
        }
    }
    // A stored human receipt remains idempotent even when the response also includes computer moves.
    assert!(
        computers
            .as_array()
            .is_some_and(|ids| ids.contains(&room["table"]["dealer"]))
    );
    let version = room["version"].as_u64().ok_or("version")?;
    let cmd = Command {
        version,
        request_id: Uuid::new_v4().to_string(),
        action: Action::ComputerDeal,
    };
    let first = e.command(&code, "human-0", cmd.clone()).await?;
    let again = e.command(&code, "human-0", cmd).await?;
    assert_eq!(first["version"], again["version"]);
    assert_eq!(first["table"]["hand"], again["table"]["hand"]);
    assert_eq!(computers.as_array().map(Vec::len), Some(3));
    Ok(())
}

#[tokio::test]
async fn computer_think_deadline_survives_restart_and_never_bursts() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("paced.sqlite");
    let path = file.to_str().ok_or("path")?;
    let (e, code) = setup(path, 3, 1).await?;
    let mut room = e.read(&code, "human-0").await?;
    for _ in 0..30 {
        if room["computerDueAt"].is_number() {
            break;
        }
        room = step(&e, &code, 1).await?;
    }
    let due = room["computerDueAt"].as_i64().ok_or("deadline")?;
    let version = room["version"].as_u64().ok_or("version")?;
    e.advance_at(&code, "human-0", due - 1).await?;
    assert_eq!(e.read(&code, "human-0").await?["version"], version);
    e.store.pool.close().await;
    let e = Engine::new(Store::open(path).await?);
    assert_eq!(e.read(&code, "human-0").await?["computerDueAt"], due);
    e.advance_at(&code, "human-0", due).await?;
    let after = e.read(&code, "human-0").await?;
    assert_eq!(after["version"], version + 1);
    if let Some(next) = after["computerDueAt"].as_i64() {
        assert!(next >= due + 3000);
    }
    e.advance_at(&code, "human-0", due).await?;
    assert_eq!(e.read(&code, "human-0").await?["version"], version + 1);
    Ok(())
}

#[tokio::test]
async fn active_table_recovers_without_browser_storage_and_backfills_older_rooms() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("resume.sqlite");
    let path = file.to_str().ok_or("path")?;
    let e = Engine::new(Store::open(path).await?);
    let room = e
        .create_configured(human(0), GameId::Thousand, None)
        .await?;
    let code = text(&room["code"])?.to_owned();
    assert_eq!(e.store.recent("human-0").await?, Some(code.clone()));
    assert_eq!(e.store.recent("outsider").await?, None);
    sqlx::query("DELETE FROM player_rooms")
        .execute(&e.store.pool)
        .await?;
    assert_eq!(e.store.recent("human-0").await?, Some(code.clone()));
    e.join(&code, human(1)).await?;
    e.store.pool.close().await;
    let e = Engine::new(Store::open(path).await?);
    let new_cookie = e.store.session(&human(1)).await?;
    let identity = e.store.authenticate(&new_cookie).await?.ok_or("session")?;
    assert_eq!(e.store.recent(&identity.id).await?, Some(code));
    Ok(())
}
