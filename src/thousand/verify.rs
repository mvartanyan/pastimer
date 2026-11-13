use super::*;
use std::collections::{BTreeMap, BTreeSet};
fn check(condition: bool) -> Result<()> {
    ensure(condition, Error::bad("Invalid Thousand deal record"))
}
/// Reconstruct all 24 card locations, legal plays, trick winners and score effects.
pub fn record(
    record: &DealRecord,
    deck: &[Card],
    settings: &Settings,
    cancelled_by: Option<&str>,
) -> Result<()> {
    check(
        deck.len() == 24
            && deck.iter().cloned().collect::<BTreeSet<_>>()
                == crate::model::thousand_deck().into_iter().collect(),
    )?;
    check(
        record.active.len() == 3
            && record.active.iter().collect::<BTreeSet<_>>().len() == 3
            && record
                .active
                .iter()
                .all(|id| record.before.totals.contains_key(id)),
    )?;
    check(
        record.barrel_players
            == record
                .active
                .iter()
                .filter(|id| record.before.get(id) == i64::from(settings.barrel_threshold))
                .cloned()
                .collect::<Vec<_>>(),
    )?;
    check(record.stock == deck[21..])?;
    let mut zones: BTreeMap<Card, Zone> = deck.iter().cloned().map(|c| (c, Zone::Deck)).collect();
    let mut plays: Vec<Play> = vec![];
    let mut completed: Vec<Trick> = vec![];
    let mut trump = None;
    let mut announced = 0usize;
    let mut collected = 0usize;
    let mut gifts = BTreeSet::new();
    let mut stock_taken = 0usize;
    let mut expected_leader = record.contract.as_ref().map(|c| c.player.clone());
    for (index, m) in record.movements.iter().enumerate() {
        check(zones.get(&m.card) == Some(&m.from))?;
        if index < 24 {
            check(m.card == deck[index] && m.from == Zone::Deck && m.actor == record.dealer)?;
            let expected = if index < 21 {
                Zone::Hand {
                    player: record.active[index % 3].clone(),
                }
            } else {
                Zone::Stock
            };
            check(m.to == expected)?;
        } else {
            match (&m.from, &m.to) {
                (Zone::Stock, Zone::Hand { player }) => {
                    check(
                        record
                            .contract
                            .as_ref()
                            .is_some_and(|c| c.player == *player)
                            && m.actor == *player
                            && stock_taken < 3
                            && plays.is_empty(),
                    )?;
                    stock_taken += 1;
                }
                (Zone::Hand { player: from }, Zone::Hand { player: to }) => {
                    check(
                        stock_taken == 3
                            && record.contract.as_ref().is_some_and(|c| c.player == *from)
                            && from != to
                            && record.active.contains(to)
                            && m.actor == *from
                            && plays.is_empty()
                            && completed.is_empty()
                            && gifts.insert(to.clone()),
                    )?;
                }
                (Zone::Hand { player }, Zone::Table { trick }) => {
                    check(
                        stock_taken == 3
                            && gifts.len() == 2
                            && collected == completed.len() * 3
                            && usize::from(*trick) == completed.len() + 1
                            && plays.len() < 3
                            && m.actor == *player,
                    )?;
                    let leader = expected_leader.as_ref().ok_or_else(Error::internal)?;
                    let start = record
                        .active
                        .iter()
                        .position(|p| p == leader)
                        .ok_or_else(Error::internal)?;
                    check(record.active[(start + plays.len()) % 3] == *player)?;
                    let hand: Vec<_> = zones
                        .iter()
                        .filter_map(|(c, z)| (z == &m.from).then_some(c))
                        .collect();
                    if let Some(first) = plays.first() {
                        let led = suit(&first.card);
                        let required = if hand.iter().any(|c| suit(c) == led) {
                            Some(led)
                        } else {
                            trump.filter(|t| hand.iter().any(|c| suit(c) == *t))
                        };
                        check(required.is_none_or(|s| suit(&m.card) == s))?;
                    }
                    if let Some(marriage) = record
                        .marriages
                        .get(announced)
                        .filter(|a| a.trick == *trick)
                    {
                        check(
                            plays.is_empty()
                                && marriage.player == *player
                                && suit(&m.card) == marriage.suit
                                && matches!(m.card.face(), "K" | "Q")
                                && completed.iter().any(|t| t.winner == *player)
                                && ["K", "Q"].iter().all(|r| {
                                    hand.iter()
                                        .any(|c| c.code() == format!("{r}{}", marriage.suit.code()))
                                }),
                        )?;
                        trump = Some(marriage.suit);
                        announced += 1;
                    }
                    plays.push(Play {
                        player: player.clone(),
                        card: m.card.clone(),
                    });
                    if plays.len() == 3 {
                        let t = Trick {
                            number: *trick,
                            winner: winner(&plays, trump)?,
                            points: plays.iter().map(|p| points(&p.card)).sum(),
                            plays: plays.clone(),
                            trump,
                        };
                        expected_leader = Some(t.winner.clone());
                        completed.push(t);
                        plays.clear();
                    }
                }
                (
                    Zone::Table { trick },
                    Zone::Pile {
                        player,
                        trick: target,
                    },
                ) => {
                    let t = completed.last().ok_or_else(Error::internal)?;
                    check(
                        *trick == t.number
                            && target == trick
                            && t.winner == *player
                            && m.actor == "system"
                            && t.plays.get(collected % 3).is_some_and(|p| p.card == m.card),
                    )?;
                    collected += 1;
                }
                _ => return Err(Error::bad("Invalid card movement")),
            }
        }
        zones.insert(m.card.clone(), m.to.clone());
    }
    check(record.movements.len() >= 24 && announced == record.marriages.len())?;
    let mut expected = record.tricks.clone();
    if let Some(pending) = &record.pending_trick {
        expected.push(pending.clone());
    }
    check(completed == expected && collected == record.tricks.len() * 3)?;
    let mut ledger = record.before.clone();
    let mut sticks = record.sticks_before.clone();
    let bonus = if record.before.totals.len() == 4
        && record.before.get(&record.dealer) < i64::from(settings.barrel_threshold)
        && record.stock_revealed
    {
        let aces = record.stock.iter().filter(|c| c.face() == "A").count() as u16 * 10;
        aces + [Suit::C, Suit::D, Suit::H, Suit::S]
            .iter()
            .filter(|s| {
                ["K", "Q"].iter().all(|r| {
                    record
                        .stock
                        .iter()
                        .any(|c| c.code() == format!("{r}{}", s.code()))
                })
            })
            .map(|s| s.marriage() / 2)
            .sum::<u16>()
    } else {
        0
    };
    check(bonus == record.dealer_bonus)?;
    if record.status == "cancelled" {
        let actor = cancelled_by.ok_or_else(|| Error::bad("Missing cancellation actor"))?;
        check(
            zones
                .iter()
                .filter(|(c, z)| {
                    c.face() == "9"
                        && **z
                            == Zone::Hand {
                                player: actor.into(),
                            }
                })
                .count()
                == 4,
        )?;
    } else {
        ledger.award(
            &record.dealer,
            i64::from(bonus),
            Some(i64::from(settings.barrel_threshold)),
        )?;
        let contract = record.contract.as_ref().ok_or_else(Error::internal)?;
        check(
            record.active.contains(&contract.player)
                && (100..=MAX_ORDER).contains(&contract.order)
                && contract.order.is_multiple_of(5),
        )?;
        if !record.barrel_players.is_empty() {
            check(
                record.barrel_players.contains(&contract.player)
                    && contract.compulsory
                    && contract.order >= 1000 - settings.barrel_threshold,
            )?;
        }
        match record.status.as_str() {
            "completed" => {
                check(
                    record.tricks.len() == 8
                        && record.pending_trick.is_none()
                        && zones.values().all(|z| matches!(z, Zone::Pile { .. })),
                )?;
                for id in &record.active {
                    let raw = record
                        .tricks
                        .iter()
                        .filter(|t| t.winner == *id)
                        .map(|t| t.points)
                        .sum::<u16>()
                        + record
                            .marriages
                            .iter()
                            .filter(|m| m.player == *id)
                            .map(|m| m.suit.marriage())
                            .sum::<u16>();
                    if id == &contract.player {
                        let delta = if raw >= contract.order {
                            i64::from(contract.order)
                        } else {
                            -i64::from(contract.order)
                        };
                        ledger.award(
                            id,
                            delta,
                            if record.barrel_players.contains(id) {
                                None
                            } else {
                                Some(i64::from(settings.barrel_threshold))
                            },
                        )?;
                    } else if !record.barrel_players.contains(id) {
                        ledger.award(
                            id,
                            i64::from(raw),
                            Some(i64::from(settings.barrel_threshold)),
                        )?;
                    }
                }
            }
            "dismissed" => {
                check(
                    !contract.compulsory
                        && record.tricks.is_empty()
                        && record.pending_trick.is_none()
                        && plays.is_empty(),
                )?;
                let count = sticks
                    .get_mut(&contract.player)
                    .ok_or_else(Error::internal)?;
                *count += 1;
                if count.is_multiple_of(3) {
                    ledger.award(&contract.player, -150, None)?;
                }
                for id in &record.active {
                    if id != &contract.player && !record.barrel_players.contains(id) {
                        ledger.award(id, 50, Some(i64::from(settings.barrel_threshold)))?;
                    }
                }
            }
            _ => return Err(Error::bad("Invalid deal status")),
        }
    }
    check(ledger == record.after && sticks == record.sticks_after)
}
