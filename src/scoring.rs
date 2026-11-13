//! Shared cumulative scores and deal checkpoints. Game rules decide which awards apply.
use crate::{
    error::{Error, Result},
    model::Player,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ledger {
    pub totals: BTreeMap<String, i64>,
}
impl Ledger {
    pub fn new(players: &[Player]) -> Self {
        Self {
            totals: players.iter().map(|p| (p.id.clone(), 0)).collect(),
        }
    }
    pub fn get(&self, id: &str) -> i64 {
        self.totals.get(id).copied().unwrap_or(0)
    }
    pub fn award(&mut self, id: &str, delta: i64, ceiling: Option<i64>) -> Result<()> {
        let score = self.totals.get_mut(id).ok_or_else(Error::internal)?;
        *score = score.checked_add(delta).ok_or_else(Error::internal)?;
        if delta > 0
            && let Some(cap) = ceiling
        {
            *score = (*score).min(cap);
        }
        Ok(())
    }
}
