use super::*;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn players(n: usize) -> Vec<Player> {
    (0..n)
        .map(|i| Player {
            id: format!("p{i}"),
            name: format!("Player {i}"),
        })
        .collect()
}
fn card(code: &str) -> Card {
    code.parse().unwrap_or_else(|_| unreachable!("test card"))
}
fn state(out: Outcome<State>) -> State {
    match out {
        Outcome::Continue { state, .. }
        | Outcome::Finished { state, .. }
        | Outcome::Shuffle { state, .. } => state,
    }
}
fn fixture(variant: Variant, hands: &[&[&str]]) -> State {
    let p = players(hands.len());
    let mut s = State::new(
        &p,
        Settings {
            player_count: p.len() as u8,
        },
        variant,
        0,
    )
    .unwrap_or_else(|_| unreachable!());
    s.deal = 1;
    s.trump = Some(card("AS"));
    s.hands = p
        .iter()
        .zip(hands)
        .map(|(p, h)| (p.id.clone(), h.iter().map(|c| card(c)).collect()))
        .collect();
    let held: BTreeSet<_> = s.hands.values().flatten().cloned().collect();
    s.stock = deck().into_iter().filter(|c| !held.contains(c)).collect();
    s.first_attacker = "p0".into();
    s.new_bout("p0".into(), &p)
        .unwrap_or_else(|_| unreachable!());
    s
}
fn attack(s: State, actor: &str, c: &str, p: &[Player]) -> Result<State> {
    Ok(state(s.apply(
        actor,
        Action::Attack {
            cards: vec![card(c)],
        },
        p,
    )?))
}
#[test]
fn requests_are_per_player_and_rank_and_ignored_requests_do_not_block() -> TestResult {
    let p = players(4);
    let mut s = fixture(
        Variant::Podkidnoy,
        &[
            &["6C", "8H"],
            &["9C", "10D", "JH", "QD", "KC", "AD"],
            &["9H", "JC"],
            &["9D", "JS"],
        ],
    );
    s = attack(s, "p0", "6C", &p)?;
    s = state(s.apply(
        "p1",
        Action::Defend {
            attack: card("6C"),
            card: card("9C"),
        },
        &p,
    )?);
    assert!(
        s.clone()
            .apply(
                "p2",
                Action::Attack {
                    cards: vec![card("9H")]
                },
                &p
            )
            .is_err()
    );
    s = state(s.apply("p2", Action::Request { rank: "9".into() }, &p)?);
    assert!(s.view("p3", &p).requests.is_empty());
    assert_eq!(s.view("p0", &p).requests.len(), 1);
    assert!(
        s.clone()
            .apply(
                "p3",
                Action::Approve {
                    player: "p2".into(),
                    rank: "9".into()
                },
                &p
            )
            .is_err()
    );
    s = state(s.apply(
        "p0",
        Action::Approve {
            player: "p2".into(),
            rank: "9".into(),
        },
        &p,
    )?);
    assert!(s.view("p2", &p).controls.attack_cards.is_empty());
    s = state(s.apply("p0", Action::Pass, &p)?);
    assert_eq!(s.turn(&p).as_deref(), Some("p2"));
    s = attack(s, "p2", "9H", &p)?;
    assert!(
        s.clone()
            .apply(
                "p1",
                Action::Defend {
                    attack: card("9H"),
                    card: card("AD")
                },
                &p
            )
            .is_err()
    );
    // Independently verify that ignoring a request closes a fully covered attack.
    let mut s = fixture(
        Variant::Podkidnoy,
        &[&["6C"], &["9C", "10D"], &["9H"], &["9D"]],
    );
    s = attack(s, "p0", "6C", &p)?;
    s = state(s.apply(
        "p1",
        Action::Defend {
            attack: card("6C"),
            card: card("9C"),
        },
        &p,
    )?);
    s = state(s.apply("p2", Action::Request { rank: "9".into() }, &p)?);
    s = state(s.apply("p0", Action::Pass, &p)?);
    assert_eq!(s.bouts.len(), 1);
    assert!(s.requests.is_empty());
    Ok(())
}
#[test]
fn new_rank_needs_new_approval_and_restores_attacker_priority() -> TestResult {
    let p = players(3);
    let mut s = fixture(
        Variant::Podkidnoy,
        &[&["6C", "6D"], &["9C", "JH", "QD", "KC"], &["9H", "JC"]],
    );
    s = attack(s, "p0", "6C", &p)?;
    s = state(s.apply(
        "p1",
        Action::Defend {
            attack: card("6C"),
            card: card("9C"),
        },
        &p,
    )?);
    s = state(s.apply("p2", Action::Request { rank: "9".into() }, &p)?);
    s = state(s.apply(
        "p0",
        Action::Approve {
            player: "p2".into(),
            rank: "9".into(),
        },
        &p,
    )?);
    s = state(s.apply("p0", Action::Pass, &p)?);
    s = attack(s, "p2", "9H", &p)?;
    s = state(s.apply(
        "p1",
        Action::Defend {
            attack: card("9H"),
            card: card("JH"),
        },
        &p,
    )?);
    assert_eq!(s.turn(&p).as_deref(), Some("p0"));
    assert!(
        s.view("p2", &p)
            .controls
            .request_ranks
            .contains(&"J".into())
    );
    assert!(!s.approvals.contains(&Permission {
        player: "p2".into(),
        rank: "J".into()
    }));
    assert!(s.approvals.contains(&Permission {
        player: "p2".into(),
        rank: "9".into()
    }));
    Ok(())
}
#[test]
fn transfer_chain_changes_attacker_and_clears_permissions_but_preserves_draw_order() -> TestResult {
    let p = players(4);
    let mut s = fixture(
        Variant::Perevodnoy,
        &[
            &["6C", "AC", "KC", "QC"],
            &["6D", "AD", "KD", "QD"],
            &["6H", "AH", "KH", "QH"],
            &["6S", "AS", "KS", "QS"],
        ],
    );
    s = attack(s, "p0", "6C", &p)?;
    s = state(s.apply("p2", Action::Request { rank: "6".into() }, &p)?);
    s = state(s.apply("p1", Action::Transfer { card: card("6D") }, &p)?);
    assert_eq!(s.attacker, "p1");
    assert!(s.requests.is_empty());
    s = state(s.apply("p2", Action::Transfer { card: card("6H") }, &p)?);
    assert_eq!(s.defender, "p3");
    assert_eq!(s.original_attacker, "p0");
    s = state(s.apply("p3", Action::Take, &p)?);
    s = state(s.apply("p2", Action::Pass, &p)?);
    assert_eq!(s.bouts.len(), 1);
    let drawn: Vec<_> = s
        .movements
        .iter()
        .filter(|m| m.from == "stock")
        .map(|m| m.to.as_str())
        .collect();
    assert_eq!(drawn, vec!["p0", "p1", "p2"]);
    assert_eq!(s.attacker, "p0");
    Ok(())
}
#[test]
fn transfer_restrictions_and_fixed_pickup_cap() -> TestResult {
    let p = players(3);
    let s = fixture(
        Variant::Podkidnoy,
        &[&["6C", "9C"], &["6D", "10C"], &["6H"]],
    );
    let s = attack(s, "p0", "6C", &p)?;
    assert!(
        s.clone()
            .apply("p1", Action::Transfer { card: card("6D") }, &p)
            .is_err()
    );
    let mut s = s;
    s.variant = Variant::Perevodnoy;
    assert!(
        s.clone()
            .apply("p1", Action::Transfer { card: card("6D") }, &p)
            .is_err()
    );
    s = state(s.apply(
        "p1",
        Action::Defend {
            attack: card("6C"),
            card: card("10C"),
        },
        &p,
    )?);
    assert!(
        s.clone()
            .apply("p1", Action::Transfer { card: card("6D") }, &p)
            .is_err()
    );
    s = state(s.apply("p1", Action::Take, &p)?);
    assert_eq!(s.limit, 2);
    assert!(
        s.clone()
            .apply(
                "p1",
                Action::Defend {
                    attack: card("6C"),
                    card: card("6D")
                },
                &p
            )
            .is_err()
    );
    Ok(())
}
#[test]
fn lowest_trump_six_player_deal_and_rematch_attack_loser() -> TestResult {
    for n in 2..=6 {
        let p = players(n);
        let s = State::new(
            &p,
            Settings {
                player_count: n as u8,
            },
            Variant::Perevodnoy,
            n - 1,
        )?;
        let s = state(s.deal_cards(deck(), &p)?);
        assert!(s.hands.values().all(|h| h.len() == 6));
        assert_eq!(s.stock.len(), 36 - n * 6);
        if n == 6 {
            assert!(
                s.hand(&p[n - 1].id)
                    .contains(s.trump.as_ref().ok_or("trump")?)
            );
        }
        let mut s = s;
        s.finished = true;
        s.loser = Some("p1".into());
        let s = state(s.apply("p0", Action::Rematch, &p)?);
        let s = state(s.deal_cards(deck(), &p)?);
        assert_eq!(s.attacker, "p0");
    }
    Ok(())
}
#[test]
fn full_games_conserve_cards_and_computer_views_do_not_include_other_hands() -> TestResult {
    for variant in [Variant::Podkidnoy, Variant::Perevodnoy] {
        for n in 2..=6 {
            let p = players(n);
            let mut cards = deck();
            crate::crypto::shuffle(&mut cards);
            let mut s = state(
                State::new(
                    &p,
                    Settings {
                        player_count: n as u8,
                    },
                    variant,
                    0,
                )?
                .deal_cards(cards, &p)?,
            );
            for _ in 0..10000 {
                let locations: Vec<_> = s
                    .hands
                    .values()
                    .flatten()
                    .chain(s.stock.iter())
                    .chain(s.discard.iter())
                    .chain(
                        s.pairs
                            .iter()
                            .flat_map(|p| std::iter::once(&p.card).chain(p.cover.iter())),
                    )
                    .collect();
                // Finished bout cards remain in the public presentation, but have been collected.
                if !s.finished {
                    assert_eq!(locations.len(), 36);
                    assert_eq!(locations.into_iter().collect::<BTreeSet<_>>().len(), 36);
                }
                if s.finished {
                    break;
                }
                let action = p
                    .iter()
                    .find_map(|p| choose(s.view(&p.id, &players(n))).map(|a| (p.id.clone(), a)))
                    .ok_or("no legal computer move")?;
                s = state(s.apply(&action.0, action.1, &p)?);
            }
            assert!(s.finished, "variant {variant:?}, {n} players");
            assert_eq!(s.archives.len(), 1);
        }
    }
    Ok(())
}

#[test]
fn six_card_cap_includes_first_attack_and_pickup_throw_ins() -> TestResult {
    let p = players(2);
    let mut s = fixture(
        Variant::Podkidnoy,
        &[
            &["6C", "6D", "6H", "6S", "9D", "9H", "9S"],
            &["9C", "10C", "JC", "QC", "KC", "AC", "7D", "8D"],
        ],
    );
    assert_eq!(s.limit, 6);
    s = state(s.apply(
        "p0",
        Action::Attack {
            cards: vec![card("6C"), card("6D"), card("6H"), card("6S")],
        },
        &p,
    )?);
    s = state(s.apply(
        "p1",
        Action::Defend {
            attack: card("6C"),
            card: card("9C"),
        },
        &p,
    )?);
    s = state(s.apply("p1", Action::Take, &p)?);
    assert!(
        s.clone()
            .apply(
                "p0",
                Action::Attack {
                    cards: vec![card("9D"), card("9H"), card("9S")]
                },
                &p
            )
            .is_err()
    );
    s = state(s.apply(
        "p0",
        Action::Attack {
            cards: vec![card("9D"), card("9H")],
        },
        &p,
    )?);
    assert_eq!(s.bouts.len(), 1);
    assert!(s.bouts[0].taken);
    assert_eq!(s.bouts[0].pairs.len(), 6);
    assert_eq!(s.hand("p1").len(), 14);
    Ok(())
}
#[test]
fn final_two_finish_together_and_trump_free_deal_fallback() -> TestResult {
    let p = players(2);
    let mut s = fixture(Variant::Podkidnoy, &[&["6C"], &["7C"]]);
    s.stock.clear();
    s = attack(s, "p0", "6C", &p)?;
    s = state(s.apply(
        "p1",
        Action::Defend {
            attack: card("6C"),
            card: card("7C"),
        },
        &p,
    )?);
    assert!(s.finished);
    assert!(s.loser.is_none());
    let s = State::new(&p, Settings { player_count: 2 }, Variant::Podkidnoy, 0)?;
    // First 12 cards are clubs/diamonds; the next (exposed) card is a heart.
    let mut cards = deck();
    let heart = cards.iter().position(|c| c.code() == "6H").ok_or("heart")?;
    cards.swap(12, heart);
    let s = state(s.deal_cards(cards, &p)?);
    assert_eq!(s.attacker, "p1");
    Ok(())
}
#[test]
fn helper_opportunities_are_clockwise_not_request_or_approval_order() -> TestResult {
    let p = players(4);
    let mut s = fixture(
        Variant::Podkidnoy,
        &[&["6C"], &["9C", "7D", "8D"], &["9H"], &["9D"]],
    );
    s = attack(s, "p0", "6C", &p)?;
    s = state(s.apply(
        "p1",
        Action::Defend {
            attack: card("6C"),
            card: card("9C"),
        },
        &p,
    )?);
    for id in ["p3", "p2"] {
        s = state(s.apply(id, Action::Request { rank: "9".into() }, &p)?);
        s = state(s.apply(
            "p0",
            Action::Approve {
                player: id.into(),
                rank: "9".into(),
            },
            &p,
        )?);
    }
    s = state(s.apply("p0", Action::Pass, &p)?);
    assert_eq!(s.turn(&p).as_deref(), Some("p2"));
    assert!(
        s.clone()
            .apply(
                "p3",
                Action::Attack {
                    cards: vec![card("9D")]
                },
                &p
            )
            .is_err()
    );
    s = state(s.apply("p2", Action::Pass, &p)?);
    assert_eq!(s.turn(&p).as_deref(), Some("p3"));
    Ok(())
}
