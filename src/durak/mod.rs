pub mod verify;

use crate::{
    error::{Error, Result, ensure},
    game::{Outcome, PublicEvent},
    model::{Card, Player, standard_deck},
    scoring::Ledger,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Variant {
    Podkidnoy,
    Perevodnoy,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub player_count: u8,
}
impl Default for Settings {
    fn default() -> Self {
        Self { player_count: 2 }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        ensure(
            (2..=6).contains(&self.player_count),
            Error::bad("Choose 2 to 6 players"),
        )
    }
}
pub fn deck() -> Vec<Card> {
    standard_deck()
        .into_iter()
        .filter(|c| c.rank() >= 4)
        .collect()
}
pub fn suit(card: &Card) -> &str {
    &card.code()[card.code().len() - 1..]
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Action {
    Attack { cards: Vec<Card> },
    Defend { attack: Card, card: Card },
    Transfer { card: Card },
    Take,
    Request { rank: String },
    Approve { player: String, rank: String },
    Pass,
    Rematch,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub card: Card,
    pub player: String,
    pub cover: Option<Card>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Permission {
    pub player: String,
    pub rank: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Movement {
    pub cards: Vec<Card>,
    pub from: String,
    pub to: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Bout {
    pub number: u32,
    pub original_attacker: String,
    pub attacker: String,
    pub defender: String,
    pub taken: bool,
    pub pairs: Vec<Pair>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub deal: u32,
    pub dealer: usize,
    pub first_attacker: String,
    pub loser: Option<String>,
    pub hands: BTreeMap<String, Vec<Card>>,
    pub discard: Vec<Card>,
    pub movements: Vec<Movement>,
    pub bouts: Vec<Bout>,
    pub scores: BTreeMap<String, i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub settings: Settings,
    pub variant: Variant,
    pub dealer: usize,
    pub deal: u32,
    pub hands: BTreeMap<String, Vec<Card>>,
    pub stock: Vec<Card>,
    pub trump: Option<Card>,
    pub discard: Vec<Card>,
    pub pairs: Vec<Pair>,
    pub original_attacker: String,
    pub attacker: String,
    pub defender: String,
    pub first_attacker: String,
    pub limit: usize,
    pub taking: bool,
    pub requests: BTreeSet<Permission>,
    pub approvals: BTreeSet<Permission>,
    pub passed: BTreeSet<String>,
    pub finished_players: BTreeSet<String>,
    pub finished: bool,
    pub loser: Option<String>,
    pub ledger: Ledger,
    pub archives: Vec<Record>,
    pub movements: Vec<Movement>,
    pub bouts: Vec<Bout>,
}
impl State {
    pub fn new(
        players: &[Player],
        settings: Settings,
        variant: Variant,
        dealer: usize,
    ) -> Result<Self> {
        settings.validate()?;
        ensure(
            players.len() == usize::from(settings.player_count)
                && dealer < players.len()
                && players.iter().map(|p| &p.id).collect::<BTreeSet<_>>().len() == players.len(),
            Error::bad("Fill all table seats"),
        )?;
        Ok(Self {
            settings,
            variant,
            dealer,
            deal: 0,
            hands: BTreeMap::new(),
            stock: vec![],
            trump: None,
            discard: vec![],
            pairs: vec![],
            original_attacker: String::new(),
            attacker: String::new(),
            defender: String::new(),
            first_attacker: String::new(),
            limit: 6,
            taking: false,
            requests: BTreeSet::new(),
            approvals: BTreeSet::new(),
            passed: BTreeSet::new(),
            finished_players: BTreeSet::new(),
            finished: false,
            loser: None,
            ledger: Ledger::new(players),
            archives: vec![],
            movements: vec![],
            bouts: vec![],
        })
    }
    pub fn shuffle(self) -> Outcome<Self> {
        Outcome::Shuffle {
            state: self,
            cards: deck(),
            events: vec![],
        }
    }
    fn event(kind: &'static str, data: Value) -> PublicEvent {
        PublicEvent { kind, data }
    }
    fn move_cards(&mut self, cards: Vec<Card>, from: &str, to: &str) {
        if !cards.is_empty() {
            self.movements.push(Movement {
                cards,
                from: from.into(),
                to: to.into(),
            });
        }
    }
    fn hand(&self, id: &str) -> &[Card] {
        self.hands.get(id).map_or(&[], Vec::as_slice)
    }
    fn next(&self, id: &str, players: &[Player]) -> Result<String> {
        let index = players
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(Error::internal)?;
        (1..=players.len())
            .map(|n| &players[(index + n) % players.len()])
            .find(|p| !self.finished_players.contains(&p.id))
            .map(|p| p.id.clone())
            .ok_or_else(Error::internal)
    }
    fn reset_priority(&mut self) {
        self.passed.clear();
    }
    fn new_bout(&mut self, attacker: String, players: &[Player]) -> Result<()> {
        self.attacker = attacker.clone();
        self.original_attacker = attacker;
        self.defender = self.next(&self.attacker, players)?;
        self.limit = self.hand(&self.defender).len().min(6);
        self.taking = false;
        self.pairs.clear();
        self.requests.clear();
        self.approvals.clear();
        self.reset_priority();
        Ok(())
    }
    pub fn deal_cards(mut self, cards: Vec<Card>, players: &[Player]) -> Result<Outcome<Self>> {
        ensure(
            cards.iter().cloned().collect::<BTreeSet<_>>() == deck().into_iter().collect()
                && cards.len() == 36,
            Error::bad("Invalid Durak deck"),
        )?;
        let previous_loser = self.loser.clone();
        self.deal += 1;
        self.finished = false;
        self.loser = None;
        self.hands = players.iter().map(|p| (p.id.clone(), vec![])).collect();
        self.stock.clear();
        self.discard.clear();
        self.movements.clear();
        self.bouts.clear();
        self.finished_players.clear();
        for (n, card) in cards.iter().take(players.len() * 6).enumerate() {
            let id = &players[(self.dealer + 1 + n) % players.len()].id;
            self.hands
                .get_mut(id)
                .ok_or_else(Error::internal)?
                .push(card.clone());
            self.move_cards(vec![card.clone()], "deck", id);
        }
        let dealt = players.len() * 6;
        self.trump = Some(cards[if dealt == 36 { 35 } else { dealt }].clone());
        if dealt < 36 {
            self.stock.extend_from_slice(&cards[dealt + 1..]);
            self.stock.push(cards[dealt].clone());
            self.move_cards(self.stock.clone(), "deck", "stock");
        }
        let trump = self.trump.as_ref().ok_or_else(Error::internal)?;
        let first = if let Some(loser) = previous_loser {
            let i = players
                .iter()
                .position(|p| p.id == loser)
                .ok_or_else(Error::internal)?;
            players[(i + players.len() - 1) % players.len()].id.clone()
        } else {
            players
                .iter()
                .flat_map(|p| {
                    self.hand(&p.id)
                        .iter()
                        .filter(|c| suit(c) == suit(trump))
                        .map(move |c| (c.rank(), p.id.clone()))
                })
                .min()
                .map(|(_, id)| id)
                .unwrap_or_else(|| players[(self.dealer + 1) % players.len()].id.clone())
        };
        self.first_attacker = first.clone();
        self.new_bout(first, players)?;
        let events = vec![Self::event(
            "durak.dealt",
            json!({"deal":self.deal,"dealer":players[self.dealer].id,"trump":self.trump,"attacker":self.attacker,"defender":self.defender,"stockCount":self.stock.len()}),
        )];
        Ok(Outcome::Continue {
            state: self,
            events,
        })
    }
    fn remove(&mut self, actor: &str, card: &Card) -> Result<()> {
        let hand = self
            .hands
            .get_mut(actor)
            .ok_or_else(|| Error::bad("Not a player"))?;
        let i = hand
            .iter()
            .position(|c| c == card)
            .ok_or_else(|| Error::bad("Card is not in your hand"))?;
        hand.remove(i);
        Ok(())
    }
    fn ranks(&self) -> BTreeSet<String> {
        self.pairs
            .iter()
            .flat_map(|p| {
                std::iter::once(p.card.face().to_owned())
                    .chain(p.cover.iter().map(|c| c.face().to_owned()))
            })
            .collect()
    }
    fn can_beat(&self, card: &Card, attack: &Card) -> bool {
        (suit(card) == suit(attack) && card.rank() > attack.rank())
            || (self
                .trump
                .as_ref()
                .is_some_and(|t| suit(card) == suit(t) && suit(attack) != suit(t)))
    }
    pub fn turn(&self, players: &[Player]) -> Option<String> {
        if self.finished {
            return None;
        }
        if !self.passed.contains(&self.attacker) {
            return Some(self.attacker.clone());
        }
        let start = players.iter().position(|p| p.id == self.attacker)?;
        (1..players.len())
            .map(|i| &players[(start + i) % players.len()])
            .find(|p| {
                p.id != self.defender
                    && !self.passed.contains(&p.id)
                    && self.approvals.iter().any(|a| a.player == p.id)
                    && !self.hand(&p.id).is_empty()
            })
            .map(|p| p.id.clone())
    }
    fn attack_cards(&self, actor: &str, players: &[Player]) -> Vec<Card> {
        if self.turn(players).as_deref() != Some(actor) || self.pairs.len() >= self.limit {
            return vec![];
        }
        let ranks = self.ranks();
        self.hand(actor)
            .iter()
            .filter(|c| {
                self.pairs.is_empty()
                    || (ranks.contains(c.face())
                        && (actor == self.attacker
                            || self.approvals.contains(&Permission {
                                player: actor.into(),
                                rank: c.face().into(),
                            })))
            })
            .cloned()
            .collect()
    }
    fn collect(
        mut self,
        players: &[Player],
        mut events: Vec<PublicEvent>,
    ) -> Result<Outcome<Self>> {
        if self.pairs.is_empty()
            || (!self.taking && self.pairs.iter().any(|p| p.cover.is_none()))
            || self.turn(players).is_some()
        {
            return Ok(Outcome::Continue {
                state: self,
                events,
            });
        }
        let cards: Vec<Card> = self
            .pairs
            .iter()
            .flat_map(|p| std::iter::once(p.card.clone()).chain(p.cover.clone()))
            .collect();
        let record = Bout {
            number: self.bouts.len() as u32 + 1,
            original_attacker: self.original_attacker.clone(),
            attacker: self.attacker.clone(),
            defender: self.defender.clone(),
            taken: self.taking,
            pairs: self.pairs.clone(),
        };
        self.bouts.push(record.clone());
        let destination = if self.taking {
            self.defender.clone()
        } else {
            "discard".into()
        };
        if self.taking {
            self.hands
                .get_mut(&self.defender)
                .ok_or_else(Error::internal)?
                .extend(cards.clone());
        } else {
            self.discard.extend(cards.clone());
        }
        self.move_cards(cards, "table", &destination);
        let index = players
            .iter()
            .position(|p| p.id == self.original_attacker)
            .ok_or_else(Error::internal)?;
        let mut drawing: Vec<String> = (0..players.len())
            .map(|i| players[(index + i) % players.len()].id.clone())
            .filter(|id| *id != self.defender)
            .collect();
        drawing.push(self.defender.clone());
        for id in &drawing {
            let need = 6usize
                .saturating_sub(self.hand(id).len())
                .min(self.stock.len());
            let drawn: Vec<Card> = self.stock.drain(..need).collect();
            self.hands
                .get_mut(id)
                .ok_or_else(Error::internal)?
                .extend(drawn.clone());
            self.move_cards(drawn, "stock", id);
        }
        events.push(Self::event("durak.boutEnded", json!({"deal":self.deal,"bout":record,"drawingOrder":drawing,"stockCount":self.stock.len()})));
        if self.stock.is_empty() {
            self.finished_players = players
                .iter()
                .filter(|p| self.hand(&p.id).is_empty())
                .map(|p| p.id.clone())
                .collect();
        }
        let remaining: Vec<_> = players
            .iter()
            .filter(|p| !self.finished_players.contains(&p.id))
            .collect();
        if remaining.len() <= 1 {
            self.finished = true;
            self.loser = remaining.first().map(|p| p.id.clone());
            if let Some(id) = &self.loser {
                self.ledger.award(id, -1, None)?;
            }
            let archive = Record {
                deal: self.deal,
                dealer: self.dealer,
                first_attacker: self.first_attacker.clone(),
                loser: self.loser.clone(),
                hands: self.hands.clone(),
                discard: self.discard.clone(),
                movements: self.movements.clone(),
                bouts: self.bouts.clone(),
                scores: self.ledger.totals.clone(),
            };
            events.push(Self::event("durak.settled",json!({"deal":self.deal,"loser":self.loser,"recordHash":crate::crypto::hash(&archive)?})));
            self.archives.push(archive);
            return Ok(Outcome::Finished {
                scores: self.ledger.totals.clone(),
                state: self,
                events,
            });
        }
        let next = if self.taking || self.finished_players.contains(&self.defender) {
            self.next(&self.defender, players)?
        } else {
            self.defender.clone()
        };
        self.new_bout(next, players)?;
        Ok(Outcome::Continue {
            state: self,
            events,
        })
    }
    pub fn apply(
        mut self,
        actor: &str,
        action: Action,
        players: &[Player],
    ) -> Result<Outcome<Self>> {
        ensure(
            players.iter().any(|p| p.id == actor),
            Error::bad("Not a player"),
        )?;
        if action == Action::Rematch {
            ensure(self.finished, Error::bad("Finish this game first"))?;
            self.dealer = self
                .loser
                .as_ref()
                .and_then(|id| players.iter().position(|p| p.id == *id))
                .unwrap_or((self.dealer + 1) % players.len());
            return Ok(self.shuffle());
        }
        ensure(
            !self.finished && !self.finished_players.contains(actor),
            Error::bad("Game finished"),
        )?;
        let mut events = vec![];
        match action {
            Action::Attack { cards } => {
                ensure(
                    !cards.is_empty()
                        && cards.len() + self.pairs.len() <= self.limit
                        && cards.iter().collect::<BTreeSet<_>>().len() == cards.len(),
                    Error::bad("Attack limit exceeded"),
                )?;
                let legal = self.attack_cards(actor, players);
                ensure(
                    cards.iter().all(|c| legal.contains(c)),
                    Error::bad("Not your turn or rank not approved"),
                )?;
                ensure(
                    !self.pairs.is_empty() || cards.iter().all(|c| c.face() == cards[0].face()),
                    Error::bad("Opening cards must have equal ranks"),
                )?;
                for card in &cards {
                    self.remove(actor, card)?;
                    self.pairs.push(Pair {
                        card: card.clone(),
                        player: actor.into(),
                        cover: None,
                    });
                }
                self.move_cards(cards.clone(), actor, "table");
                events.push(Self::event(
                    "durak.attacked",
                    json!({"deal":self.deal,"player":actor,"cards":cards}),
                ));
            }
            Action::Defend { attack, card } => {
                ensure(
                    actor == self.defender && !self.taking,
                    Error::bad("Not your turn"),
                )?;
                let i = self
                    .pairs
                    .iter()
                    .position(|p| p.card == attack && p.cover.is_none())
                    .ok_or_else(|| Error::bad("Choose an uncovered card"))?;
                ensure(
                    self.can_beat(&card, &attack),
                    Error::bad("Card cannot beat this attack"),
                )?;
                let new_rank = !self.ranks().contains(card.face());
                self.remove(actor, &card)?;
                self.pairs[i].cover = Some(card.clone());
                self.move_cards(vec![card.clone()], actor, "table");
                if new_rank {
                    self.reset_priority();
                }
                events.push(Self::event(
                    "durak.defended",
                    json!({"deal":self.deal,"player":actor,"attack":attack,"card":card}),
                ));
            }
            Action::Transfer { card } => {
                ensure(
                    self.variant == Variant::Perevodnoy
                        && actor == self.defender
                        && !self.taking
                        && !self.pairs.is_empty()
                        && self
                            .pairs
                            .iter()
                            .all(|p| p.cover.is_none() && p.card.face() == card.face()),
                    Error::bad("Cannot transfer this attack"),
                )?;
                let defender = self.next(actor, players)?;
                ensure(
                    self.pairs.len() < self.hand(&defender).len().min(6),
                    Error::bad("Next defender has too few cards"),
                )?;
                self.remove(actor, &card)?;
                self.pairs.push(Pair {
                    card: card.clone(),
                    player: actor.into(),
                    cover: None,
                });
                self.move_cards(vec![card.clone()], actor, "table");
                self.attacker = actor.into();
                self.defender = defender;
                self.limit = self.hand(&self.defender).len().min(6);
                self.requests.clear();
                self.approvals.clear();
                self.reset_priority();
                events.push(Self::event(
                    "durak.transferred",
                    json!({"deal":self.deal,"player":actor,"card":card,"defender":self.defender}),
                ));
            }
            Action::Take => {
                ensure(
                    actor == self.defender && !self.taking && !self.pairs.is_empty(),
                    Error::bad("Not your turn"),
                )?;
                self.taking = true;
                self.reset_priority();
                events.push(Self::event(
                    "durak.taking",
                    json!({"deal":self.deal,"player":actor}),
                ));
            }
            Action::Request { rank } => {
                ensure(
                    actor != self.attacker
                        && actor != self.defender
                        && self.pairs.len() < self.limit
                        && self.ranks().contains(&rank)
                        && self.hand(actor).iter().any(|c| c.face() == rank),
                    Error::bad("Cannot request this rank"),
                )?;
                let permission = Permission {
                    player: actor.into(),
                    rank,
                };
                ensure(
                    !self.approvals.contains(&permission) && self.requests.insert(permission),
                    Error::bad("Already requested or approved"),
                )?;
                events.push(Self::event(
                    "durak.requested",
                    json!({"deal":self.deal,"player":actor}),
                ));
            }
            Action::Approve { player, rank } => {
                ensure(
                    actor == self.attacker,
                    Error::bad("Only the current attacker can approve"),
                )?;
                let permission = Permission { player, rank };
                ensure(
                    self.requests.remove(&permission),
                    Error::bad("Request is no longer pending"),
                )?;
                self.passed.remove(&permission.player);
                self.approvals.insert(permission.clone());
                events.push(Self::event(
                    "durak.approved",
                    json!({"deal":self.deal,"permission":permission}),
                ));
            }
            Action::Pass => {
                ensure(
                    !self.pairs.is_empty() && self.turn(players).as_deref() == Some(actor),
                    Error::bad("Not your turn"),
                )?;
                self.passed.insert(actor.into());
                events.push(Self::event(
                    "durak.passed",
                    json!({"deal":self.deal,"player":actor}),
                ));
            }
            Action::Rematch => return Err(Error::internal()),
        }
        if self.pairs.len() == self.limit {
            self.passed.extend(players.iter().map(|p| p.id.clone()));
        }
        self.collect(players, events)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Defence {
    pub attack: Card,
    pub cards: Vec<Card>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Controls {
    pub attack_cards: Vec<Card>,
    pub defences: Vec<Defence>,
    pub transfer_cards: Vec<Card>,
    pub request_ranks: Vec<String>,
    pub can_take: bool,
    pub can_pass: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Seat {
    pub id: String,
    pub name: String,
    pub count: usize,
    pub finished: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub kind: String,
    pub variant: Variant,
    pub deal: u32,
    pub phase: String,
    pub hand: Vec<Card>,
    pub seats: Vec<Seat>,
    pub trump: Option<Card>,
    pub stock_count: usize,
    pub discard_count: usize,
    pub pairs: Vec<Pair>,
    pub original_attacker: String,
    pub attacker: String,
    pub defender: String,
    pub turn: Option<String>,
    pub taking: bool,
    pub limit: usize,
    pub requests: Vec<Permission>,
    pub approvals: Vec<Permission>,
    pub loser: Option<String>,
    pub controls: Controls,
}
impl State {
    pub fn view(&self, viewer: &str, players: &[Player]) -> View {
        let live = !self.finished && !self.finished_players.contains(viewer);
        let defences = if live && viewer == self.defender && !self.taking {
            self.pairs
                .iter()
                .filter(|p| p.cover.is_none())
                .map(|p| Defence {
                    attack: p.card.clone(),
                    cards: self
                        .hand(viewer)
                        .iter()
                        .filter(|c| self.can_beat(c, &p.card))
                        .cloned()
                        .collect(),
                })
                .collect()
        } else {
            vec![]
        };
        let transfer_cards = if live
            && self.variant == Variant::Perevodnoy
            && viewer == self.defender
            && !self.taking
            && !self.pairs.is_empty()
            && self.pairs.iter().all(|p| p.cover.is_none())
            && self
                .next(viewer, players)
                .is_ok_and(|id| self.pairs.len() < self.hand(&id).len().min(6))
        {
            self.hand(viewer)
                .iter()
                .filter(|c| self.pairs.iter().all(|p| p.card.face() == c.face()))
                .cloned()
                .collect()
        } else {
            vec![]
        };
        let request_ranks = if live
            && viewer != self.attacker
            && viewer != self.defender
            && self.pairs.len() < self.limit
        {
            self.ranks()
                .into_iter()
                .filter(|r| {
                    self.hand(viewer).iter().any(|c| c.face() == r)
                        && !self.approvals.contains(&Permission {
                            player: viewer.into(),
                            rank: r.clone(),
                        })
                        && !self.requests.contains(&Permission {
                            player: viewer.into(),
                            rank: r.clone(),
                        })
                })
                .collect()
        } else {
            vec![]
        };
        View {
            kind: "durak".into(),
            variant: self.variant,
            deal: self.deal,
            phase: if self.finished { "finished" } else { "attack" }.into(),
            hand: self.hand(viewer).to_vec(),
            seats: players
                .iter()
                .map(|p| Seat {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    count: self.hand(&p.id).len(),
                    finished: self.finished_players.contains(&p.id),
                })
                .collect(),
            trump: self.trump.clone(),
            stock_count: self.stock.len(),
            discard_count: self.discard.len(),
            pairs: self.pairs.clone(),
            original_attacker: self.original_attacker.clone(),
            attacker: self.attacker.clone(),
            defender: self.defender.clone(),
            turn: self.turn(players),
            taking: self.taking,
            limit: self.limit,
            requests: self
                .requests
                .iter()
                .filter(|r| viewer == self.attacker || r.player == viewer)
                .cloned()
                .collect(),
            approvals: self.approvals.iter().cloned().collect(),
            loser: self.loser.clone(),
            controls: Controls {
                attack_cards: if live {
                    self.attack_cards(viewer, players)
                } else {
                    vec![]
                },
                defences,
                transfer_cards,
                request_ranks,
                can_take: live && viewer == self.defender && !self.taking && !self.pairs.is_empty(),
                can_pass: live
                    && !self.pairs.is_empty()
                    && self.turn(players).as_deref() == Some(viewer),
            },
        }
    }
}
/// Computer decisions use the same private projection as a human, never the state.
pub fn choose(view: View) -> Option<Action> {
    if let Some(card) = view.controls.attack_cards.first() {
        return Some(Action::Attack {
            cards: vec![card.clone()],
        });
    }
    if let Some(card) = view.controls.transfer_cards.first() {
        return Some(Action::Transfer { card: card.clone() });
    }
    for defence in &view.controls.defences {
        if let Some(card) = defence.cards.first() {
            return Some(Action::Defend {
                attack: defence.attack.clone(),
                card: card.clone(),
            });
        }
    }
    if view.controls.can_take {
        return Some(Action::Take);
    }
    if view.controls.can_pass
        && view.attacker == view.turn.as_deref().unwrap_or("")
        && let Some(r) = view.requests.first()
    {
        return Some(Action::Approve {
            player: r.player.clone(),
            rank: r.rank.clone(),
        });
    }
    if view.controls.can_pass {
        return Some(Action::Pass);
    }
    view.controls
        .request_ranks
        .first()
        .map(|rank| Action::Request { rank: rank.clone() })
}

#[cfg(test)]
mod tests;
