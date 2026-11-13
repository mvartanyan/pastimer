use crate::{
    error::{Error, Result, ensure},
    model::{Card, Player, standard_deck},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub struct PublicEvent {
    pub kind: &'static str,
    pub data: Value,
}
/// Mutually exclusive outcomes prevent finishing and requesting a shuffle together.
pub enum Outcome<S> {
    Continue {
        state: S,
        events: Vec<PublicEvent>,
    },
    Shuffle {
        state: S,
        cards: Vec<Card>,
        events: Vec<PublicEvent>,
    },
    Finished {
        state: S,
        scores: BTreeMap<String, i64>,
        events: Vec<PublicEvent>,
    },
}
impl<S> Outcome<S> {
    pub fn map<T>(self, f: impl FnOnce(S) -> T) -> Outcome<T> {
        match self {
            Self::Continue { state, events } => Outcome::Continue {
                state: f(state),
                events,
            },
            Self::Shuffle {
                state,
                cards,
                events,
            } => Outcome::Shuffle {
                state: f(state),
                cards,
                events,
            },
            Self::Finished {
                state,
                scores,
                events,
            } => Outcome::Finished {
                state: f(state),
                scores,
                events,
            },
        }
    }
}
pub trait GameRules {
    type State: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync;
    type Action;
    type View: Serialize;
    fn initialize(players: &[Player]) -> Outcome<Self::State>;
    fn on_deck(
        state: Self::State,
        cards: Vec<Card>,
        players: &[Player],
    ) -> Result<Outcome<Self::State>>;
    fn apply(
        state: Self::State,
        actor: &str,
        action: Self::Action,
        players: &[Player],
    ) -> Result<Outcome<Self::State>>;
    fn view(state: &Self::State, viewer: &str, players: &[Player]) -> Self::View;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighCardState {
    hands: BTreeMap<String, Vec<Card>>,
    played: BTreeMap<String, Card>,
    scores: BTreeMap<String, i64>,
    round: u8,
}
#[derive(Serialize)]
pub struct Seat {
    #[serde(flatten)]
    player: Player,
    count: usize,
    played: bool,
    score: i64,
}
#[derive(Serialize)]
pub struct ActionView {
    #[serde(rename = "type")]
    kind: &'static str,
    label: &'static str,
    input: &'static str,
}
#[derive(Serialize)]
pub struct HighCardView {
    round: u8,
    hand: Vec<Card>,
    seats: Vec<Seat>,
    actions: Vec<ActionView>,
}
pub struct HighCard;
impl GameRules for HighCard {
    type State = HighCardState;
    type Action = Card;
    type View = HighCardView;
    fn initialize(players: &[Player]) -> Outcome<Self::State> {
        Outcome::Shuffle {
            state: HighCardState {
                hands: BTreeMap::new(),
                played: BTreeMap::new(),
                scores: players.iter().map(|p| (p.id.clone(), 0)).collect(),
                round: 1,
            },
            cards: standard_deck(),
            events: vec![],
        }
    }
    fn on_deck(
        mut state: Self::State,
        cards: Vec<Card>,
        players: &[Player],
    ) -> Result<Outcome<Self::State>> {
        ensure(cards.len() >= players.len() * 5, Error::internal())?;
        state.hands = players.iter().map(|p| (p.id.clone(), vec![])).collect();
        for (i, card) in cards.into_iter().take(players.len() * 5).enumerate() {
            let player = players.get(i % players.len()).ok_or_else(Error::internal)?;
            state
                .hands
                .get_mut(&player.id)
                .ok_or_else(Error::internal)?
                .push(card);
        }
        Ok(Outcome::Continue {
            state,
            events: vec![],
        })
    }
    fn apply(
        mut state: Self::State,
        actor: &str,
        card: Card,
        players: &[Player],
    ) -> Result<Outcome<Self::State>> {
        ensure(
            !state.played.contains_key(actor),
            Error::bad("Already played this round"),
        )?;
        let hand = state
            .hands
            .get_mut(actor)
            .ok_or_else(|| Error::bad("Card is not in your hand"))?;
        let index = hand
            .iter()
            .position(|c| *c == card)
            .ok_or_else(|| Error::bad("Card is not in your hand"))?;
        hand.remove(index);
        state.played.insert(actor.into(), card);
        if state.played.len() != players.len() {
            return Ok(Outcome::Continue {
                state,
                events: vec![],
            });
        }
        let highest = state
            .played
            .values()
            .map(Card::rank)
            .max()
            .ok_or_else(Error::internal)?;
        let winners: Vec<_> = players
            .iter()
            .filter(|p| state.played.get(&p.id).is_some_and(|c| c.rank() == highest))
            .map(|p| p.id.clone())
            .collect();
        for id in &winners {
            *state.scores.get_mut(id).ok_or_else(Error::internal)? += 1;
        }
        let events = vec![PublicEvent {
            kind: "round.revealed",
            data: json!({"round":state.round,"cards":state.played,"winners":winners}),
        }];
        let finished = state.round == 5;
        state.round += 1;
        state.played.clear();
        if finished {
            Ok(Outcome::Finished {
                scores: state.scores.clone(),
                state,
                events,
            })
        } else {
            Ok(Outcome::Continue { state, events })
        }
    }
    fn view(state: &Self::State, viewer: &str, players: &[Player]) -> Self::View {
        let hand = state.hands.get(viewer).cloned().unwrap_or_default();
        let can_play = !hand.is_empty() && !state.played.contains_key(viewer);
        HighCardView {
            round: state.round,
            hand,
            seats: players
                .iter()
                .map(|p| Seat {
                    player: p.clone(),
                    count: state.hands.get(&p.id).map_or(0, Vec::len),
                    played: state.played.contains_key(&p.id),
                    score: state.scores.get(&p.id).copied().unwrap_or(0),
                })
                .collect(),
            actions: if can_play {
                vec![ActionView {
                    kind: "play",
                    label: "Play selected card",
                    input: "card",
                }]
            } else {
                vec![]
            },
        }
    }
}
