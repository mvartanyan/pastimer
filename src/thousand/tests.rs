use super::*;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn players(n: u8) -> Vec<Player> {
    (0..n)
        .map(|i| Player {
            id: format!("p{i}"),
            name: format!("Player {i}"),
        })
        .collect()
}
fn cards(s: &str) -> Result<Vec<Card>> {
    s.split_whitespace().map(str::parse).collect()
}
fn state(outcome: Outcome<State>) -> State {
    match outcome {
        Outcome::Continue { state, .. }
        | Outcome::Finished { state, .. }
        | Outcome::Shuffle { state, .. } => state,
    }
}
fn apply(s: State, id: &str, action: Action, p: &[Player]) -> Result<State> {
    Ok(state(s.apply(id, action, p, 10000)?))
}
fn dealt(n: u8) -> Result<(Vec<Player>, State, Vec<Card>)> {
    let p = players(n);
    let deck = crate::model::thousand_deck();
    let s = State::new(
        &p,
        Settings {
            player_count: n,
            ..Settings::default()
        },
        0,
    )?;
    Ok((p.clone(), state(s.deal_cards(deck.clone(), &p)?), deck))
}
fn pass_all(mut s: State, p: &[Player]) -> Result<State> {
    while let Phase::Auction(a) = s.phase.clone() {
        let id = p[a.turn].id.clone();
        s = apply(s, &id, Action::Pass, p)?;
    }
    Ok(s)
}
fn prepare(mut s: State, p: &[Player]) -> Result<State> {
    s = pass_all(s, p)?;
    let id = State::id(p, s.contractor.ok_or_else(Error::internal)?)?.to_owned();
    s = apply(s, &id, Action::TakeStock, p)?;
    let hand = s.hands.get(&id).ok_or_else(Error::internal)?;
    let recipients: Vec<_> = s
        .active
        .iter()
        .filter(|i| p[**i].id != id)
        .map(|i| p[*i].id.clone())
        .collect();
    let transfers = vec![
        Transfer {
            player: recipients[0].clone(),
            card: hand[0].clone(),
            public: true,
        },
        Transfer {
            player: recipients[1].clone(),
            card: hand[1].clone(),
            public: false,
        },
    ];
    s = apply(s, &id, Action::Distribute { transfers }, p)?;
    let order = s.order;
    apply(s, &id, Action::Order { amount: order }, p)
}
fn play_deal(mut s: State, p: &[Player]) -> Result<State> {
    loop {
        match &s.phase {
            Phase::Trick { turn, .. } => {
                let id = p[*turn].id.clone();
                let card = s
                    .legal_cards(&id)
                    .first()
                    .cloned()
                    .ok_or_else(Error::internal)?;
                let announce = s.can_announce(&id, &card);
                s = apply(s, &id, Action::Play { card, announce }, p)?;
            }
            Phase::Closing { close_at, .. } => {
                let now = *close_at;
                s = state(s.close(p, now)?);
            }
            _ => return Ok(s),
        }
    }
}
#[test]
fn ordinary_auction_here_pass_and_compulsory_status() -> TestResult {
    let (p, s, _) = dealt(3)?;
    assert_eq!(s.order, 100);
    assert!(s.compulsory);
    assert!(s.view("p0", &p, 0).stock.is_none());
    assert!(
        s.clone()
            .apply("p2", Action::Bid { amount: 105 }, &p, 0)
            .is_err()
    );
    assert!(s.clone().apply("p0", Action::Pass, &p, 0).is_err());
    let s = apply(s, "p2", Action::Pass, &p)?;
    let s = apply(s, "p0", Action::Here, &p)?;
    assert!(!s.compulsory);
    assert!(
        s.clone()
            .apply("p2", Action::Bid { amount: 110 }, &p, 0)
            .is_err()
    );
    let s = apply(s, "p1", Action::Bid { amount: 115 }, &p)?;
    let s = apply(s, "p0", Action::Pass, &p)?;
    assert_eq!(s.contractor, Some(1));
    assert!(!s.compulsory);
    assert!(s.stock_visible);
    let s = apply(s, "p1", Action::Dismiss, &p)?;
    assert_eq!(s.ledger.get("p0"), 50);
    assert_eq!(s.ledger.get("p2"), 50);
    assert_eq!(s.ledger.get("p1"), 0);
    assert_eq!(s.sticks["p1"], 1);
    Ok(())
}
#[test]
fn final_order_accepts_five_point_increases_without_removing_obligation() -> TestResult {
    let (p, s, _) = dealt(3)?;
    let mut s = pass_all(s, &p)?;
    assert!(s.clone().apply("p1", Action::Dismiss, &p, 0).is_err());
    s = apply(s, "p1", Action::TakeStock, &p)?;
    let hand = s.hands["p1"].clone();
    s = apply(
        s,
        "p1",
        Action::Distribute {
            transfers: vec![
                Transfer {
                    player: "p2".into(),
                    card: hand[0].clone(),
                    public: true,
                },
                Transfer {
                    player: "p0".into(),
                    card: hand[1].clone(),
                    public: false,
                },
            ],
        },
        &p,
    )?;
    assert!(
        s.clone()
            .apply("p1", Action::Order { amount: 385 }, &p, 0)
            .is_err()
    );
    s = apply(s, "p1", Action::Order { amount: 115 }, &p)?;
    assert!(s.compulsory);
    assert_eq!(s.order, 115);
    assert!(s.apply("p1", Action::Dismiss, &p, 0).is_err());
    Ok(())
}
#[test]
fn transfers_are_atomic_private_and_allow_independent_visibility() -> TestResult {
    let (p, s, _) = dealt(4)?;
    let s = pass_all(s, &p)?;
    let s = apply(s, "p1", Action::TakeStock, &p)?;
    let hand = s.hands["p1"].clone();
    assert!(
        s.clone()
            .apply(
                "p1",
                Action::Distribute {
                    transfers: vec![
                        Transfer {
                            player: "p2".into(),
                            card: hand[0].clone(),
                            public: true
                        },
                        Transfer {
                            player: "p2".into(),
                            card: hand[1].clone(),
                            public: false
                        }
                    ]
                },
                &p,
                0
            )
            .is_err()
    );
    let s = apply(
        s,
        "p1",
        Action::Distribute {
            transfers: vec![
                Transfer {
                    player: "p2".into(),
                    card: hand[0].clone(),
                    public: true,
                },
                Transfer {
                    player: "p3".into(),
                    card: hand[1].clone(),
                    public: false,
                },
            ],
        },
        &p,
    )?;
    assert!(
        s.hands
            .values()
            .filter(|h| !h.is_empty())
            .all(|h| h.len() == 8)
    );
    assert_eq!(s.view("p0", &p, 0).transfers.len(), 1);
    assert_eq!(s.view("p3", &p, 0).transfers.len(), 2);
    assert!(s.view("p0", &p, 0).hand.is_empty());
    Ok(())
}
#[test]
fn follow_suit_then_trump_without_obligation_to_overtake() -> TestResult {
    let (p, mut s, _) = dealt(3)?;
    s.phase = Phase::Trick {
        turn: 2,
        plays: vec![Play {
            player: "p1".into(),
            card: "AH".parse()?,
        }],
    };
    s.trump = Some(Suit::S);
    s.hands.insert("p2".into(), cards("9H KH AS")?);
    assert_eq!(s.legal_cards("p2"), cards("9H KH")?);
    s.hands.insert("p2".into(), cards("9S AS AC")?);
    assert_eq!(s.legal_cards("p2"), cards("9S AS")?);
    s.hands.insert("p2".into(), cards("AC 9D")?);
    assert_eq!(s.legal_cards("p2"), cards("AC 9D")?);
    assert!(
        s.apply(
            "p0",
            Action::Play {
                card: "AC".parse()?,
                announce: false
            },
            &p,
            0
        )
        .is_err()
    );
    Ok(())
}
#[test]
fn closure_deadline_and_each_players_top_trick_are_distinct() -> TestResult {
    let (p, s, _) = dealt(3)?;
    let mut s = prepare(s, &p)?;
    for _ in 0..3 {
        let Phase::Trick { turn, .. } = s.phase else {
            return Err("trick".into());
        };
        let id = p[turn].id.clone();
        let card = s.legal_cards(&id)[0].clone();
        s = apply(
            s,
            &id,
            Action::Play {
                card,
                announce: false,
            },
            &p,
        )?;
    }
    let Phase::Closing {
        close_at,
        ref trick,
    } = s.phase
    else {
        return Err("closing".into());
    };
    assert_eq!(close_at, 13000);
    let winner = trick.winner.clone();
    assert!(s.clone().close(&p, 12999).is_err());
    assert!(
        s.view("p0", &p, 12000)
            .piles
            .iter()
            .all(|pile| pile.top.is_none())
    );
    s = state(s.close(&p, 13000)?);
    let view = s.view("p0", &p, 13000);
    assert_eq!(view.piles.iter().filter(|p| p.top.is_some()).count(), 1);
    assert!(view.piles.iter().any(|p| p.player == winner
        && p.count == 1
        && p.top.as_ref().is_some_and(|t| t.winner == winner)));
    let other = p
        .iter()
        .find(|p| p.id != winner)
        .ok_or("player")?
        .id
        .clone();
    let mut second = s.tricks[0].clone();
    second.number = 2;
    second.winner = other.clone();
    s.tricks.push(second);
    let view = s.view("p0", &p, 14000);
    assert_eq!(view.piles.iter().filter(|p| p.top.is_some()).count(), 2);
    assert_eq!(
        view.piles
            .iter()
            .find(|p| p.player == winner)
            .and_then(|p| p.top.as_ref())
            .map(|t| t.number),
        Some(1)
    );
    Ok(())
}
#[test]
fn complete_deals_reconstruct_all_card_movements_and_rotate_dealer() -> TestResult {
    for n in [3, 4] {
        let (p, s, deck) = dealt(n)?;
        let s = play_deal(prepare(s, &p)?, &p)?;
        assert!(matches!(s.phase, Phase::AwaitShuffle));
        assert_eq!(s.dealer, 1);
        let record = &s.archives[0];
        assert_eq!(record.tricks.len(), 8);
        assert_eq!(record.movements.len(), 24 + 3 + 2 + 24 + 24);
        verify::record(record, &deck, &s.settings, None)?;
        let s = state(s.deal_cards(deck, &p)?);
        assert!(s.tricks.is_empty());
        assert!(s.view("p1", &p, 0).piles.iter().all(|p| p.top.is_none()));
    }
    Ok(())
}
#[test]
fn barrel_stock_only_counts_active_players_and_every_contract_is_compulsory() -> TestResult {
    let p = players(4);
    let mut initial = State::new(
        &p,
        Settings {
            player_count: 4,
            ..Settings::default()
        },
        0,
    )?;
    initial.ledger.totals.insert("p0".into(), 900);
    let s = state(
        initial
            .clone()
            .deal_cards(crate::model::thousand_deck(), &p)?,
    );
    assert!(!s.stock_visible);
    assert!(matches!(s.phase, Phase::Auction(_)));
    assert_eq!(s.dealer_bonus, 0);
    initial.ledger.totals.insert("p2".into(), 900);
    let s = state(
        initial
            .clone()
            .deal_cards(crate::model::thousand_deck(), &p)?,
    );
    assert!(s.stock_visible);
    assert!(matches!(s.phase, Phase::Stock));
    assert_eq!(s.contractor, Some(2));
    assert_eq!(s.order, 100);
    assert_eq!(s.dealer_bonus, 0);
    assert!(s.apply("p2", Action::Dismiss, &p, 0).is_err());
    initial.ledger.totals.insert("p3".into(), 900);
    let s = state(initial.deal_cards(crate::model::thousand_deck(), &p)?);
    let Phase::Auction(a) = s.phase.clone() else {
        return Err("auction".into());
    };
    assert_eq!(a.leader, 2);
    assert_eq!(a.turn, 3);
    let s = apply(s, "p3", Action::Bid { amount: 110 }, &p)?;
    let s = apply(s, "p2", Action::Pass, &p)?;
    assert!(s.compulsory);
    assert!(s.apply("p3", Action::Dismiss, &p, 0).is_err());
    Ok(())
}
#[test]
fn four_nines_reverses_dealer_bonus_and_preserves_dealer_and_audit() -> TestResult {
    let p = players(4);
    let mut initial = State::new(
        &p,
        Settings {
            player_count: 4,
            ..Settings::default()
        },
        0,
    )?;
    initial.ledger.totals.insert("p0".into(), 880);
    initial.ledger.totals.insert("p2".into(), 900);
    let nines = cards("9C 9D 9H 9S AC AD AH")?;
    let stock = cards("AS KH QH")?;
    let remaining: Vec<_> = crate::model::thousand_deck()
        .into_iter()
        .filter(|c| !nines.contains(c) && !stock.contains(c))
        .collect();
    let mut deck = vec![];
    for i in 0..7 {
        deck.push(nines[i].clone());
        deck.push(remaining[i].clone());
        deck.push(remaining[i + 7].clone());
    }
    deck.extend(stock);
    let s = state(initial.deal_cards(deck.clone(), &p)?);
    assert_eq!(s.dealer_bonus, 60);
    assert_eq!(s.ledger.get("p0"), 900);
    let s = apply(s, "p1", Action::FourNines, &p)?;
    assert_eq!(s.ledger.get("p0"), 880);
    assert_eq!(s.dealer, 0);
    assert_eq!(s.archives[0].status, "cancelled");
    verify::record(&s.archives[0], &deck, &s.settings, Some("p1"))?;
    let s = state(s.deal_cards(deck, &p)?);
    assert_eq!(s.deal, 2);
    assert_eq!(s.dealer, 0);
    assert_eq!(s.archives.len(), 1);
    Ok(())
}
#[test]
fn late_four_nines_cancels_partial_trick_and_invalid_claim_is_rejected() -> TestResult {
    let (p, mut s, deck) = dealt(3)?;
    assert!(s.clone().apply("p2", Action::FourNines, &p, 0).is_err());
    // A fixture rearrangement gives p0 all nines while preserving 24 cards.
    for face in ["9C", "9D", "9H", "9S"] {
        let card: Card = face.parse()?;
        let owner = s
            .hands
            .iter()
            .find(|(_, h)| h.contains(&card))
            .map(|(p, _)| p.clone());
        if let Some(owner) = owner
            && owner != "p0"
        {
            s.take(&owner, &card)?;
            let other = s
                .hands
                .get("p0")
                .ok_or("hand")?
                .iter()
                .find(|c| c.face() != "9")
                .cloned()
                .ok_or("card")?;
            s.take("p0", &other)?;
            s.hands.get_mut(&owner).ok_or("hand")?.push(other);
            s.hands.get_mut("p0").ok_or("hand")?.push(card);
        }
    }
    s.phase = Phase::Trick {
        turn: 1,
        plays: vec![],
    };
    s.contractor = Some(1);
    let card = s.hands["p1"][0].clone();
    let s = apply(
        s,
        "p1",
        Action::Play {
            card,
            announce: false,
        },
        &p,
    )?;
    let s = apply(s, "p0", Action::FourNines, &p)?;
    assert!(matches!(s.phase, Phase::AwaitShuffle));
    assert_eq!(s.archives[0].status, "cancelled");
    assert_eq!(s.archives[0].movements.len(), deck.len() + 1);
    Ok(())
}
#[test]
fn third_stick_penalty_only_and_dealer_keeps_bonus() -> TestResult {
    let (p, mut s, deck) = dealt(4)?;
    s.sticks.insert("p0".into(), 2);
    s.sticks_before = s.sticks.clone();
    s = apply(s, "p2", Action::Pass, &p)?;
    s = apply(s, "p3", Action::Here, &p)?;
    s = apply(s, "p1", Action::Bid { amount: 110 }, &p)?;
    s = apply(s, "p3", Action::Pass, &p)?;
    // Use contractor p1 with two prior sticks for the third-stick boundary.
    s.sticks.insert("p1".into(), 2);
    s.sticks_before = s.sticks.clone();
    let bonus = s.dealer_bonus;
    s = apply(s, "p1", Action::Dismiss, &p)?;
    assert_eq!(s.ledger.get("p1"), -150);
    assert_eq!(s.ledger.get("p2"), 50);
    assert_eq!(s.ledger.get("p3"), 50);
    assert_eq!(s.ledger.get("p0"), i64::from(bonus));
    verify::record(&s.archives[0], &deck, &s.settings, None)?;
    Ok(())
}
#[test]
fn settings_validate_barrel_floor_and_card_points() -> TestResult {
    for t in [620, 625, 800, 900] {
        Settings {
            player_count: 3,
            barrel_threshold: t,
        }
        .validate()?;
    }
    for t in [615, 621, 905] {
        assert!(
            Settings {
                player_count: 3,
                barrel_threshold: t
            }
            .validate()
            .is_err()
        );
    }
    assert_eq!(
        crate::model::thousand_deck()
            .iter()
            .map(points)
            .sum::<u16>(),
        140
    );
    let p = players(3);
    let mut s = State::new(
        &p,
        Settings {
            player_count: 3,
            barrel_threshold: 800,
        },
        0,
    )?;
    s.ledger.totals.insert("p1".into(), 800);
    s.ledger.totals.insert("p2".into(), 800);
    let s = state(s.deal_cards(crate::model::thousand_deck(), &p)?);
    assert_eq!(s.order, 200);
    Ok(())
}
#[test]
fn attainable_380_and_full_match_through_barrel() -> TestResult {
    let p = players(3);
    let mut s = State::new(&p, Settings::default(), 0)?;
    let own = cards("AH 10H KH QH KD QD KC QC")?;
    let left = cards("9H AD JD 9D AS 10S KS QS")?;
    let right = cards("JH 10D AC 10C JC 9C JS 9S")?;
    let sequence = [
        ("AH", "9H", "JH"),
        ("QH", "AD", "10D"),
        ("KH", "JD", "AC"),
        ("10H", "9D", "10C"),
        ("QD", "AS", "JS"),
        ("KD", "10S", "9S"),
        ("QC", "KS", "JC"),
        ("KC", "QS", "9C"),
    ];
    for round in 0..4 {
        let active: Vec<_> = (1..=3).map(|i| (s.dealer + i) % 3).collect();
        let owner_pos = active.iter().position(|i| *i == 1).ok_or("owner")?;
        let ids = [
            "p1",
            p[active[(owner_pos + 1) % 3]].id.as_str(),
            p[active[(owner_pos + 2) % 3]].id.as_str(),
        ];
        let hands = BTreeMap::from([
            (ids[0].to_owned(), own[..7].to_vec()),
            (ids[1].to_owned(), left[..7].to_vec()),
            (ids[2].to_owned(), right[..7].to_vec()),
        ]);
        let mut deck = vec![];
        for ((a, b), c) in hands[&p[active[0]].id]
            .iter()
            .zip(&hands[&p[active[1]].id])
            .zip(&hands[&p[active[2]].id])
        {
            deck.extend([a.clone(), b.clone(), c.clone()]);
        }
        deck.extend([own[7].clone(), left[7].clone(), right[7].clone()]);
        s = state(s.deal_cards(deck.clone(), &p)?);
        while let Phase::Auction(a) = s.phase.clone() {
            let id = p[a.turn].id.clone();
            let action = if id == "p1" {
                Action::Bid {
                    amount: s.order + 10,
                }
            } else {
                Action::Pass
            };
            s = apply(s, &id, action, &p)?;
        }
        assert_eq!(s.contractor, Some(1));
        s = apply(s, "p1", Action::TakeStock, &p)?;
        s = apply(
            s,
            "p1",
            Action::Distribute {
                transfers: vec![
                    Transfer {
                        player: ids[1].into(),
                        card: left[7].clone(),
                        public: false,
                    },
                    Transfer {
                        player: ids[2].into(),
                        card: right[7].clone(),
                        public: false,
                    },
                ],
            },
            &p,
        )?;
        let order = if round < 3 { 380 } else { 100 };
        s = apply(s, "p1", Action::Order { amount: order }, &p)?;
        for (i, (a, b, c)) in sequence.iter().enumerate() {
            for (id, code) in ids.iter().zip([a, b, c]) {
                let card: Card = code.parse()?;
                s = apply(
                    s,
                    id,
                    Action::Play {
                        announce: *id == "p1" && [1, 4, 6].contains(&i),
                        card,
                    },
                    &p,
                )?;
            }
            let Phase::Closing { close_at, .. } = s.phase else {
                return Err("closing".into());
            };
            s = state(s.close(&p, close_at)?);
        }
        assert_eq!(s.raw["p1"], 380);
        verify::record(s.archives.last().ok_or("record")?, &deck, &s.settings, None)?;
        assert_eq!(s.ledger.get("p1"), [380, 760, 900, 1000][round]);
    }
    assert!(matches!(s.phase, Phase::Finished));
    Ok(())
}

#[test]
fn marriage_is_optional_and_counts_when_announcing_trick_is_lost() -> TestResult {
    let (p, mut s, _) = dealt(3)?;
    s.hands.insert("p1".into(), cards("QH KH")?);
    s.hands.insert("p2".into(), cards("AH")?);
    s.hands.insert("p0".into(), cards("9H")?);
    s.tricks.push(Trick {
        number: 1,
        plays: vec![],
        trump: None,
        winner: "p1".into(),
        points: 0,
    });
    s.phase = Phase::Trick {
        turn: 1,
        plays: vec![],
    };
    assert_eq!(s.view("p0", &p, 0).trick_leader.as_deref(), Some("p1"));
    assert_eq!(s.view("p0", &p, 0).trick_winner, None);
    let silent = apply(
        s.clone(),
        "p1",
        Action::Play {
            card: "QH".parse()?,
            announce: false,
        },
        &p,
    )?;
    assert_eq!(silent.trump, None);
    assert_eq!(silent.raw["p1"], 0);
    let s = apply(
        s,
        "p1",
        Action::Play {
            card: "QH".parse()?,
            announce: true,
        },
        &p,
    )?;
    let s = apply(
        s,
        "p2",
        Action::Play {
            card: "AH".parse()?,
            announce: false,
        },
        &p,
    )?;
    let s = apply(
        s,
        "p0",
        Action::Play {
            card: "9H".parse()?,
            announce: false,
        },
        &p,
    )?;
    let Phase::Closing { trick, .. } = &s.phase else {
        return Err("closing".into());
    };
    assert_eq!(trick.winner, "p2");
    assert_eq!(s.raw["p1"], 100);
    assert_eq!(s.trump, Some(Suit::H));
    let view = s.view("p0", &p, 0);
    assert_eq!(view.trick_leader.as_deref(), Some("p1"));
    assert_eq!(view.trick_winner.as_deref(), Some("p2"));
    assert_eq!(view.announcing_card, Some("QH".parse()?));
    assert_eq!(silent.view("p0", &p, 0).announcing_card, None);
    Ok(())
}
#[test]
fn failed_barrel_contract_exits_and_barrel_defenders_gain_nothing() -> TestResult {
    let p = players(3);
    let mut initial = State::new(
        &p,
        Settings {
            player_count: 3,
            barrel_threshold: 620,
        },
        0,
    )?;
    initial.ledger.totals.insert("p1".into(), 620);
    initial.ledger.totals.insert("p2".into(), 620);
    let deck = crate::model::thousand_deck();
    let s = state(initial.deal_cards(deck.clone(), &p)?);
    let s = play_deal(prepare(s, &p)?, &p)?;
    assert_eq!(s.ledger.get("p1"), 240);
    assert_eq!(s.ledger.get("p2"), 620);
    assert!(!s.view("p1", &p, 0).seats[1].on_barrel);
    verify::record(&s.archives[0], &deck, &s.settings, None)?;
    Ok(())
}
#[test]
fn audit_rejects_teleported_cards_and_false_trick_winners() -> TestResult {
    let (p, s, deck) = dealt(3)?;
    let s = play_deal(prepare(s, &p)?, &p)?;
    let mut record = s.archives[0].clone();
    record.movements[25].from = Zone::Deck;
    assert!(verify::record(&record, &deck, &s.settings, None).is_err());
    let mut record = s.archives[0].clone();
    record.tricks[0].winner = "invented".into();
    assert!(verify::record(&record, &deck, &s.settings, None).is_err());
    Ok(())
}
