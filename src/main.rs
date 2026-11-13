use pastimer::{
    http::{Config, router},
    store::Store,
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        dev_auth: std::env::var("DEV_AUTH").as_deref() == Ok("1"),
        production: std::env::var("NODE_ENV").as_deref() == Ok("production")
            || std::env::var("PASTIMER_ENV").as_deref() == Ok("production"),
        bot_token: std::env::var("TELEGRAM_BOT_TOKEN").ok(),
    };
    let db = std::env::var("DATABASE_PATH").unwrap_or_else(|_| ".data/pastimer.sqlite".into());
    let store = Store::open(&db).await?;
    let app = router(store.clone(), config)?;
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse::<u16>()?;
    let listener = tokio::net::TcpListener::bind((host.as_str(), port)).await?;
    println!("Pastimer listening on http://{}", listener.local_addr()?);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    store.pool.close().await;
    Ok(())
}
async fn shutdown() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    {
        if let Ok(mut term) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            tokio::select! {()=ctrl_c=>{},_=term.recv()=>{}}
            return;
        }
    }
    ctrl_c.await;
}
