use crate::{
    crypto::{hash, token},
    error::{Error, Result},
    model::{Player, Room},
};
use sqlx::{
    SqliteConnection, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{path::Path, time::Duration};
#[derive(Clone)]
pub struct Store {
    pub pool: SqlitePool,
}
impl Store {
    pub async fn open(path: &str) -> Result<Self> {
        if path != ":memory:"
            && let Some(parent) = Path::new(path)
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
        {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|_| Error::internal())?;
        }
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(5))
            .journal_mode(SqliteJournalMode::Wal);
        let pool = SqlitePoolOptions::new()
            .max_connections(if path == ":memory:" { 1 } else { 8 })
            .acquire_timeout(Duration::from_secs(10))
            .connect_with(options)
            .await?;
        sqlx::raw_sql("CREATE TABLE IF NOT EXISTS rooms(id TEXT PRIMARY KEY, code TEXT UNIQUE NOT NULL, snapshot TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS commands(room_id TEXT NOT NULL, actor TEXT NOT NULL, request_id TEXT NOT NULL, fingerprint TEXT NOT NULL, PRIMARY KEY(room_id,actor,request_id));
          CREATE TABLE IF NOT EXISTS sessions(token_hash TEXT PRIMARY KEY, player TEXT NOT NULL, expires INTEGER NOT NULL);")
          .execute(&pool).await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS player_rooms(player_id TEXT PRIMARY KEY, room_code TEXT NOT NULL)")
            .execute(&pool).await?;
        Ok(Self { pool })
    }
    pub async fn get(conn: &mut SqliteConnection, code: &str) -> Result<Option<Room>> {
        let row = sqlx::query_scalar::<_, String>("SELECT snapshot FROM rooms WHERE code=?")
            .bind(code)
            .fetch_optional(conn)
            .await?;
        row.map(|row| serde_json::from_str(&row).map_err(Error::from))
            .transpose()
    }
    pub async fn save(conn: &mut SqliteConnection, room: &Room) -> Result<()> {
        sqlx::query("INSERT INTO rooms VALUES(?,?,?) ON CONFLICT(id) DO UPDATE SET snapshot=excluded.snapshot")
            .bind(&room.id).bind(&room.code).bind(serde_json::to_string(room)?).execute(conn).await?;
        Ok(())
    }
    pub async fn remember(conn: &mut SqliteConnection, actor: &str, code: &str) -> Result<()> {
        sqlx::query("INSERT INTO player_rooms VALUES(?,?) ON CONFLICT(player_id) DO UPDATE SET room_code=excluded.room_code")
            .bind(actor).bind(code).execute(conn).await?;
        Ok(())
    }
    pub async fn recent(&self, actor: &str) -> Result<Option<String>> {
        let saved = sqlx::query_scalar("SELECT room_code FROM player_rooms WHERE player_id=?")
            .bind(actor)
            .fetch_optional(&self.pool)
            .await?;
        if saved.is_some() {
            return Ok(saved);
        }
        // Recover membership for tables created before the reconnect index existed.
        Ok(sqlx::query_scalar("SELECT code FROM rooms WHERE EXISTS (SELECT 1 FROM json_each(rooms.snapshot,'$.players') AS p WHERE json_extract(p.value,'$.id')=?) ORDER BY rowid DESC LIMIT 1")
            .bind(actor).fetch_optional(&self.pool).await?)
    }
    pub async fn session(&self, player: &Player) -> Result<String> {
        let secret = token();
        let now = chrono::Utc::now().timestamp_millis();
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query("DELETE FROM sessions WHERE expires < ?")
            .bind(now)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO sessions VALUES(?,?,?)")
            .bind(hash(&secret)?)
            .bind(serde_json::to_string(player)?)
            .bind(now + 86_400_000)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(secret)
    }
    pub async fn authenticate(&self, secret: &str) -> Result<Option<Player>> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT player FROM sessions WHERE token_hash=? AND expires>?",
        )
        .bind(hash(secret)?)
        .bind(chrono::Utc::now().timestamp_millis())
        .fetch_optional(&self.pool)
        .await?;
        row.map(|r| serde_json::from_str(&r).map_err(Error::from))
            .transpose()
    }
}
