use crate::{
    error::{Error, Result, ensure},
    model::Player,
};
use axum::http::StatusCode;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::collections::BTreeMap;
type HmacSha256 = Hmac<Sha256>;
fn unauthorized(message: &str) -> Error {
    Error::new(StatusCode::UNAUTHORIZED, message)
}
pub fn authenticate(init_data: &str, bot_token: &str, now: i64) -> Result<Player> {
    let mut fields = BTreeMap::new();
    for (key, value) in url::form_urlencoded::parse(init_data.as_bytes()) {
        ensure(
            fields
                .insert(key.into_owned(), value.into_owned())
                .is_none(),
            unauthorized("Duplicate Telegram fields"),
        )?;
    }
    let digest = fields
        .remove("hash")
        .ok_or_else(|| unauthorized("Invalid Telegram signature"))?;
    ensure(
        digest.len() == 64
            && digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        unauthorized("Invalid Telegram signature"),
    )?;
    let check = fields
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut secret = HmacSha256::new_from_slice(b"WebAppData").map_err(|_| Error::internal())?;
    secret.update(bot_token.as_bytes());
    let mut signature = HmacSha256::new_from_slice(&secret.finalize().into_bytes())
        .map_err(|_| Error::internal())?;
    signature.update(check.as_bytes());
    let bytes = hex::decode(digest).map_err(|_| unauthorized("Invalid Telegram signature"))?;
    signature
        .verify_slice(&bytes)
        .map_err(|_| unauthorized("Invalid Telegram signature"))?;
    let date = fields
        .get("auth_date")
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| unauthorized("Telegram login expired"))?;
    ensure(
        date <= now.saturating_add(30) && date >= now.saturating_sub(300),
        unauthorized("Telegram login expired"),
    )?;
    #[derive(Deserialize)]
    struct User {
        id: u64,
        first_name: String,
    }
    let user: User = serde_json::from_str(fields.get("user").map(String::as_str).unwrap_or("null"))
        .map_err(|_| unauthorized("Invalid Telegram user"))?;
    ensure(
        user.id > 0 && user.id <= 9_007_199_254_740_991,
        unauthorized("Invalid Telegram user"),
    )?;
    Ok(Player {
        id: format!("telegram:{}", user.id),
        name: user.first_name.chars().take(80).collect(),
    })
}
