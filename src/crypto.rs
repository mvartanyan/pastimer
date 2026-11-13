use crate::error::Result;
use rand::{RngCore, rngs::OsRng, seq::SliceRandom};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Protocol v1 canonical JSON: sorted ASCII field names; preserved array order.
/// Protocol payloads use integer numbers, never floating-point values.
pub fn hash<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    let value = serde_json::to_value(value)?;
    Ok(hex::encode(Sha256::digest(canonical(&value).as_bytes())))
}
pub fn canonical(value: &Value) -> String {
    match value {
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(canonical).collect::<Vec<_>>().join(",")
        ),
        Value::Object(map) => {
            let mut items: Vec<_> = map.iter().collect();
            items.sort_by(|a, b| a.0.cmp(b.0));
            format!(
                "{{{}}}",
                items
                    .into_iter()
                    .map(|(k, v)| format!("{}:{}", Value::String(k.clone()), canonical(v)))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        other => other.to_string(),
    }
}
pub fn token() -> String {
    let mut bytes = [0; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}
pub fn shuffle<T>(cards: &mut [T]) {
    cards.shuffle(&mut OsRng);
}
