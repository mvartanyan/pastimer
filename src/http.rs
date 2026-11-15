use crate::{
    engine::Engine,
    error::{Error, Result, ensure},
    model::{Command, GameId, Player},
    store::Store,
    telegram,
};
use axum::{
    Json, Router,
    extract::{
        ConnectInfo, DefaultBodyLimit, Path, Query, Request, State, rejection::JsonRejection,
    },
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use uuid::Uuid;
#[derive(Clone, Default)]
pub struct Config {
    pub dev_auth: bool,
    pub production: bool,
    pub bot_token: Option<String>,
}
struct Bucket {
    count: u32,
    until: Instant,
}
#[derive(Clone)]
struct App {
    engine: Engine,
    config: Arc<Config>,
    buckets: Arc<Mutex<HashMap<IpAddr, Bucket>>>,
}
pub fn router(store: Store, config: Config) -> Result<Router> {
    ensure(
        !(config.production && config.dev_auth),
        Error::bad("DEV_AUTH is forbidden in production"),
    )?;
    let state = App {
        engine: Engine::new(store),
        config: Arc::new(config),
        buckets: Arc::default(),
    };
    Ok(Router::new()
        .route("/",get(||async{([(header::CONTENT_TYPE,"text/html; charset=utf-8")],include_str!("../public/index.html"))}))
        .route("/app.js",get(||async{([(header::CONTENT_TYPE,"text/javascript")],include_str!("../public/app.js"))}))
        .route("/i18n.js",get(||async{([(header::CONTENT_TYPE,"text/javascript")],include_str!("../public/i18n.js"))}))
        .route("/archive.js",get(||async{([(header::CONTENT_TYPE,"text/javascript")],include_str!("../public/archive.js"))}))
        .route("/cards.js",get(||async{([(header::CONTENT_TYPE,"text/javascript")],include_str!("../public/cards.js"))}))
        .route("/thousand.js",get(||async{([(header::CONTENT_TYPE,"text/javascript")],include_str!("../public/thousand.js"))}))
        .route("/style.css",get(||async{([(header::CONTENT_TYPE,"text/css")],include_str!("../public/style.css"))}))
        .route("/api/health",get(||async{Json(json!({"ok":true}))}))
        .route("/api/games",get(||async{Json(json!([{"id":"high-card-demo","name":"High card · platform demo","version":1,"minPlayers":2,"maxPlayers":4},{"id":"thousand","name":"Тысяча · Thousand","version":1,"minPlayers":3,"maxPlayers":4}]))}))
        .route("/api/auth",post(login)).route("/api/me",get(me))
        .route("/api/rooms",post(create)).route("/api/rooms/{code}",get(read))
        .route("/api/rooms/active",get(active_room))
        .route("/api/rooms/history",get(history))
        .route("/api/rooms/{code}/record",get(record))
        .route("/api/rooms/{code}/join",post(join)).route("/api/rooms/{code}/commands",post(command))
        .route("/api/rooms/{code}/audit",get(audit))
        .fallback(||async{Error::new(StatusCode::NOT_FOUND,"Not found")})
        .layer(DefaultBodyLimit::max(16_384))
        .layer(middleware::from_fn_with_state(state.clone(),boundary)).with_state(state))
}
fn header_text<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}
async fn boundary(State(state): State<App>, request: Request, next: Next) -> Response {
    let check = (|| -> Result<()> {
        if request.uri().path().starts_with("/api/") {
            if let Some(peer) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
                let now = Instant::now();
                let mut buckets = state.buckets.lock().map_err(|_| Error::internal())?;
                if buckets.len() >= 10_000 {
                    buckets.retain(|_, b| b.until > now);
                }
                ensure(
                    buckets.contains_key(&peer.0.ip()) || buckets.len() < 10_000,
                    Error::new(StatusCode::TOO_MANY_REQUESTS, "Too many requests"),
                )?;
                let bucket = buckets.entry(peer.0.ip()).or_insert(Bucket {
                    count: 0,
                    until: now + Duration::from_secs(60),
                });
                if bucket.until <= now {
                    bucket.count = 0;
                    bucket.until = now + Duration::from_secs(60);
                }
                bucket.count += 1;
                ensure(
                    bucket.count <= 1200,
                    Error::new(StatusCode::TOO_MANY_REQUESTS, "Too many requests"),
                )?;
            }
            if request.method() != axum::http::Method::GET {
                let headers = request.headers();
                ensure(
                    header_text(headers, "content-type").is_some_and(|s| {
                        s.split(';')
                            .next()
                            .is_some_and(|s| s.trim().eq_ignore_ascii_case("application/json"))
                    }),
                    Error::new(StatusCode::UNSUPPORTED_MEDIA_TYPE, "JSON required"),
                )?;
                if let Some(origin) = header_text(headers, "origin") {
                    let url = url::Url::parse(origin)
                        .map_err(|_| Error::new(StatusCode::FORBIDDEN, "Invalid origin"))?;
                    let host = &url[url::Position::BeforeHost..url::Position::AfterPort];
                    ensure(
                        matches!(url.scheme(), "http" | "https")
                            && Some(host) == header_text(headers, "host"),
                        Error::new(StatusCode::FORBIDDEN, "Cross-origin request rejected"),
                    )?;
                }
                ensure(
                    header_text(headers, "sec-fetch-site") != Some("cross-site"),
                    Error::new(StatusCode::FORBIDDEN, "Cross-site request rejected"),
                )?;
            }
        }
        Ok(())
    })();
    let mut response = match check {
        Ok(()) => next.run(request).await,
        Err(error) => error.into_response(),
    };
    for (name, value) in [
        ("x-content-type-options", "nosniff"),
        ("referrer-policy", "no-referrer"),
        ("cache-control", "no-store"),
    ] {
        response.headers_mut().insert(
            header::HeaderName::from_static(name),
            header::HeaderValue::from_static(value),
        );
    }
    response
}
fn parsed<T>(body: std::result::Result<Json<T>, JsonRejection>) -> Result<T> {
    body.map(|Json(body)| body).map_err(|error| {
        Error::new(
            if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
                error.status()
            } else {
                StatusCode::BAD_REQUEST
            },
            "Invalid JSON request",
        )
    })
}
async fn player(state: &App, headers: &HeaderMap) -> Result<Player> {
    let secret = header_text(headers, "cookie")
        .and_then(|s| {
            s.split(';')
                .find_map(|s| s.trim().strip_prefix("pastimer_session="))
        })
        .ok_or_else(|| Error::new(StatusCode::UNAUTHORIZED, "Sign in first"))?;
    state
        .engine
        .store
        .authenticate(secret)
        .await?
        .ok_or_else(|| Error::new(StatusCode::UNAUTHORIZED, "Sign in first"))
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Login {
    Telegram {
        #[serde(rename = "initData")]
        init_data: String,
    },
    Local {
        name: String,
    },
}
async fn login(
    State(state): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    body: std::result::Result<Json<Login>, JsonRejection>,
) -> Result<Response> {
    let p = match parsed(body)? {
        Login::Telegram { init_data } => telegram::authenticate(
            &init_data,
            state.config.bot_token.as_deref().ok_or_else(|| {
                Error::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Telegram authentication is not configured",
                )
            })?,
            chrono::Utc::now().timestamp(),
        )?,
        Login::Local { name } => {
            ensure(
                state.config.dev_auth && peer.ip().is_loopback(),
                Error::new(StatusCode::UNAUTHORIZED, "Telegram login required"),
            )?;
            ensure(
                !name.trim().is_empty() && name.chars().count() <= 80,
                Error::bad("Enter a display name"),
            )?;
            Player {
                id: format!("dev:{}", Uuid::new_v4()),
                name: name.trim().into(),
            }
        }
    };
    let secret = state.engine.store.session(&p).await?;
    let cookie = format!(
        "pastimer_session={secret}; Path=/; HttpOnly; SameSite=Strict; Max-Age=86400{}",
        if state.config.production {
            "; Secure"
        } else {
            ""
        }
    );
    Ok(([(header::SET_COOKIE, cookie)], Json(p)).into_response())
}
async fn me(State(state): State<App>, headers: HeaderMap) -> Result<Json<Player>> {
    Ok(Json(player(&state, &headers).await?))
}
async fn active_room(State(state): State<App>, headers: HeaderMap) -> Result<Json<Option<Value>>> {
    let p = player(&state, &headers).await?;
    let room = if let Some(code) = state.engine.store.recent(&p.id).await? {
        Some(state.engine.read(&code, &p.id).await?)
    } else {
        None
    };
    Ok(Json(room))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Create {
    game: GameId,
    #[serde(default)]
    options: Option<crate::thousand::Settings>,
    #[serde(default)]
    computer_count: u8,
}
async fn create(
    State(state): State<App>,
    headers: HeaderMap,
    body: std::result::Result<Json<Create>, JsonRejection>,
) -> Result<Json<Value>> {
    let p = player(&state, &headers).await?;
    let body = parsed(body)?;
    Ok(Json(
        state
            .engine
            .create_with_computers(p, body.game, body.options, body.computer_count)
            .await?,
    ))
}
async fn read(
    State(state): State<App>,
    Path(code): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>> {
    let p = player(&state, &headers).await?;
    Ok(Json(state.engine.read(&code, &p.id).await?))
}
async fn join(
    State(state): State<App>,
    Path(code): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>> {
    let p = player(&state, &headers).await?;
    Ok(Json(state.engine.join(&code, p).await?))
}
async fn audit(
    State(state): State<App>,
    Path(code): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>> {
    let p = player(&state, &headers).await?;
    Ok(Json(state.engine.audit(&code, &p.id).await?))
}
async fn command(
    State(state): State<App>,
    Path(code): Path<String>,
    headers: HeaderMap,
    body: std::result::Result<Json<Command>, JsonRejection>,
) -> Result<Json<Value>> {
    let p = player(&state, &headers).await?;
    Ok(Json(
        state.engine.command(&code, &p.id, parsed(body)?).await?,
    ))
}

#[derive(Deserialize)]
struct HistoryQuery {
    before: Option<i64>,
}
async fn history(
    State(state): State<App>,
    headers: HeaderMap,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Value>> {
    let player = player(&state, &headers).await?;
    Ok(Json(state.engine.history(&player.id, query.before).await?))
}
async fn record(
    State(state): State<App>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<Value>> {
    let player = player(&state, &headers).await?;
    Ok(Json(state.engine.record(&code, &player.id).await?))
}
