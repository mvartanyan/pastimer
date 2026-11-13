use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use pastimer::{
    audit, crypto,
    engine::Engine,
    error::{Error, Result},
    game::{GameRules, HighCard, Outcome},
    http::{self, Config},
    model::*,
    store::Store,
    telegram,
};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tower::ServiceExt;
use uuid::Uuid;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn a() -> Player {
    Player {
        id: "a".into(),
        name: "Alice".into(),
    }
}
fn b() -> Player {
    Player {
        id: "b".into(),
        name: "Bob".into(),
    }
}
async fn setup() -> Result<Engine> {
    Ok(Engine::new(Store::open(":memory:").await?))
}
fn text(value: &Value) -> Result<&str> {
    value.as_str().ok_or_else(Error::internal)
}
fn number(value: &Value) -> Result<u64> {
    value.as_u64().ok_or_else(Error::internal)
}
async fn command(engine: &Engine, code: &str, actor: &str, action: Action) -> Result<Value> {
    let version = number(&engine.read(code, actor).await?["version"])?;
    engine
        .command(
            code,
            actor,
            Command {
                version,
                request_id: Uuid::new_v4().to_string(),
                action,
            },
        )
        .await
}
async fn start(engine: &Engine) -> Result<String> {
    let room = engine.create(a(), GameId::HighCard).await?;
    let code = text(&room["code"])?.to_owned();
    engine.join(&code, b()).await?;
    command(engine, &code, "a", Action::Start).await?;
    Ok(code)
}
async fn acknowledge(engine: &Engine, code: &str) -> Result<()> {
    let room = engine.read(code, "a").await?;
    let action = Action::Acknowledge {
        shuffle_id: u32::try_from(number(&room["deck"]["shuffleId"])?)
            .map_err(|_| Error::internal())?,
        commitment: text(&room["deck"]["commitment"])?.into(),
    };
    for actor in ["a", "b"] {
        command(engine, code, actor, action.clone()).await?;
    }
    Ok(())
}
async fn complete(engine: &Engine, code: &str) -> Result<()> {
    for _ in 0..5 {
        for actor in ["a", "b"] {
            let room = engine.read(code, actor).await?;
            command(
                engine,
                code,
                actor,
                Action::Play {
                    card: text(&room["table"]["hand"][0])?.parse()?,
                },
            )
            .await?;
        }
    }
    Ok(())
}
#[tokio::test]
async fn complete_private_multiplayer_game_and_detect_tampering() -> TestResult {
    let engine = setup().await?;
    let code = start(&engine).await?;
    let room = engine.read(&code, "a").await?;
    assert_eq!(room["phase"], "committing");
    assert_eq!(room["table"]["hand"], json!([]));
    assert!(
        command(
            &engine,
            &code,
            "a",
            Action::Play {
                card: "AS".parse()?
            }
        )
        .await
        .is_err()
    );
    let pending = engine.audit(&code, "a").await?;
    assert!(pending["decks"][0].get("proof").is_none());
    assert!(pending.get("commands").is_none());
    let anchor = pending["events"]
        .as_array()
        .ok_or("events missing")?
        .last()
        .ok_or("empty events")?["hash"]
        .clone();
    acknowledge(&engine, &code).await?;
    let av = engine.read(&code, "a").await?;
    let bv = engine.read(&code, "b").await?;
    let hand = av["table"]["hand"].as_array().ok_or("hand missing")?;
    assert_eq!(hand.len(), 5);
    assert!(av["table"].get("hands").is_none());
    assert!(hand.iter().all(|card| {
        !bv["table"]["hand"]
            .as_array()
            .is_some_and(|other| other.contains(card))
    }));
    assert!(
        command(
            &engine,
            &code,
            "a",
            Action::Play {
                card: text(&bv["table"]["hand"][0])?.parse()?
            }
        )
        .await
        .is_err()
    );
    complete(&engine, &code).await?;
    let mut audit = engine.audit(&code, "a").await?;
    audit["receipts"] = json!([{"roomId":audit["roomId"],"eventHash":anchor}]);
    assert_eq!(audit::verify(&audit)?["moves"], 10);
    let mut altered = audit.clone();
    altered["decks"][0]["proof"]["cards"]
        .as_array_mut()
        .ok_or("cards missing")?
        .reverse();
    assert!(audit::verify(&altered).is_err());
    let mut altered = audit.clone();
    altered["events"][0]["actor"] = json!("mallory");
    assert!(audit::verify(&altered).is_err());
    let mut altered = audit.clone();
    altered["commands"][0]["salt"] = json!("fake");
    assert!(audit::verify(&altered).is_err());
    audit["receipts"][0]["eventHash"] = json!("fake");
    assert!(audit::verify(&audit).is_err());
    assert!(
        command(
            &engine,
            &code,
            "a",
            Action::Play {
                card: "AS".parse()?
            }
        )
        .await
        .is_err()
    );
    Ok(())
}
#[tokio::test]
async fn permissions_conflicts_idempotency_and_transaction_rollback() -> TestResult {
    let engine = setup().await?;
    let created = engine.create(a(), GameId::HighCard).await?;
    let code = text(&created["code"])?;
    assert_eq!(
        engine
            .read(code, "b")
            .await
            .err()
            .ok_or("expected failure")?
            .status,
        StatusCode::FORBIDDEN
    );
    engine.join(code, b()).await?;
    assert!(command(&engine, code, "b", Action::Start).await.is_err());
    let cmd = Command {
        version: 2,
        request_id: Uuid::new_v4().to_string(),
        action: Action::Start,
    };
    let first = engine.command(code, "a", cmd.clone()).await?;
    assert_eq!(engine.command(code, "a", cmd.clone()).await?, first);
    let other = Command {
        action: Action::Play {
            card: "AS".parse()?,
        },
        ..cmd.clone()
    };
    assert!(engine.command(code, "a", other).await.is_err());
    let stale = Command {
        request_id: Uuid::new_v4().to_string(),
        ..cmd
    };
    assert!(engine.command(code, "a", stale).await.is_err());
    let before = engine.read(code, "a").await?;
    assert!(
        command(
            &engine,
            code,
            "a",
            Action::Acknowledge {
                shuffle_id: 1,
                commitment: "fake".into()
            }
        )
        .await
        .is_err()
    );
    assert_eq!(engine.read(code, "a").await?, before);
    assert!(
        engine
            .join(
                code,
                Player {
                    id: "c".into(),
                    name: "Chris".into()
                }
            )
            .await
            .is_err()
    );
    Ok(())
}
#[tokio::test]
async fn sessions_snapshots_and_command_receipts_survive_database_reopen() -> TestResult {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("game.sqlite");
    let path = path.to_str().ok_or("path")?;
    let engine = Engine::new(Store::open(path).await?);
    let room = engine.create(a(), GameId::HighCard).await?;
    let code = text(&room["code"])?;
    engine.join(code, b()).await?;
    let cmd = Command {
        version: 2,
        request_id: Uuid::new_v4().to_string(),
        action: Action::Start,
    };
    let before = engine.command(code, "a", cmd.clone()).await?;
    let secret = engine.store.session(&a()).await?;
    engine.store.pool.close().await;
    let engine = Engine::new(Store::open(path).await?);
    assert_eq!(engine.read(code, "a").await?, before);
    assert_eq!(engine.store.authenticate(&secret).await?, Some(a()));
    assert_eq!(engine.command(code, "a", cmd).await?, before);
    acknowledge(&engine, code).await?;
    assert_eq!(engine.read(code, "a").await?["phase"], "playing");
    Ok(())
}
#[tokio::test]
async fn every_midgame_shuffle_repeats_acknowledgement_gate() -> TestResult {
    let engine = setup().await?;
    let code = start(&engine).await?;
    acknowledge(&engine, &code).await?;
    let mut tx = engine.store.pool.begin_with("BEGIN IMMEDIATE").await?;
    let mut room = Engine::load(&mut tx, &code).await?;
    let Some(GameState::HighCard(state)) = room.lifecycle.state().cloned() else {
        return Err("state missing".into());
    };
    Engine::outcome(
        &mut room,
        Outcome::Shuffle {
            state,
            cards: thousand_deck(),
            events: vec![],
        },
        "a",
    )?;
    Store::save(&mut tx, &room).await?;
    tx.commit().await?;
    assert_eq!(engine.read(&code, "a").await?["deck"]["shuffleId"], 2);
    assert!(
        command(
            &engine,
            &code,
            "b",
            Action::Play {
                card: "AS".parse()?
            }
        )
        .await
        .is_err()
    );
    acknowledge(&engine, &code).await?;
    assert_eq!(engine.read(&code, "a").await?["phase"], "playing");
    Ok(())
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_duplicate_requests_commit_exactly_once() -> TestResult {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("concurrent.sqlite");
    let engine = Engine::new(Store::open(path.to_str().ok_or("path")?).await?);
    let room = engine.create(a(), GameId::HighCard).await?;
    let code = text(&room["code"])?.to_owned();
    engine.join(&code, b()).await?;
    let cmd = Command {
        version: 2,
        request_id: Uuid::new_v4().to_string(),
        action: Action::Start,
    };
    let mut jobs = tokio::task::JoinSet::new();
    for _ in 0..16 {
        let engine = engine.clone();
        let code = code.clone();
        let cmd = cmd.clone();
        jobs.spawn(async move { engine.command(&code, "a", cmd).await });
    }
    while let Some(result) = jobs.join_next().await {
        assert_eq!(result??["version"], 3);
    }
    let audit = engine.audit(&code, "a").await?;
    assert_eq!(
        audit["events"]
            .as_array()
            .ok_or("events")?
            .iter()
            .filter(|e| e["type"] == "game.started")
            .count(),
        1
    );
    Ok(())
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn racing_different_moves_never_overwrite_one_another() -> TestResult {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("concurrent.sqlite");
    let engine = Engine::new(Store::open(path.to_str().ok_or("path")?).await?);
    let code = start(&engine).await?;
    acknowledge(&engine, &code).await?;
    let version = number(&engine.read(&code, "a").await?["version"])?;
    let mut jobs = tokio::task::JoinSet::new();
    for actor in ["a", "b"] {
        let card = text(&engine.read(&code, actor).await?["table"]["hand"][0])?.parse()?;
        let engine = engine.clone();
        let code = code.clone();
        jobs.spawn(async move {
            engine
                .command(
                    &code,
                    actor,
                    Command {
                        version,
                        request_id: Uuid::new_v4().to_string(),
                        action: Action::Play { card },
                    },
                )
                .await
        });
    }
    let mut successes = 0;
    let mut conflicts = 0;
    while let Some(result) = jobs.join_next().await {
        match result? {
            Ok(_) => successes += 1,
            Err(e) if e.status == StatusCode::CONFLICT => conflicts += 1,
            Err(e) => return Err(e.into()),
        }
    }
    assert_eq!((successes, conflicts), (1, 1));
    assert_eq!(
        engine.audit(&code, "a").await?["events"]
            .as_array()
            .ok_or("events")?
            .iter()
            .filter(|e| e["type"] == "move.committed")
            .count(),
        1
    );
    Ok(())
}
#[test]
fn telegram_signatures_expiry_and_duplicate_fields() -> TestResult {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let token = "test-token";
    let now = 1_700_000_000;
    let user = json!({"id":42,"first_name":"Alice"}).to_string();
    let date = now.to_string();
    let check = format!("auth_date={date}\nuser={user}");
    let mut secret = Hmac::<Sha256>::new_from_slice(b"WebAppData")?;
    secret.update(token.as_bytes());
    let mut mac = Hmac::<Sha256>::new_from_slice(&secret.finalize().into_bytes())?;
    mac.update(check.as_bytes());
    let hash = hex::encode(mac.finalize().into_bytes());
    let init = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("auth_date", &date)
        .append_pair("user", &user)
        .append_pair("hash", &hash)
        .finish();
    assert_eq!(
        telegram::authenticate(&init, token, now)?,
        Player {
            id: "telegram:42".into(),
            name: "Alice".into()
        }
    );
    assert!(telegram::authenticate(&init, token, now + 301).is_err());
    assert!(telegram::authenticate(&init, "wrong-token", now).is_err());
    assert!(telegram::authenticate(&format!("{init}&auth_date=1"), token, now).is_err());
    Ok(())
}
async fn request(
    app: &Router,
    method: &str,
    path: &str,
    cookie: Option<&str>,
    body: Option<Value>,
    origin: Option<&str>,
) -> Result<(StatusCode, Value, Option<String>)> {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost");
    if let Some(cookie) = cookie {
        req = req.header("cookie", cookie);
    }
    if let Some(origin) = origin {
        req = req.header("origin", origin);
    }
    let body = if let Some(body) = body {
        req = req.header("content-type", "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };
    let mut req = req.body(body).map_err(|_| Error::internal())?;
    req.extensions_mut().insert(ConnectInfo(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        1234,
    )));
    let response = app
        .clone()
        .oneshot(req)
        .await
        .map_err(|_| Error::internal())?;
    let status = response.status();
    let cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_owned());
    let bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|_| Error::internal())?
        .to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Ok((status, body, cookie))
}
#[tokio::test]
async fn http_auth_isolation_typed_input_and_origin_validation() -> TestResult {
    let app = http::router(
        Store::open(":memory:").await?,
        Config {
            dev_auth: true,
            ..Config::default()
        },
    )?;
    assert_eq!(
        request(&app, "GET", "/api/me", None, None, None).await?.0,
        StatusCode::UNAUTHORIZED
    );
    let (status, _, cookie) = request(
        &app,
        "POST",
        "/api/auth",
        None,
        Some(json!({"name":"Alice"})),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let cookie = cookie.ok_or("cookie")?;
    let (_, created, _) = request(
        &app,
        "POST",
        "/api/rooms",
        Some(&cookie),
        Some(json!({"game":"high-card-demo"})),
        None,
    )
    .await?;
    let path = format!("/api/rooms/{}", text(&created["code"])?);
    assert_eq!(
        request(&app, "GET", &path, Some(&cookie), None, None)
            .await?
            .0,
        StatusCode::OK
    );
    let (_, _, cookie2) = request(
        &app,
        "POST",
        "/api/auth",
        None,
        Some(json!({"name":"Bob"})),
        None,
    )
    .await?;
    assert_eq!(
        request(&app, "GET", &path, cookie2.as_deref(), None, None)
            .await?
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/rooms",
            Some(&cookie),
            Some(json!({"game":"high-card-demo"})),
            Some("https://evil.example")
        )
        .await?
        .0,
        StatusCode::FORBIDDEN
    );
    for action in [
        Value::Null,
        json!({"type":"play","card":"garbage"}),
        json!({"type":"play","card":"AS","unexpected":true}),
    ] {
        assert_eq!(
            request(
                &app,
                "POST",
                &format!("{path}/commands"),
                Some(&cookie),
                Some(json!({"version":1,"requestId":Uuid::new_v4(),"action":action})),
                None
            )
            .await?
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        request(
            &app,
            "POST",
            &format!("{path}/commands"),
            Some(&cookie),
            Some(json!({"version":-1,"requestId":Uuid::new_v4(),"action":{"type":"start"}})),
            None
        )
        .await?
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(&app, "GET", "/", None, None, None).await?.0,
        StatusCode::OK
    );
    assert!(
        http::router(
            Store::open(":memory:").await?,
            Config {
                production: true,
                dev_auth: true,
                ..Config::default()
            }
        )
        .is_err()
    );
    Ok(())
}
#[tokio::test]
async fn node_snapshots_audit_hashes_and_retry_receipts_remain_compatible() -> TestResult {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/node-v1.json"))?;
    assert_eq!(audit::verify(&fixture["audit"])?["moves"], 10);
    let engine = setup().await?;
    for phase in ["lobby", "committing", "finished"] {
        let room: Room = serde_json::from_value(fixture[phase].clone())?;
        let mut conn = engine.store.pool.acquire().await?;
        Store::save(&mut conn, &room).await?;
        drop(conn);
        assert_eq!(engine.read(&room.code, "fixture:a").await?["phase"], phase);
    }
    let committing: Room = serde_json::from_value(fixture["committing"].clone())?;
    let receipt = &fixture["startReceipt"];
    let mut tx = engine.store.pool.begin_with("BEGIN IMMEDIATE").await?;
    Store::save(&mut tx, &committing).await?;
    sqlx::query("INSERT INTO commands VALUES(?,?,?,?)")
        .bind(&committing.id)
        .bind(text(&receipt["actor"])?)
        .bind(text(&receipt["requestId"])?)
        .bind(text(&receipt["fingerprint"])?)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let retried = engine
        .command(
            &committing.code,
            "fixture:a",
            Command {
                version: number(&receipt["version"])?,
                request_id: text(&receipt["requestId"])?.into(),
                action: Action::Start,
            },
        )
        .await?;
    assert_eq!(retried["version"], committing.version);
    let deck = &committing.decks[0];
    for actor in ["fixture:a", "fixture:b"] {
        command(
            &engine,
            &committing.code,
            actor,
            Action::Acknowledge {
                shuffle_id: deck.proof.shuffle_id,
                commitment: deck.commitment.clone(),
            },
        )
        .await?;
    }
    assert_eq!(
        engine.read(&committing.code, "fixture:a").await?["phase"],
        "playing"
    );
    Ok(())
}
#[test]
fn typed_decks_and_canonical_hash_vector() -> TestResult {
    assert_eq!(standard_deck().len(), 52);
    assert_eq!(thousand_deck().len(), 24);
    assert!("1S".parse::<Card>().is_err());
    assert!("AS🂡".parse::<Card>().is_err());
    assert_eq!(
        crypto::hash(&json!({"b":2,"a":1}))?,
        "43258cff783fe7036d8a43033f830adfc60ec037382473548ac742b888292777"
    );
    let Outcome::Shuffle { cards, .. } = HighCard::initialize(&[a(), b()]) else {
        return Err("expected initial shuffle".into());
    };
    assert_eq!(cards.len(), 52);
    Ok(())
}
