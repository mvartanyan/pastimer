use crate::{
    error::{Error, Result},
    game::HighCardState,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Card(String);
impl Card {
    pub fn code(&self) -> &str {
        &self.0
    }
    pub fn face(&self) -> &str {
        &self.0[..self.0.len() - 1]
    }
    pub fn rank(&self) -> usize {
        let rank = &self.0[..self.0.len() - 1];
        [
            "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A",
        ]
        .iter()
        .position(|r| *r == rank)
        .unwrap_or(0)
    }
}
impl FromStr for Card {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self> {
        let valid = ["C", "D", "H", "S"].iter().any(|s| {
            value.strip_suffix(s).is_some_and(|r| {
                [
                    "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A",
                ]
                .contains(&r)
            })
        });
        if valid {
            Ok(Self(value.into()))
        } else {
            Err(Error::bad("Invalid card"))
        }
    }
}
impl TryFrom<String> for Card {
    type Error = Error;
    fn try_from(s: String) -> Result<Self> {
        s.parse()
    }
}
impl From<Card> for String {
    fn from(card: Card) -> String {
        card.0
    }
}
pub fn standard_deck() -> Vec<Card> {
    ["C", "D", "H", "S"]
        .into_iter()
        .flat_map(|s| {
            [
                "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A",
            ]
            .map(|r| Card(format!("{r}{s}")))
        })
        .collect()
}
pub fn thousand_deck() -> Vec<Card> {
    standard_deck()
        .into_iter()
        .filter(|c| c.rank() >= 7)
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Player {
    pub id: String,
    pub name: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameId {
    #[serde(rename = "high-card-demo")]
    HighCard,
    #[serde(rename = "thousand")]
    Thousand,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Action {
    Thousand {
        action: crate::thousand::Action,
    },
    Start,
    Acknowledge {
        #[serde(rename = "shuffleId")]
        shuffle_id: u32,
        commitment: String,
    },
    Play {
        card: Card,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub version: u64,
    pub request_id: String,
    pub action: Action,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "game", content = "data", rename_all = "kebab-case")]
pub enum GameState {
    HighCard(HighCardState),
    Thousand(Box<crate::thousand::State>),
}
impl<'de> Deserialize<'de> for GameState {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "game", content = "data", rename_all = "kebab-case")]
        enum Tagged {
            HighCard(HighCardState),
            Thousand(Box<crate::thousand::State>),
        }
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Stored {
            Tagged(Tagged),
            Legacy(HighCardState),
        }
        Ok(match Stored::deserialize(deserializer)? {
            Stored::Tagged(Tagged::HighCard(s)) | Stored::Legacy(s) => Self::HighCard(s),
            Stored::Tagged(Tagged::Thousand(s)) => Self::Thousand(s),
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "phase", content = "state", rename_all = "lowercase")]
pub enum Lifecycle {
    Lobby,
    Committing(GameState),
    Playing(GameState),
    Finished(GameState),
}
impl Lifecycle {
    pub fn phase(&self) -> &'static str {
        match self {
            Self::Lobby => "lobby",
            Self::Committing(_) => "committing",
            Self::Playing(_) => "playing",
            Self::Finished(_) => "finished",
        }
    }
    pub fn state(&self) -> Option<&GameState> {
        match self {
            Self::Lobby => None,
            Self::Committing(s) | Self::Playing(s) | Self::Finished(s) => Some(s),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckProof {
    pub protocol: String,
    pub room_id: String,
    pub shuffle_id: u32,
    pub cards: Vec<Card>,
    pub salt: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub proof: DeckProof,
    pub commitment: String,
    pub acknowledgements: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveProof {
    #[serde(default, rename = "dealId", skip_serializing_if = "Option::is_none")]
    pub deal_id: Option<u32>,
    pub actor: String,
    pub version: u64,
    pub action: Action,
    pub salt: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub seq: u64,
    pub room_id: String,
    pub at: String,
    pub actor: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub data: Value,
    pub previous_hash: String,
    pub hash: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    #[serde(default)]
    pub options: Option<crate::thousand::Settings>,
    pub id: String,
    pub code: String,
    pub host: String,
    pub game: GameId,
    pub game_version: u32,
    pub players: Vec<Player>,
    pub version: u64,
    #[serde(flatten)]
    pub lifecycle: Lifecycle,
    pub scores: BTreeMap<String, i64>,
    pub decks: Vec<Deck>,
    pub commands: Vec<MoveProof>,
    pub events: Vec<Event>,
}

/// Explicit public projection. A private GameState cannot be used as the table view.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomView {
    pub options: Option<crate::thousand::Settings>,
    pub id: String,
    pub code: String,
    pub host: String,
    pub game: GameId,
    pub game_version: u32,
    pub version: u64,
    pub phase: &'static str,
    pub players: Vec<Player>,
    pub scores: BTreeMap<String, i64>,
    pub latest_event_hash: Option<String>,
    pub deck: Option<DeckView>,
    pub table: Option<TableView>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckView {
    pub shuffle_id: u32,
    pub commitment: String,
    pub count: usize,
    pub acknowledgements: Vec<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum TableView {
    HighCard(crate::game::HighCardView),
    Thousand(Box<crate::thousand::View>),
}
