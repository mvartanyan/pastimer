use crate::{
    error::{Error, Result, ensure},
    game::{Outcome, PublicEvent},
    model::{Card, Player},
    scoring::Ledger,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
pub mod verify;
pub const MAX_ORDER: u16 = 380;
pub const TRICK_DELAY_MS: i64 = 3000;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub player_count: u8,
    pub barrel_threshold: u16,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            player_count: 3,
            barrel_threshold: 900,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        ensure(
            matches!(self.player_count, 3 | 4)
                && (620..=900).contains(&self.barrel_threshold)
                && self.barrel_threshold.is_multiple_of(5),
            Error::bad("Choose 3 or 4 players and a barrel from 620 to 900 in steps of 5"),
        )
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suit {
    C,
    D,
    H,
    S,
}
impl Suit {
    pub fn marriage(self) -> u16 {
        match self {
            Self::H => 100,
            Self::D => 80,
            Self::C => 60,
            Self::S => 40,
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::D => "D",
            Self::H => "H",
            Self::S => "S",
        }
    }
}
pub fn suit(card: &Card) -> Suit {
    match card.code().as_bytes().last() {
        Some(b'C') => Suit::C,
        Some(b'D') => Suit::D,
        Some(b'H') => Suit::H,
        _ => Suit::S,
    }
}
pub fn rank(card: &Card) -> u8 {
    match card.face() {
        "A" => 5,
        "10" => 4,
        "K" => 3,
        "Q" => 2,
        "J" => 1,
        _ => 0,
    }
}
pub fn points(card: &Card) -> u16 {
    match card.face() {
        "A" | "10" => 10,
        "K" | "Q" | "J" => 5,
        _ => 0,
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Action {
    Shuffle,
    Bid {
        amount: u16,
    },
    Pass,
    Here,
    TakeStock,
    Distribute {
        transfers: Vec<Transfer>,
    },
    Order {
        amount: u16,
    },
    Dismiss,
    FourNines,
    Play {
        card: Card,
        #[serde(default)]
        announce: bool,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transfer {
    pub player: String,
    pub card: Card,
    pub public: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auction {
    eligible: Vec<usize>,
    leader: usize,
    turn: usize,
    last_pass: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Phase {
    AwaitShuffle,
    Auction(Auction),
    Stock,
    Distribution,
    Preparation,
    Trick {
        turn: usize,
        plays: Vec<Play>,
    },
    Closing {
        trick: Trick,
        #[serde(rename = "closeAt")]
        close_at: i64,
    },
    Finished,
}
impl Phase {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AwaitShuffle => "awaitShuffle",
            Self::Auction(_) => "auction",
            Self::Stock => "stock",
            Self::Distribution => "distribution",
            Self::Preparation => "preparation",
            Self::Trick { .. } => "trick",
            Self::Closing { .. } => "closing",
            Self::Finished => "finished",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Play {
    pub player: String,
    pub card: Card,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Trick {
    pub number: u8,
    pub plays: Vec<Play>,
    pub trump: Option<Suit>,
    pub winner: String,
    pub points: u16,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Zone {
    Deck,
    Stock,
    Hand { player: String },
    Table { trick: u8 },
    Pile { player: String, trick: u8 },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movement {
    pub card: Card,
    pub from: Zone,
    pub to: Zone,
    pub actor: String,
    pub public: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marriage {
    pub player: String,
    pub suit: Suit,
    pub trick: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub player: String,
    pub order: u16,
    pub compulsory: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DealRecord {
    pub deal: u32,
    pub dealer: String,
    pub active: Vec<String>,
    pub barrel_players: Vec<String>,
    pub status: String,
    pub contract: Option<Contract>,
    pub before: Ledger,
    pub after: Ledger,
    pub sticks_before: BTreeMap<String, u32>,
    pub sticks_after: BTreeMap<String, u32>,
    pub stock: Vec<Card>,
    pub stock_revealed: bool,
    pub pending_trick: Option<Trick>,
    pub movements: Vec<Movement>,
    pub tricks: Vec<Trick>,
    pub marriages: Vec<Marriage>,
    pub dealer_bonus: u16,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub settings: Settings,
    pub dealer: usize,
    pub deal: u32,
    pub phase: Phase,
    pub ledger: Ledger,
    pub sticks: BTreeMap<String, u32>,
    checkpoint: Ledger,
    sticks_before: BTreeMap<String, u32>,
    active: Vec<usize>,
    barrel: Vec<usize>,
    hands: BTreeMap<String, Vec<Card>>,
    stock: Vec<Card>,
    stock_visible: bool,
    stock_taken: bool,
    bonus_paid: bool,
    dealer_bonus: u16,
    contractor: Option<usize>,
    order: u16,
    compulsory: bool,
    trump: Option<Suit>,
    raw: BTreeMap<String, u16>,
    tricks: Vec<Trick>,
    marriages: Vec<Marriage>,
    movements: Vec<Movement>,
    pub archives: Vec<DealRecord>,
}
fn event(kind: &'static str, data: Value) -> PublicEvent {
    PublicEvent { kind, data }
}
impl State {
    pub fn new(players: &[Player], settings: Settings, dealer: usize) -> Result<Self> {
        settings.validate()?;
        ensure(
            players.len() == usize::from(settings.player_count) && dealer < players.len(),
            Error::bad("Fill every seat before starting"),
        )?;
        let ledger = Ledger::new(players);
        let sticks = players
            .iter()
            .map(|p| (p.id.clone(), 0))
            .collect::<BTreeMap<_, _>>();
        Ok(Self {
            settings,
            dealer,
            deal: 0,
            phase: Phase::AwaitShuffle,
            checkpoint: ledger.clone(),
            ledger,
            sticks_before: sticks.clone(),
            sticks,
            active: vec![],
            barrel: vec![],
            hands: BTreeMap::new(),
            stock: vec![],
            stock_visible: false,
            stock_taken: false,
            bonus_paid: false,
            dealer_bonus: 0,
            contractor: None,
            order: 100,
            compulsory: true,
            trump: None,
            raw: BTreeMap::new(),
            tricks: vec![],
            marriages: vec![],
            movements: vec![],
            archives: vec![],
        })
    }
    fn id(players: &[Player], seat: usize) -> Result<&str> {
        players
            .get(seat)
            .map(|p| p.id.as_str())
            .ok_or_else(Error::internal)
    }
    fn actor(players: &[Player], actor: &str) -> Result<usize> {
        players
            .iter()
            .position(|p| p.id == actor)
            .ok_or_else(|| Error::bad("Unknown player"))
    }
    fn next(list: &[usize], seat: usize) -> Result<usize> {
        let index = list
            .iter()
            .position(|p| *p == seat)
            .ok_or_else(Error::internal)?;
        list.get((index + 1) % list.len())
            .copied()
            .ok_or_else(Error::internal)
    }
    fn move_card(&mut self, card: Card, from: Zone, to: Zone, actor: &str, public: bool) {
        self.movements.push(Movement {
            card,
            from,
            to,
            actor: actor.into(),
            public,
        });
    }
    fn take(&mut self, id: &str, card: &Card) -> Result<()> {
        let hand = self
            .hands
            .get_mut(id)
            .ok_or_else(|| Error::bad("No hand"))?;
        let pos = hand
            .iter()
            .position(|c| c == card)
            .ok_or_else(|| Error::bad("Card is not in your hand"))?;
        hand.remove(pos);
        Ok(())
    }
    fn reveal_stock(&mut self, players: &[Player], events: &mut Vec<PublicEvent>) -> Result<()> {
        if !self.stock_visible {
            self.stock_visible = true;
            events.push(event(
                "stock.revealed",
                json!({"deal":self.deal,"cards":self.stock}),
            ));
        }
        if !self.bonus_paid {
            self.bonus_paid = true;
            let dealer = Self::id(players, self.dealer)?;
            if players.len() == 4
                && self.checkpoint.get(dealer) < i64::from(self.settings.barrel_threshold)
            {
                self.dealer_bonus =
                    self.stock.iter().filter(|c| c.face() == "A").count() as u16 * 10;
                for s in [Suit::C, Suit::D, Suit::H, Suit::S] {
                    if self
                        .stock
                        .iter()
                        .any(|c| c.code() == format!("K{}", s.code()))
                        && self
                            .stock
                            .iter()
                            .any(|c| c.code() == format!("Q{}", s.code()))
                    {
                        self.dealer_bonus += s.marriage() / 2;
                    }
                }
                self.ledger.award(
                    dealer,
                    i64::from(self.dealer_bonus),
                    Some(i64::from(self.settings.barrel_threshold)),
                )?;
                events.push(event("dealer.bonus",json!({"deal":self.deal,"player":dealer,"points":self.dealer_bonus,"score":self.ledger.get(dealer)})));
            }
        }
        Ok(())
    }
    pub fn deal_cards(mut self, cards: Vec<Card>, players: &[Player]) -> Result<Outcome<Self>> {
        ensure(
            matches!(self.phase, Phase::AwaitShuffle),
            Error::bad("Not waiting for a shuffle"),
        )?;
        ensure(
            cards.len() == 24
                && cards.iter().cloned().collect::<BTreeSet<_>>()
                    == crate::model::thousand_deck().into_iter().collect(),
            Error::internal(),
        )?;
        self.deal += 1;
        self.checkpoint = self.ledger.clone();
        self.sticks_before = self.sticks.clone();
        self.active = (1..=players.len())
            .map(|n| (self.dealer + n) % players.len())
            .filter(|p| players.len() == 3 || *p != self.dealer)
            .collect();
        self.barrel = self
            .active
            .iter()
            .copied()
            .filter(|p| {
                self.ledger.get(&players[*p].id) == i64::from(self.settings.barrel_threshold)
            })
            .collect();
        self.hands = players.iter().map(|p| (p.id.clone(), vec![])).collect();
        self.raw = players.iter().map(|p| (p.id.clone(), 0)).collect();
        self.stock.clear();
        self.stock_visible = false;
        self.stock_taken = false;
        self.bonus_paid = false;
        self.dealer_bonus = 0;
        self.contractor = None;
        self.trump = None;
        self.tricks.clear();
        self.marriages.clear();
        self.movements.clear();
        let dealer = Self::id(players, self.dealer)?;
        for (i, card) in cards.into_iter().enumerate() {
            if i < 21 {
                let id = Self::id(players, self.active[i % 3])?;
                self.hands
                    .get_mut(id)
                    .ok_or_else(Error::internal)?
                    .push(card.clone());
                self.move_card(
                    card,
                    Zone::Deck,
                    Zone::Hand { player: id.into() },
                    dealer,
                    false,
                );
            } else {
                self.stock.push(card.clone());
                self.move_card(
                    card,
                    Zone::Deck,
                    Zone::Stock,
                    dealer,
                    !self.barrel.is_empty(),
                );
            }
        }
        let mut events = vec![event(
            "deal.started",
            json!({"deal":self.deal,"dealer":dealer,"active":self.active.iter().map(|p|&players[*p].id).collect::<Vec<_>>(),"openStock":!self.barrel.is_empty()}),
        )];
        if !self.barrel.is_empty() {
            self.reveal_stock(players, &mut events)?;
        }
        let eligible = if self.barrel.is_empty() {
            self.active.clone()
        } else {
            self.barrel.clone()
        };
        let leader = *eligible.first().ok_or_else(Error::internal)?;
        self.order = if self.barrel.is_empty() {
            100
        } else {
            1000 - self.settings.barrel_threshold
        };
        self.compulsory = true;
        if eligible.len() == 1 {
            self.contractor = Some(leader);
            self.phase = Phase::Stock;
            events.push(event("auction.finished",json!({"deal":self.deal,"contractor":Self::id(players,leader)?,"order":self.order,"compulsory":true})));
        } else {
            let turn = Self::next(&eligible, leader)?;
            self.phase = Phase::Auction(Auction {
                eligible,
                leader,
                turn,
                last_pass: false,
            });
            events.push(event(
                "auction.opened",
                json!({"deal":self.deal,"player":Self::id(players,leader)?,"amount":self.order}),
            ));
        }
        Ok(Outcome::Continue {
            state: self,
            events,
        })
    }
    fn finish_auction(
        &mut self,
        auction: Auction,
        players: &[Player],
        events: &mut Vec<PublicEvent>,
    ) -> Result<()> {
        self.contractor = Some(auction.leader);
        self.phase = Phase::Stock;
        self.reveal_stock(players, events)?;
        events.push(event("auction.finished",json!({"deal":self.deal,"contractor":Self::id(players,auction.leader)?,"order":self.order,"compulsory":self.compulsory})));
        Ok(())
    }
    fn record(&mut self, status: &str, players: &[Player]) -> Result<DealRecord> {
        let record = DealRecord {
            deal: self.deal,
            dealer: Self::id(players, self.dealer)?.into(),
            active: self.active.iter().map(|p| players[*p].id.clone()).collect(),
            barrel_players: self.barrel.iter().map(|p| players[*p].id.clone()).collect(),
            status: status.into(),
            contract: self.contractor.map(|p| Contract {
                player: players[p].id.clone(),
                order: self.order,
                compulsory: self.compulsory,
            }),
            before: self.checkpoint.clone(),
            after: self.ledger.clone(),
            sticks_before: self.sticks_before.clone(),
            sticks_after: self.sticks.clone(),
            stock: self.stock.clone(),
            stock_revealed: self.stock_visible,
            pending_trick: match &self.phase {
                Phase::Closing { trick, .. } => Some(trick.clone()),
                _ => None,
            },
            movements: self.movements.clone(),
            tricks: self.tricks.clone(),
            marriages: self.marriages.clone(),
            dealer_bonus: self.dealer_bonus,
        };
        self.archives.push(record.clone());
        Ok(record)
    }
    fn settled(
        mut self,
        status: &str,
        players: &[Player],
        mut events: Vec<PublicEvent>,
    ) -> Result<Outcome<Self>> {
        let record = self.record(status, players)?;
        events.push(event("deal.settled",json!({"deal":self.deal,"status":status,"recordHash":crate::crypto::hash(&record)?,"scores":self.ledger.totals,"sticks":self.sticks})));
        self.hands.clear();
        self.stock_visible = false;
        if self.ledger.totals.values().any(|s| *s >= 1000) {
            self.phase = Phase::Finished;
            return Ok(Outcome::Finished {
                scores: self.ledger.totals.clone(),
                state: self,
                events,
            });
        }
        self.dealer = (self.dealer + 1) % players.len();
        self.phase = Phase::AwaitShuffle;
        Ok(Outcome::Continue {
            state: self,
            events,
        })
    }
    pub fn legal_cards(&self, id: &str) -> Vec<Card> {
        let hand = self.hands.get(id).cloned().unwrap_or_default();
        let Phase::Trick { plays, .. } = &self.phase else {
            return vec![];
        };
        let Some(first) = plays.first() else {
            return hand;
        };
        let required = if hand.iter().any(|c| suit(c) == suit(&first.card)) {
            Some(suit(&first.card))
        } else {
            self.trump.filter(|t| hand.iter().any(|c| suit(c) == *t))
        };
        hand.into_iter()
            .filter(|c| required.is_none_or(|s| suit(c) == s))
            .collect()
    }
    pub fn can_announce(&self, id: &str, card: &Card) -> bool {
        let Phase::Trick { plays, .. } = &self.phase else {
            return false;
        };
        plays.is_empty()
            && matches!(card.face(), "K" | "Q")
            && self.tricks.iter().any(|t| t.winner == id)
            && self.hands.get(id).is_some_and(|h| {
                ["K", "Q"].iter().all(|r| {
                    h.iter()
                        .any(|c| c.code() == format!("{r}{}", suit(card).code()))
                })
            })
    }
    pub fn due(&self, now: i64) -> bool {
        matches!(self.phase,Phase::Closing{close_at,..}if close_at<=now)
    }
    pub fn close(mut self, players: &[Player], now: i64) -> Result<Outcome<Self>> {
        let Phase::Closing { trick, close_at } = &self.phase else {
            return Err(Error::bad("No trick to close"));
        };
        ensure(now >= *close_at, Error::conflict("Trick is still open"))?;
        let trick = trick.clone();
        for play in &trick.plays {
            self.move_card(
                play.card.clone(),
                Zone::Table {
                    trick: trick.number,
                },
                Zone::Pile {
                    player: trick.winner.clone(),
                    trick: trick.number,
                },
                "system",
                true,
            );
        }
        *self
            .raw
            .get_mut(&trick.winner)
            .ok_or_else(Error::internal)? += trick.points;
        self.tricks.push(trick.clone());
        self.phase = Phase::Trick {
            turn: Self::actor(players, &trick.winner)?,
            plays: vec![],
        };
        let events = vec![event(
            "trick.closed",
            json!({"deal":self.deal,"trick":trick}),
        )];
        if self.tricks.len() == 8 {
            let contractor = self.contractor.ok_or_else(Error::internal)?;
            for seat in &self.active {
                let id = Self::id(players, *seat)?;
                if *seat == contractor {
                    let delta = if self.raw.get(id).copied().unwrap_or(0) >= self.order {
                        i64::from(self.order)
                    } else {
                        -i64::from(self.order)
                    };
                    self.ledger.award(
                        id,
                        delta,
                        if self.barrel.contains(seat) {
                            None
                        } else {
                            Some(i64::from(self.settings.barrel_threshold))
                        },
                    )?;
                } else if !self.barrel.contains(seat) {
                    self.ledger.award(
                        id,
                        i64::from(self.raw.get(id).copied().unwrap_or(0)),
                        Some(i64::from(self.settings.barrel_threshold)),
                    )?;
                }
            }
            self.settled("completed", players, events)
        } else {
            self.phase = Phase::Trick {
                turn: Self::actor(players, &trick.winner)?,
                plays: vec![],
            };
            Ok(Outcome::Continue {
                state: self,
                events,
            })
        }
    }
    pub fn apply(
        mut self,
        actor: &str,
        action: Action,
        players: &[Player],
        now: i64,
    ) -> Result<Outcome<Self>> {
        let seat = Self::actor(players, actor)?;
        let mut events = vec![];
        if matches!(action, Action::FourNines) {
            ensure(
                !matches!(self.phase, Phase::AwaitShuffle | Phase::Finished)
                    && self
                        .hands
                        .get(actor)
                        .is_some_and(|h| h.iter().filter(|c| c.face() == "9").count() == 4),
                Error::bad("Four nines must still be in your hand"),
            )?;
            self.ledger = self.checkpoint.clone();
            self.sticks = self.sticks_before.clone();
            let record = self.record("cancelled", players)?;
            events.push(event("deal.cancelled",json!({"deal":self.deal,"player":actor,"recordHash":crate::crypto::hash(&record)?,"scores":self.ledger.totals})));
            self.hands.clear();
            self.stock_visible = false;
            self.phase = Phase::AwaitShuffle;
            return Ok(Outcome::Continue {
                state: self,
                events,
            });
        }
        match action {
            Action::Shuffle => {
                ensure(
                    matches!(self.phase, Phase::AwaitShuffle) && seat == self.dealer,
                    Error::bad("Only the dealer may start the next shuffle"),
                )?;
                return Ok(Outcome::Shuffle {
                    state: self,
                    cards: crate::model::thousand_deck(),
                    events,
                });
            }
            Action::Bid { .. } | Action::Here | Action::Pass => {
                let Phase::Auction(mut auction) = self.phase.clone() else {
                    return Err(Error::bad("No auction in progress"));
                };
                ensure(seat == auction.turn, Error::bad("Not your bidding turn"))?;
                match action {
                    Action::Pass => {
                        auction.eligible.retain(|p| *p != seat);
                        auction.last_pass = true;
                        events.push(event(
                            "auction.pass",
                            json!({"deal":self.deal,"player":actor}),
                        ));
                    }
                    Action::Here => {
                        ensure(
                            auction.last_pass,
                            Error::bad("Here is only available immediately after a pass"),
                        )?;
                        auction.leader = seat;
                        auction.last_pass = false;
                        self.compulsory = !self.barrel.is_empty();
                        events.push(event(
                            "auction.here",
                            json!({"deal":self.deal,"player":actor,"amount":self.order}),
                        ));
                    }
                    Action::Bid { amount } => {
                        ensure(
                            amount <= MAX_ORDER
                                && amount.is_multiple_of(5)
                                && amount >= self.order + 10,
                            Error::bad("Raise by at least 10 to a multiple of 5, up to 380"),
                        )?;
                        self.order = amount;
                        auction.leader = seat;
                        auction.last_pass = false;
                        self.compulsory = !self.barrel.is_empty();
                        events.push(event(
                            "auction.bid",
                            json!({"deal":self.deal,"player":actor,"amount":amount}),
                        ));
                    }
                    _ => return Err(Error::internal()),
                }
                if auction.eligible.len() == 1 {
                    self.finish_auction(auction, players, &mut events)?;
                } else {
                    let next = (1..=players.len())
                        .map(|n| (seat + n) % players.len())
                        .find(|s| auction.eligible.contains(s))
                        .ok_or_else(Error::internal)?;
                    auction.turn = next;
                    self.phase = Phase::Auction(auction);
                }
            }
            Action::TakeStock => {
                ensure(
                    matches!(self.phase, Phase::Stock) && self.contractor == Some(seat),
                    Error::bad("Only the contractor may take the stock"),
                )?;
                for card in self.stock.clone() {
                    self.hands
                        .get_mut(actor)
                        .ok_or_else(Error::internal)?
                        .push(card.clone());
                    self.move_card(
                        card,
                        Zone::Stock,
                        Zone::Hand {
                            player: actor.into(),
                        },
                        actor,
                        true,
                    );
                }
                self.stock_taken = true;
                self.phase = Phase::Distribution;
                events.push(event(
                    "stock.taken",
                    json!({"deal":self.deal,"player":actor}),
                ));
            }
            Action::Distribute { transfers } => {
                ensure(
                    matches!(self.phase, Phase::Distribution) && self.contractor == Some(seat),
                    Error::bad("Not your distribution"),
                )?;
                let expected: BTreeSet<_> = self
                    .active
                    .iter()
                    .filter(|s| **s != seat)
                    .map(|s| players[*s].id.clone())
                    .collect();
                ensure(
                    transfers.len() == 2
                        && transfers
                            .iter()
                            .map(|t| t.player.clone())
                            .collect::<BTreeSet<_>>()
                            == expected
                        && transfers[0].card != transfers[1].card,
                    Error::bad("Choose two different cards, one for each opponent"),
                )?;
                for transfer in transfers {
                    self.take(actor, &transfer.card)?;
                    self.hands
                        .get_mut(&transfer.player)
                        .ok_or_else(Error::internal)?
                        .push(transfer.card.clone());
                    self.move_card(
                        transfer.card.clone(),
                        Zone::Hand {
                            player: actor.into(),
                        },
                        Zone::Hand {
                            player: transfer.player.clone(),
                        },
                        actor,
                        transfer.public,
                    );
                    let mut data = json!({"deal":self.deal,"from":actor,"to":transfer.player,"public":transfer.public});
                    if transfer.public {
                        data["card"] = json!(transfer.card);
                    }
                    events.push(event("card.transferred", data));
                }
                self.phase = Phase::Preparation;
            }
            Action::Order { amount } => {
                ensure(
                    matches!(self.phase, Phase::Preparation) && self.contractor == Some(seat),
                    Error::bad("Not preparing your contract"),
                )?;
                ensure(
                    amount >= self.order && amount <= MAX_ORDER && amount.is_multiple_of(5),
                    Error::bad("Order must be a multiple of 5 from the winning bid through 380"),
                )?;
                self.order = amount;
                self.phase = Phase::Trick {
                    turn: seat,
                    plays: vec![],
                };
                events.push(event("contract.ordered",json!({"deal":self.deal,"player":actor,"amount":amount,"compulsory":self.compulsory})));
            }
            Action::Dismiss => {
                ensure(
                    matches!(
                        self.phase,
                        Phase::Stock | Phase::Distribution | Phase::Preparation
                    ) && self.contractor == Some(seat)
                        && !self.compulsory,
                    Error::bad("This contract cannot be dismissed"),
                )?;
                let count = self.sticks.get_mut(actor).ok_or_else(Error::internal)?;
                *count += 1;
                if count.is_multiple_of(3) {
                    self.ledger.award(actor, -150, None)?;
                }
                for other in &self.active {
                    if *other != seat && !self.barrel.contains(other) {
                        self.ledger.award(
                            Self::id(players, *other)?,
                            50,
                            Some(i64::from(self.settings.barrel_threshold)),
                        )?;
                    }
                }
                return self.settled("dismissed", players, events);
            }
            Action::Play { card, announce } => {
                let Phase::Trick { turn, mut plays } = self.phase.clone() else {
                    return Err(Error::bad("Not accepting a card now"));
                };
                ensure(turn == seat, Error::bad("Not your turn"))?;
                ensure(
                    self.legal_cards(actor).contains(&card),
                    Error::bad("Follow suit, otherwise trump if possible"),
                )?;
                let number = (self.tricks.len() + 1) as u8;
                if announce {
                    ensure(
                        self.can_announce(actor, &card),
                        Error::bad("Cannot announce this marriage"),
                    )?;
                    let s = suit(&card);
                    self.trump = Some(s);
                    *self.raw.get_mut(actor).ok_or_else(Error::internal)? += s.marriage();
                    self.marriages.push(Marriage {
                        player: actor.into(),
                        suit: s,
                        trick: number,
                    });
                    events.push(event("marriage.announced",json!({"deal":self.deal,"player":actor,"suit":s,"points":s.marriage(),"trick":number})));
                }
                self.take(actor, &card)?;
                self.move_card(
                    card.clone(),
                    Zone::Hand {
                        player: actor.into(),
                    },
                    Zone::Table { trick: number },
                    actor,
                    true,
                );
                plays.push(Play {
                    player: actor.into(),
                    card: card.clone(),
                });
                events.push(event(
                    "card.played",
                    json!({"deal":self.deal,"player":actor,"card":card,"trick":number}),
                ));
                if plays.len() == 3 {
                    let winner = winner(&plays, self.trump)?;
                    let trick = Trick {
                        number,
                        points: plays.iter().map(|p| points(&p.card)).sum(),
                        plays,
                        trump: self.trump,
                        winner,
                    };
                    self.phase = Phase::Closing {
                        trick: trick.clone(),
                        close_at: now + TRICK_DELAY_MS,
                    };
                    events.push(event(
                        "trick.completed",
                        json!({"deal":self.deal,"trick":trick,"closeAt":now+TRICK_DELAY_MS}),
                    ));
                } else {
                    self.phase = Phase::Trick {
                        turn: Self::next(&self.active, seat)?,
                        plays,
                    };
                }
            }
            Action::FourNines => return Err(Error::internal()),
        }
        Ok(Outcome::Continue {
            state: self,
            events,
        })
    }
}
pub fn winner(plays: &[Play], trump: Option<Suit>) -> Result<String> {
    let first = plays.first().ok_or_else(Error::internal)?;
    let led = suit(&first.card);
    plays
        .iter()
        .max_by_key(|p| {
            (
                if Some(suit(&p.card)) == trump {
                    2
                } else if suit(&p.card) == led {
                    1
                } else {
                    0
                },
                rank(&p.card),
            )
        })
        .map(|p| p.player.clone())
        .ok_or_else(Error::internal)
}

pub mod view;
pub use view::View;
#[cfg(test)]
mod tests;
