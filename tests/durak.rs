use pastimer::{
    audit, crypto,
    durak::{self, Settings, View},
    engine::Engine,
    model::{Action, Command, GameId, GameState, Lifecycle, Player, Room},
    store::Store,
};
use serde_json::{Value, json};
use uuid::Uuid;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn player(i: usize) -> Player {
    Player {
        id: format!("durak:{i}"),
        name: format!("Player {i}"),
    }
}
async fn send(
    e: &Engine,
    code: &str,
    actor: &str,
    action: durak::Action,
) -> Result<Value, Box<dyn std::error::Error>> {
    let v = e.read(code, actor).await?;
    Ok(e.command(
        code,
        actor,
        Command {
            version: v["version"].as_u64().ok_or("version")?,
            request_id: Uuid::new_v4().to_string(),
            action: Action::Durak { action },
        },
    )
    .await?)
}
async fn start(
    e: &Engine,
    game: GameId,
    n: u8,
    computers: u8,
) -> Result<String, Box<dyn std::error::Error>> {
    let v = e
        .create_durak(player(0), game, Settings { player_count: n }, computers)
        .await?;
    let code = v["code"].as_str().ok_or("code")?.to_owned();
    for i in 1..usize::from(n - computers) {
        e.join(&code, player(i)).await?;
    }
    let v = e.read(&code, &player(0).id).await?;
    e.command(
        &code,
        &player(0).id,
        Command {
            version: v["version"].as_u64().ok_or("version")?,
            request_id: Uuid::new_v4().to_string(),
            action: Action::Start,
        },
    )
    .await?;
    Ok(code)
}
async fn finish(e: &Engine, code: &str, n: usize) -> TestResult {
    for _ in 0..3000 {
        let mut choice = None;
        for i in 0..n {
            let v = e.read(code, &player(i).id).await?;
            if v["phase"] == "finished" {
                return Ok(());
            }
            let view: View = serde_json::from_value(v["table"].clone())?;
            if let Some(action) = durak::choose(view) {
                choice = Some((player(i).id, action));
                break;
            }
        }
        let (actor, action) = choice.ok_or("no move")?;
        send(e, code, &actor, action).await?;
    }
    Err("game did not finish".into())
}
#[tokio::test]
async fn variants_complete_replay_rematch_and_seal_the_next_game() -> TestResult {
    for game in [GameId::DurakPodkidnoy, GameId::DurakPerevodnoy] {
        let e = Engine::new(Store::open(":memory:").await?);
        let code = start(&e, game, 3, 0).await?;
        let active = e.audit(&code, &player(0).id).await?;
        assert!(active["decks"][0].get("proof").is_none());
        assert_eq!(active["commands"], json!([]));
        assert_eq!(audit::verify(&active)?["deals"], 0);
        finish(&e, &code, 3).await?;
        let a = e.audit(&code, &player(0).id).await?;
        assert_eq!(audit::verify(&a)?["deals"], 1);
        let mut tampered = a.clone();
        tampered["deals"][0]["hands"] = json!({});
        assert!(audit::verify(&tampered).is_err());
        // Rehash a forged record and the entire event chain: rule replay must still reject it.
        let mut forged = a.clone();
        forged["deals"][0]["hands"] = json!({});
        let digest = crypto::hash(&forged["deals"][0])?;
        let mut previous = "0".repeat(64);
        for event in forged["events"].as_array_mut().ok_or("events")? {
            if event["type"] == "durak.settled" {
                event["data"]["recordHash"] = json!(digest);
            }
            event["previousHash"] = json!(previous);
            let mut body = event.as_object().ok_or("event")?.clone();
            body.remove("hash");
            previous = crypto::hash(&body)?;
            event["hash"] = json!(previous);
        }
        assert!(audit::verify(&forged).is_err());
        let mut tampered = a.clone();
        tampered["commands"][0]["actor"] = json!("outsider");
        assert!(audit::verify(&tampered).is_err());
        send(&e, &code, &player(0).id, durak::Action::Rematch).await?;
        let a = e.audit(&code, &player(0).id).await?;
        assert_eq!(audit::verify(&a)?["deals"], 1);
        assert!(a["decks"][1].get("proof").is_none());
        finish(&e, &code, 3).await?;
        assert_eq!(
            audit::verify(&e.audit(&code, &player(0).id).await?)?["deals"],
            2
        );
    }
    Ok(())
}
#[tokio::test]
async fn durable_retry_concurrency_recovery_and_private_reads() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("durak.sqlite");
    let path = path.to_str().ok_or("path")?;
    let e = Engine::new(Store::open(path).await?);
    let code = start(&e, GameId::DurakPerevodnoy, 2, 0).await?;
    let v = e.read(&code, &player(0).id).await?;
    let actor = v["table"]["attacker"].as_str().ok_or("attacker")?;
    let own = e.read(&code, actor).await?;
    let view: View = serde_json::from_value(own["table"].clone())?;
    let action = durak::choose(view).ok_or("move")?;
    let cmd = Command {
        version: own["version"].as_u64().ok_or("version")?,
        request_id: "durable-durak-move".into(),
        action: Action::Durak { action },
    };
    let (a, b) = tokio::join!(
        e.command(&code, actor, cmd.clone()),
        e.command(&code, actor, cmd.clone())
    );
    assert_eq!(a?["version"], b?["version"]);
    let before = e.record(&code, actor).await?;
    let before_audit = e.audit(&code, actor).await?;
    assert!(e.read(&code, "outsider").await.is_err());
    assert!(e.record(&code, "outsider").await.is_err());
    assert!(e.audit(&code, "outsider").await.is_err());
    let recovered = Engine::new(Store::open(path).await?);
    assert_eq!(
        recovered.command(&code, actor, cmd).await?["version"],
        own["version"].as_u64().ok_or("version")? + 1
    );
    assert_eq!(before, recovered.record(&code, actor).await?);
    assert_eq!(before_audit, recovered.audit(&code, actor).await?);
    assert_eq!(
        recovered.store.recent(actor).await?.as_deref(),
        Some(code.as_str())
    );
    let mut conn = recovered.store.pool.acquire().await?;
    let room = Engine::load(&mut conn, &code).await?;
    drop(conn);
    let Some(GameState::Durak(state)) = room.lifecycle.state() else {
        return Err("state".into());
    };
    let projected = serde_json::to_value(state.view(actor, &room.players))?;
    assert!(projected.get("hands").is_none());
    assert!(projected.get("movements").is_none());
    assert!(projected.get("stock").is_none());
    // Policy is invariant when unseen cards are redistributed without changing counts.
    let mut other = (**state).clone();
    let enemy = room.players.iter().find(|p| p.id != actor).ok_or("other")?;
    if !other.stock.is_empty() && !other.hands[&enemy.id].is_empty() {
        let first = other.stock[0].clone();
        other.stock[0] = other.hands[&enemy.id][0].clone();
        other.hands.get_mut(&enemy.id).ok_or("hand")?[0] = first;
    }
    assert_eq!(
        serde_json::to_value(durak::choose(state.view(actor, &room.players)))?,
        serde_json::to_value(durak::choose(other.view(actor, &room.players)))?
    );
    Ok(())
}
#[tokio::test]
async fn computers_complete_using_durable_scheduling_and_history_is_read_only() -> TestResult {
    let e = Engine::new(Store::open(":memory:").await?);
    let code = start(&e, GameId::DurakPerevodnoy, 4, 3).await?;
    let now = chrono::Utc::now().timestamp_millis();
    for step in 1..3000 {
        e.advance_at(&code, &player(0).id, now + step * 5000)
            .await?;
        let v = e.read(&code, &player(0).id).await?;
        if v["phase"] == "finished" {
            break;
        }
        if let Some(action) = durak::choose(serde_json::from_value(v["table"].clone())?) {
            send(&e, &code, &player(0).id, action).await?;
        }
    }
    let a = e.audit(&code, &player(0).id).await?;
    assert_eq!(a["phase"], "finished");
    assert_eq!(audit::verify(&a)?["deals"], 1);
    send(&e, &code, &player(0).id, durak::Action::Rematch).await?;
    let before = e.audit(&code, &player(0).id).await?;
    e.history(&player(0).id, None).await?;
    e.record(&code, &player(0).id).await?;
    assert_eq!(before, e.audit(&code, &player(0).id).await?);
    let mut conn = e.store.pool.acquire().await?;
    let room: Room = Engine::load(&mut conn, &code).await?;
    assert!(matches!(
        room.lifecycle,
        Lifecycle::Playing(GameState::Durak(_))
    ));
    assert_eq!(
        crypto::hash(&room.decks[0].proof)?,
        before["decks"][0]["commitment"]
    );
    Ok(())
}
