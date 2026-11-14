use crate::{
    model::{Action, Card, GameState, Lifecycle, Room, thousand_deck},
    thousand::{self, Play, Suit, Transfer, View},
};

fn marriages(hand: &[Card]) -> impl Iterator<Item = Suit> + '_ {
    [Suit::H, Suit::D, Suit::C, Suit::S]
        .into_iter()
        .filter(|s| {
            ["K", "Q"]
                .iter()
                .all(|r| hand.iter().any(|c| c.code() == format!("{r}{}", s.code())))
        })
}
fn cash_cards(hand: &[Card], played: &[Card], protected: Option<Suit>) -> Vec<Card> {
    hand.iter()
        .filter(|c| {
            !(Some(thousand::suit(c)) == protected && matches!(c.face(), "K" | "Q"))
                && thousand_deck().iter().all(|other| {
                    thousand::suit(other) != thousand::suit(c)
                        || thousand::rank(other) <= thousand::rank(c)
                        || hand.contains(other)
                        || played.contains(other)
                })
        })
        .cloned()
        .collect()
}
fn cash_points(hand: &[Card], winners: &[Card], played: &[Card]) -> u16 {
    let mut unseen: Vec<_> = thousand_deck()
        .into_iter()
        .filter(|c| !hand.contains(c) && !played.contains(c))
        .map(|c| thousand::points(&c))
        .collect();
    unseen.sort_unstable();
    winners.iter().map(thousand::points).sum::<u16>()
        + unseen.into_iter().take(2 * winners.len()).sum::<u16>()
}
/// A lower bound while retaining the opening lead: cash unbeatable cards before
/// declaring one protected marriage. Never count stock luck or an unseen hand.
#[derive(Debug, PartialEq, Eq)]
pub struct Estimate {
    pub points: u16,
    pub tricks: usize,
    pub marriage: Option<Suit>,
}
pub fn estimate(hand: &[Card]) -> Estimate {
    estimate_known(hand, &[], false)
}
fn estimate_known(hand: &[Card], played: &[Card], has_entry: bool) -> Estimate {
    let cash = cash_cards(hand, played, None);
    let mut best = Estimate {
        points: cash_points(hand, &cash, played),
        tricks: cash.len(),
        marriage: None,
    };
    for suit in marriages(hand) {
        let winners = cash_cards(hand, played, Some(suit));
        if winners.is_empty() && !has_entry {
            continue;
        }
        let points = cash_points(hand, &winners, played) + suit.marriage();
        if points > best.points {
            best = Estimate {
                points,
                tricks: winners.len(),
                marriage: Some(suit),
            };
        }
    }
    best
}
fn distribution(hand: &[Card], recipients: &[crate::model::Player]) -> Vec<Transfer> {
    let mut best = None;
    for a in 0..hand.len() {
        for b in a + 1..hand.len() {
            let kept: Vec<_> = hand
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != a && *i != b)
                .map(|(_, c)| c.clone())
                .collect();
            let score = (
                estimate(&kept).points,
                20 - thousand::points(&hand[a]) - thousand::points(&hand[b]),
                10 - thousand::rank(&hand[a]) - thousand::rank(&hand[b]),
            );
            if best.as_ref().is_none_or(|(old, _, _)| score > *old) {
                best = Some((score, a, b));
            }
        }
    }
    best.map_or_else(Vec::new, |(_, a, b)| {
        recipients
            .iter()
            .zip([a, b])
            .map(|(p, i)| Transfer {
                player: p.id.clone(),
                card: hand[i].clone(),
                public: false,
            })
            .collect()
    })
}
fn choose(view: View) -> Option<thousand::Action> {
    let c = &view.controls;
    if c.pass {
        let bound = estimate(&view.hand).points.min(thousand::MAX_ORDER);
        if c.here && bound >= view.order {
            return Some(thousand::Action::Here);
        }
        if let Some(amount) = c.bid_min.filter(|amount| *amount <= bound) {
            return Some(thousand::Action::Bid { amount });
        }
        return Some(thousand::Action::Pass);
    }
    if c.take_stock {
        return Some(thousand::Action::TakeStock);
    }
    if c.recipients.len() == 2 {
        return Some(thousand::Action::Distribute {
            transfers: distribution(&view.hand, &c.recipients),
        });
    }
    if c.order {
        return Some(thousand::Action::Order { amount: view.order });
    }
    if c.legal_cards.is_empty() {
        return None;
    }
    let mut legal = c.legal_cards.clone();
    legal.sort_by_key(|card| (thousand::points(card), thousand::rank(card)));
    let mut announce = false;
    let card = if view.plays.is_empty() {
        let protected =
            estimate_known(&view.hand, &view.played_cards, !c.announce_cards.is_empty()).marriage;
        let cash = cash_cards(&view.hand, &view.played_cards, protected);
        // Before declaring trump, realize the tricks used by the bidding bound.
        let winner = if view.trump.is_none() {
            cash.iter().max_by_key(|c| thousand::rank(c)).cloned()
        } else {
            cash.iter()
                .filter(|c| Some(thousand::suit(c)) == view.trump)
                .max_by_key(|c| thousand::rank(c))
                .cloned()
        };
        winner
            .or_else(|| {
                let marriage = c
                    .announce_cards
                    .iter()
                    .max_by_key(|c| thousand::suit(c).marriage())
                    .cloned();
                announce = marriage.is_some();
                marriage
            })
            .or_else(|| legal.iter().find(|c| c.face() == "A").cloned())
            .or_else(|| legal.first().cloned())?
    } else {
        legal
            .iter()
            .find(|card| {
                let mut trick = view.plays.clone();
                trick.push(Play {
                    player: "computer-candidate".into(),
                    card: (*card).clone(),
                });
                thousand::winner(&trick, view.trump)
                    .is_ok_and(|winner| winner == "computer-candidate")
            })
            .cloned()
            .or_else(|| legal.first().cloned())?
    };
    Some(thousand::Action::Play { card, announce })
}
pub fn next(room: &Room, now: i64) -> Option<(String, Action)> {
    let Lifecycle::Playing(GameState::Thousand(state)) = &room.lifecycle else {
        return None;
    };
    room.computers.iter().find_map(|id| {
        choose(state.view(id, &room.players, now))
            .map(|action| (id.clone(), Action::Thousand { action }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cards(value: &str) -> crate::error::Result<Vec<Card>> {
        value.split_whitespace().map(str::parse).collect()
    }
    fn view(hand: Vec<Card>) -> View {
        View {
            kind: "thousand",
            phase: "trick",
            deal: 1,
            dealer: "me".into(),
            contractor: Some("me".into()),
            order: 110,
            compulsory: false,
            turn: Some("me".into()),
            hand: hand.clone(),
            stock: None,
            stock_taken: true,
            trump: None,
            seats: vec![],
            piles: vec![],
            plays: vec![],
            close_at: None,
            server_now: 0,
            controls: thousand::view::Controls {
                legal_cards: hand,
                ..Default::default()
            },
            transfers: vec![],
            deliveries: vec![],
            played_cards: vec![],
            last_deal: None,
            audit_deals: 0,
        }
    }
    #[test]
    fn bids_then_cashes_an_ace_before_declaring_the_protected_marriage() -> crate::error::Result<()>
    {
        let hand = cards("AC KH QH 9C JC 9D JS")?;
        let mut auction = view(hand.clone());
        auction.controls.pass = true;
        auction.controls.bid_min = Some(110);
        assert!(matches!(
            choose(auction),
            Some(thousand::Action::Bid { amount: 110 })
        ));
        assert!(
            matches!(choose(view(hand.clone())),Some(thousand::Action::Play{card,announce:false})if card.code()=="AC")
        );
        let mut next = view(hand.into_iter().filter(|c| c.code() != "AC").collect());
        next.played_cards = cards("AC KC 10C")?;
        next.controls.announce_cards = cards("KH QH")?;
        assert!(
            matches!(choose(next),Some(thousand::Action::Play{card,announce:true})if thousand::suit(&card)==Suit::H)
        );
        Ok(())
    }
    #[test]
    fn takes_a_winnable_trick_instead_of_throwing_away_the_lowest_card() -> crate::error::Result<()>
    {
        let mut own = view(cards("AC 9C")?);
        own.plays = vec![
            Play {
                player: "a".into(),
                card: "KC".parse()?,
            },
            Play {
                player: "b".into(),
                card: "10C".parse()?,
            },
        ];
        assert!(matches!(choose(own),Some(thousand::Action::Play{card,..})if card.code()=="AC"));
        Ok(())
    }
    #[test]
    fn guaranteed_bids_require_a_won_trick_before_a_marriage() -> crate::error::Result<()> {
        let no_entry = cards("KH QH 9C JC QD 9S JS")?;
        assert_eq!(estimate(&no_entry).points, 0);
        let with_entry = cards("KH QH AC JC QD 9S JS")?;
        assert!(estimate(&with_entry).points >= 110);
        assert_eq!(estimate(&with_entry).marriage, Some(Suit::H));
        Ok(())
    }
    #[test]
    fn distribution_preserves_the_guaranteed_contract() -> crate::error::Result<()> {
        let hand = cards("AC KH QH 10S 9S JS 9D JD QC 9C")?;
        let recipients = [
            crate::model::Player {
                id: "a".into(),
                name: "A".into(),
            },
            crate::model::Player {
                id: "b".into(),
                name: "B".into(),
            },
        ];
        let gifts = distribution(&hand, &recipients);
        let kept: Vec<_> = hand
            .into_iter()
            .filter(|c| !gifts.iter().any(|t| t.card == *c))
            .collect();
        assert!(estimate(&kept).points >= 110);
        assert!(kept.iter().any(|c| c.code() == "AC"));
        assert!(kept.iter().any(|c| c.code() == "KH"));
        assert!(kept.iter().any(|c| c.code() == "QH"));
        Ok(())
    }
    #[test]
    fn does_not_switch_trump_while_cashing_the_guaranteed_leads() -> crate::error::Result<()> {
        let mut own = view(cards("KH QH KC QC JC 9D 9S")?);
        own.played_cards = cards("AC 10C")?;
        own.controls.announce_cards = cards("KH QH KC QC")?;
        assert!(
            matches!(choose(own),Some(thousand::Action::Play{card,announce:false})if card.code()=="KC")
        );
        Ok(())
    }
    #[test]
    fn guaranteed_bid_is_actually_made_against_legal_defenders() -> crate::error::Result<()> {
        use crate::game::Outcome;
        fn state(outcome: Outcome<thousand::State>) -> thousand::State {
            match outcome {
                Outcome::Continue { state, .. }
                | Outcome::Shuffle { state, .. }
                | Outcome::Finished { state, .. } => state,
            }
        }
        let players = vec![
            crate::model::Player {
                id: "a".into(),
                name: "A".into(),
            },
            crate::model::Player {
                id: "b".into(),
                name: "B".into(),
            },
            crate::model::Player {
                id: "computer".into(),
                name: "Computer".into(),
            },
        ];
        let hand = cards("AC KH QH 9S JS 9D JD")?;
        let rest: Vec<_> = thousand_deck()
            .into_iter()
            .filter(|c| !hand.contains(c))
            .collect();
        let mut deck = vec![];
        for i in 0..7 {
            deck.extend([rest[i].clone(), hand[i].clone(), rest[i + 7].clone()]);
        }
        deck.extend_from_slice(&rest[14..]);
        let initial = thousand::State::new(&players, thousand::Settings::default(), 0)?;
        let mut game = state(initial.deal_cards(deck, &players)?);
        let mut now = 0;
        for _ in 0..100 {
            if !game.archives.is_empty() {
                break;
            }
            if matches!(game.phase, thousand::Phase::Closing { .. }) {
                now += 3000;
                game = state(game.close(&players, now)?);
                continue;
            }
            let mut moved = false;
            for player in &players {
                let own = game.view(&player.id, &players, now);
                let action = if player.id == "computer" {
                    choose(own)
                } else if own.controls.pass {
                    Some(thousand::Action::Pass)
                } else {
                    own.controls
                        .legal_cards
                        .into_iter()
                        .max_by_key(thousand::rank)
                        .map(|card| thousand::Action::Play {
                            card,
                            announce: false,
                        })
                };
                if let Some(action) = action {
                    game = state(game.apply(&player.id, action, &players, now)?);
                    moved = true;
                    break;
                }
            }
            assert!(moved, "The fixture must reach a legal turn");
        }
        let record = game
            .archives
            .first()
            .ok_or_else(crate::error::Error::internal)?;
        let contract = record
            .contract
            .as_ref()
            .ok_or_else(crate::error::Error::internal)?;
        assert_eq!(contract.player, "computer");
        assert_eq!(contract.order, 110);
        assert_eq!(
            record.after.get("computer") - record.before.get("computer"),
            110
        );
        Ok(())
    }
}
