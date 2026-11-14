use pastimer::{
    audit,
    engine::Engine,
    error::{Error, Result},
    model::{Action, Command, GameId, GameState, Lifecycle, Player},
    store::Store,
    thousand::{self, Settings},
};
use serde_json::{Value, json};
use uuid::Uuid;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn player(i: u8) -> Player {
    Player {
        id: format!("p{i}"),
        name: format!("Player {i}"),
    }
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(Error::internal)
}
async fn setup(path: &str, n: u8) -> Result<(Engine, String)> {
    let engine = Engine::new(Store::open(path).await?);
    let room = engine
        .create_configured(
            player(0),
            GameId::Thousand,
            Some(Settings {
                player_count: n,
                barrel_threshold: 900,
            }),
        )
        .await?;
    let code = text(&room["code"])?.to_owned();
    for i in 1..n {
        engine.join(&code, player(i)).await?;
    }
    let version = engine.read(&code, "p0").await?["version"]
        .as_u64()
        .ok_or_else(Error::internal)?;
    engine
        .command(
            &code,
            "p0",
            Command {
                version,
                request_id: Uuid::new_v4().to_string(),
                action: Action::Start,
            },
        )
        .await?;
    Ok((engine, code))
}
async fn cmd(e: &Engine, code: &str, actor: &str, action: thousand::Action) -> Result<Value> {
    let version = e.read(code, actor).await?["version"]
        .as_u64()
        .ok_or_else(Error::internal)?;
    e.command(
        code,
        actor,
        Command {
            version,
            request_id: Uuid::new_v4().to_string(),
            action: Action::Thousand { action },
        },
    )
    .await
}
async fn shuffle(e: &Engine, code: &str) -> Result<Value> {
    let room = e.read(code, "p0").await?;
    cmd(
        e,
        code,
        text(&room["table"]["dealer"])?,
        thousand::Action::Shuffle,
    )
    .await
}
async fn prepare(e: &Engine, code: &str) -> Result<Value> {
    let mut room = shuffle(e, code).await?;
    while room["table"]["phase"] == "auction" {
        let id = text(&room["table"]["turn"])?.to_owned();
        room = cmd(e, code, &id, thousand::Action::Pass).await?;
    }
    let id = text(&room["table"]["contractor"])?.to_owned();
    room = cmd(e, code, &id, thousand::Action::TakeStock).await?;
    let hand = room["table"]["hand"]
        .as_array()
        .ok_or_else(Error::internal)?;
    let recipients = room["table"]["controls"]["recipients"]
        .as_array()
        .ok_or_else(Error::internal)?;
    let transfers = recipients
        .iter()
        .enumerate()
        .map(|(i, p)| {
            Ok(thousand::Transfer {
                player: text(&p["id"])?.into(),
                card: text(&hand[i])?.parse()?,
                public: i == 0,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    room = cmd(e, code, &id, thousand::Action::Distribute { transfers }).await?;
    let amount = room["table"]["order"]
        .as_u64()
        .ok_or_else(Error::internal)? as u16;
    cmd(e, code, &id, thousand::Action::Order { amount }).await
}
#[tokio::test]
async fn http_engine_plays_and_verifies_a_deal_without_acknowledgements() -> TestResult {
    for n in [3, 4] {
        let (e, code) = setup(":memory:", n).await?;
        let mut room = prepare(&e, &code).await?;
        let before = e.audit(&code, "p0").await?;
        assert_eq!(before["protocol"], "pastimer-audit-v2");
        assert_eq!(before["deals"], json!([]));
        assert!(before["decks"][0].get("proof").is_none());
        assert!(
            !before["events"]
                .as_array()
                .ok_or("events")?
                .iter()
                .any(|v| v["type"] == "deck.acknowledged")
        );
        assert_eq!(audit::verify(&before)?["deals"], 0);
        for _ in 0..24 {
            let id = text(&room["table"]["turn"])?.to_owned();
            let own = e.read(&code, &id).await?;
            let card = text(&own["table"]["controls"]["legalCards"][0])?.parse()?;
            room = cmd(
                &e,
                &code,
                &id,
                thousand::Action::Play {
                    card,
                    announce: false,
                },
            )
            .await?;
            if room["table"]["phase"] == "closing" {
                assert_eq!(
                    room["table"]["trickLeader"],
                    room["table"]["plays"][0]["player"]
                );
                let winner = room["table"]["trickWinner"].clone();
                assert!(winner.is_string());
                let close_at = room["table"]["closeAt"].as_i64().ok_or("deadline")?;
                e.advance_at(&code, "p0", close_at - 1).await?;
                assert_eq!(e.read(&code, "p0").await?["table"]["phase"], "closing");
                e.advance_at(&code, "p0", close_at).await?;
                room = e.read(&code, "p0").await?;
                assert!(room["table"]["trickWinner"].is_null());
                if room["table"]["phase"] == "trick" {
                    assert_eq!(room["table"]["trickLeader"], winner);
                }
            }
        }
        assert_eq!(room["table"]["phase"], "awaitShuffle");
        let export = e.audit(&code, "p0").await?;
        assert_eq!(audit::verify(&export)?["deals"], 1);
        assert_eq!(
            export["deals"][0]["tricks"]
                .as_array()
                .ok_or("tricks")?
                .len(),
            8
        );
        assert_eq!(
            export["deals"][0]["movements"]
                .as_array()
                .ok_or("movements")?
                .len(),
            77
        );
        let piles = room["table"]["piles"].as_array().ok_or("piles")?;
        for pile in piles {
            let last = export["deals"][0]["tricks"]
                .as_array()
                .ok_or("tricks")?
                .iter()
                .rev()
                .find(|t| t["winner"] == pile["player"]);
            assert_eq!(pile["top"], last.cloned().unwrap_or(Value::Null));
        }
        let mut forged = export.clone();
        forged["deals"][0]["movements"][0]["to"] = json!({"kind":"stock"});
        assert!(audit::verify(&forged).is_err());
        let mut forged = export.clone();
        forged["decks"][0]["proof"]["salt"] = json!("fake");
        assert!(audit::verify(&forged).is_err());
        // Re-hashing a changed command must not hide a disagreement with the recorded plays.
        let mut forged = export.clone();
        let index = forged["commands"]
            .as_array()
            .ok_or("commands")?
            .iter()
            .position(|c| c["action"]["action"]["type"] == "play")
            .ok_or("play")?;
        let old = pastimer::crypto::hash(&forged["commands"][index])?;
        forged["commands"][index]["action"]["action"]["card"] = json!("2C");
        let changed = pastimer::crypto::hash(&forged["commands"][index])?;
        let mut previous = "0".repeat(64);
        for event in forged["events"].as_array_mut().ok_or("events")? {
            if event["data"]["commitment"] == old {
                event["data"]["commitment"] = json!(changed);
            }
            event["previousHash"] = json!(previous);
            event.as_object_mut().ok_or("event")?.remove("hash");
            previous = pastimer::crypto::hash(event)?;
            event["hash"] = json!(previous);
        }
        assert!(audit::verify(&forged).is_err());
        // A new deal remains hidden while the previous deal stays independently verifiable.
        shuffle(&e, &code).await?;
        let next = e.audit(&code, "p0").await?;
        assert_eq!(audit::verify(&next)?["deals"], 1);
        assert!(next["decks"][0].get("proof").is_some());
        assert!(next["decks"][1].get("proof").is_none());
    }
    Ok(())
}
#[tokio::test]
async fn sqlite_restart_recovers_pending_trick_deadline_and_exact_retry() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("game.sqlite");
    let path = path.to_str().ok_or("path")?;
    let (e, code) = setup(path, 3).await?;
    let mut room = prepare(&e, &code).await?;
    let mut last = None;
    for _ in 0..3 {
        let id = text(&room["table"]["turn"])?.to_owned();
        let own = e.read(&code, &id).await?;
        let command = Command {
            version: own["version"].as_u64().ok_or("version")?,
            request_id: Uuid::new_v4().to_string(),
            action: Action::Thousand {
                action: thousand::Action::Play {
                    card: text(&own["table"]["controls"]["legalCards"][0])?.parse()?,
                    announce: false,
                },
            },
        };
        room = e.command(&code, &id, command.clone()).await?;
        last = Some((id, command));
    }
    let deadline = room["table"]["closeAt"].as_i64().ok_or("deadline")?;
    e.store.pool.close().await;
    let e = Engine::new(Store::open(path).await?);
    assert_eq!(e.read(&code, "p0").await?["table"]["closeAt"], deadline);
    let (id, command) = last.ok_or("command")?;
    let count = e.audit(&code, "p0").await?["events"]
        .as_array()
        .ok_or("events")?
        .len();
    e.command(&code, &id, command).await?;
    assert_eq!(
        e.audit(&code, "p0").await?["events"]
            .as_array()
            .ok_or("events")?
            .len(),
        count
    );
    e.advance_at(&code, "p0", deadline).await?;
    let room = e.read(&code, "p0").await?;
    assert_eq!(room["table"]["phase"], "trick");
    assert_eq!(
        room["table"]["piles"]
            .as_array()
            .ok_or("piles")?
            .iter()
            .map(|p| p["count"].as_u64().unwrap_or(0))
            .sum::<u64>(),
        1
    );
    Ok(())
}
#[tokio::test]
async fn invalid_distribution_rolls_back_every_card_and_event() -> TestResult {
    let (e, code) = setup(":memory:", 4).await?;
    let mut room = shuffle(&e, &code).await?;
    while room["table"]["phase"] == "auction" {
        let id = text(&room["table"]["turn"])?.to_owned();
        room = cmd(&e, &code, &id, thousand::Action::Pass).await?;
    }
    let id = text(&room["table"]["contractor"])?.to_owned();
    let before = cmd(&e, &code, &id, thousand::Action::TakeStock).await?;
    let recipients = before["table"]["controls"]["recipients"]
        .as_array()
        .ok_or("recipients")?;
    let valid = text(&before["table"]["hand"][0])?.parse()?;
    let other = recipients[0]["id"].as_str().ok_or("id")?;
    let forbidden = e.read(&code, other).await?["table"]["hand"][0]
        .as_str()
        .ok_or("card")?
        .parse()?;
    let bad = thousand::Action::Distribute {
        transfers: vec![
            thousand::Transfer {
                player: text(&recipients[0]["id"])?.into(),
                card: valid,
                public: true,
            },
            thousand::Transfer {
                player: text(&recipients[1]["id"])?.into(),
                card: forbidden,
                public: false,
            },
        ],
    };
    assert!(cmd(&e, &code, &id, bad).await.is_err());
    let mut after = e.read(&code, &id).await?;
    let mut before = before;
    after["table"]["serverNow"] = Value::Null;
    before["table"]["serverNow"] = Value::Null;
    assert_eq!(after, before);
    Ok(())
}
#[tokio::test]
async fn settings_capacity_and_tagged_snapshot_round_trip() -> TestResult {
    let store = Store::open(":memory:").await?;
    let e = Engine::new(store);
    assert!(
        e.create_configured(
            player(0),
            GameId::Thousand,
            Some(Settings {
                player_count: 2,
                barrel_threshold: 900
            })
        )
        .await
        .is_err()
    );
    let (e, code) = setup(":memory:", 3).await?;
    assert!(e.join(&code, player(3)).await.is_err());
    let mut conn = e.store.pool.acquire().await?;
    let room = Engine::load(&mut conn, &code).await?;
    let encoded = serde_json::to_value(&room)?;
    assert_eq!(encoded["state"]["game"], "thousand");
    assert!(matches!(
        room.lifecycle,
        Lifecycle::Playing(GameState::Thousand(_))
    ));
    Ok(())
}

#[tokio::test]
async fn cancelled_deal_reveals_and_replays_its_complete_initial_deck() -> TestResult {
    use pastimer::{
        crypto::{hash, token},
        model::{Card, Deck, DeckProof, MoveProof},
    };
    let (e, code) = setup(":memory:", 3).await?;
    // A deterministic committed-deal fixture; production randomness has no override.
    let mut tx = e.store.pool.begin_with("BEGIN IMMEDIATE").await?;
    let mut room = Engine::load(&mut tx, &code).await?;
    let Some(GameState::Thousand(state)) = room.lifecycle.state() else {
        return Err("state".into());
    };
    let state = (**state).clone();
    let actor = room.players[state.dealer].id.clone();
    let holder = room.players[(state.dealer + 1) % 3].id.clone();
    let nines = "9C 9D 9H 9S AC AD AH"
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<Vec<Card>>>()?;
    let rest: Vec<_> = pastimer::model::thousand_deck()
        .into_iter()
        .filter(|c| !nines.contains(c))
        .collect();
    let mut cards = vec![];
    for (i, nine) in nines.iter().enumerate() {
        cards.extend([nine.clone(), rest[i].clone(), rest[i + 7].clone()]);
    }
    cards.extend_from_slice(&rest[14..]);
    let proof = MoveProof {
        deal_id: Some(1),
        actor: actor.clone(),
        version: room.version,
        action: Action::Thousand {
            action: thousand::Action::Shuffle,
        },
        salt: token(),
    };
    Engine::event(
        &mut room,
        &actor,
        "move.committed",
        json!({"deal":1,"commitment":hash(&proof)?}),
    )?;
    room.commands.push(proof);
    let proof = DeckProof {
        protocol: "pastimer-deck-v1".into(),
        room_id: room.id.clone(),
        shuffle_id: 1,
        cards: cards.clone(),
        salt: token(),
    };
    let commitment = hash(&proof)?;
    Engine::event(
        &mut room,
        &actor,
        "deck.committed",
        json!({"protocol":proof.protocol,"shuffleId":1,"deal":1,"count":24,"commitment":commitment}),
    )?;
    room.decks.push(Deck {
        proof,
        commitment,
        acknowledgements: vec![],
    });
    let next = state.deal_cards(cards, &room.players)?;
    Engine::thousand_outcome(&mut room, next, &actor)?;
    room.version += 1;
    Store::save(&mut tx, &room).await?;
    tx.commit().await?;
    let cancelled = cmd(&e, &code, &holder, thousand::Action::FourNines).await?;
    assert_eq!(cancelled["table"]["dealer"], actor);
    let audit = e.audit(&code, "p0").await?;
    assert_eq!(audit["deals"][0]["status"], "cancelled");
    assert_eq!(
        audit["deals"][0]["movements"]
            .as_array()
            .ok_or("moves")?
            .len(),
        24
    );
    assert_eq!(audit::verify(&audit)?["deals"], 1);
    shuffle(&e, &code).await?;
    let audit = e.audit(&code, "p0").await?;
    assert_eq!(audit::verify(&audit)?["deals"], 1);
    assert!(audit["decks"][1].get("proof").is_none());
    Ok(())
}
