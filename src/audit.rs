use crate::{
    crypto::hash,
    error::{Error, Result, ensure},
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
fn check(condition: bool, message: &str) -> Result<()> {
    ensure(condition, Error::bad(message))
}
pub fn verify(audit: &Value) -> Result<Value> {
    if audit["protocol"] == "pastimer-durak-audit-v1" {
        return crate::durak::verify::audit(audit);
    }
    if audit["protocol"] == "pastimer-audit-v2" {
        return verify_thousand(audit);
    }
    check(
        audit["protocol"] == "pastimer-audit-v1" && audit["phase"] == "finished",
        "A completed game is required",
    )?;
    let events = audit["events"]
        .as_array()
        .ok_or_else(|| Error::bad("Missing event log"))?;
    let mut previous = "0".repeat(64);
    let mut hashes = BTreeSet::new();
    for (index, event) in events.iter().enumerate() {
        let mut body = event
            .as_object()
            .cloned()
            .ok_or_else(|| Error::bad("Invalid event"))?;
        let digest = body
            .remove("hash")
            .and_then(|v| v.as_str().map(str::to_owned))
            .ok_or_else(|| Error::bad("Missing event hash"))?;
        check(
            event["seq"] == index + 1
                && event["roomId"] == audit["roomId"]
                && event["previousHash"] == previous
                && hash(&body)? == digest,
            "Invalid event chain",
        )?;
        hashes.insert(digest.clone());
        previous = digest;
    }
    check(
        events.first().is_some_and(|e| e["type"] == "room.created")
            && events.iter().any(|e| e["type"] == "game.finished"),
        "Incomplete game log",
    )?;
    let receipts = match audit.get("receipts") {
        Some(value) => value
            .as_array()
            .ok_or_else(|| Error::bad("Invalid receipts"))?
            .clone(),
        None => vec![],
    };
    for receipt in &receipts {
        check(
            receipt["roomId"] == audit["roomId"]
                && receipt["eventHash"]
                    .as_str()
                    .is_some_and(|h| hashes.contains(h)),
            "Saved receipt does not match history",
        )?;
    }
    let commits: Vec<_> = events
        .iter()
        .filter(|e| e["type"] == "deck.committed")
        .collect();
    let decks = audit["decks"]
        .as_array()
        .ok_or_else(|| Error::bad("Missing deck proof"))?;
    check(commits.len() == decks.len(), "Missing deck proof")?;
    for (index, deck) in decks.iter().enumerate() {
        let proof = &deck["proof"];
        let data = &commits[index]["data"];
        check(
            deck["shuffleId"] == index + 1
                && proof["shuffleId"] == deck["shuffleId"]
                && proof["roomId"] == audit["roomId"]
                && proof["protocol"] == "pastimer-deck-v1",
            "Wrong deck identity",
        )?;
        let cards = proof["cards"]
            .as_array()
            .ok_or_else(|| Error::bad("Invalid deck cards"))?;
        let unique: Option<BTreeSet<_>> = cards.iter().map(Value::as_str).collect();
        check(
            unique.is_some_and(|u| u.len() == cards.len()) && data["count"] == cards.len(),
            "Invalid deck cards",
        )?;
        check(
            data["shuffleId"] == deck["shuffleId"]
                && data["commitment"] == deck["commitment"]
                && deck["commitment"] == hash(proof)?,
            "Deck commitment mismatch",
        )?;
    }
    let moves: Vec<_> = events
        .iter()
        .filter(|e| e["type"] == "move.committed")
        .collect();
    let commands = audit["commands"]
        .as_array()
        .ok_or_else(|| Error::bad("Missing move proofs"))?;
    check(moves.len() == commands.len(), "Missing move proofs")?;
    for (command, event) in commands.iter().zip(moves.iter()) {
        check(
            event["data"]["commitment"] == hash(command)?,
            "Move commitment mismatch",
        )?;
    }
    Ok(
        json!({"valid":true,"events":events.len(),"decks":decks.len(),"moves":moves.len(),"anchoredReceipts":receipts.len()}),
    )
}

fn verify_thousand(audit: &Value) -> Result<Value> {
    use crate::{
        model::{DeckProof, MoveProof},
        thousand::{DealRecord, Settings},
    };
    let events = audit["events"]
        .as_array()
        .ok_or_else(|| Error::bad("Missing event log"))?;
    let mut previous = "0".repeat(64);
    let mut hashes = BTreeSet::new();
    for (i, e) in events.iter().enumerate() {
        let mut body = e
            .as_object()
            .cloned()
            .ok_or_else(|| Error::bad("Invalid event"))?;
        let digest = body
            .remove("hash")
            .ok_or_else(|| Error::bad("Missing hash"))?;
        check(
            e["seq"] == i + 1
                && e["roomId"] == audit["roomId"]
                && e["previousHash"] == previous
                && digest == hash(&body)?,
            "Invalid event chain",
        )?;
        previous = digest
            .as_str()
            .ok_or_else(|| Error::bad("Invalid hash"))?
            .into();
        hashes.insert(previous.clone());
    }
    check(
        events.first().is_some_and(|e| e["type"] == "room.created") && audit["game"] == "thousand",
        "Invalid Thousand audit",
    )?;
    let settings: Settings = serde_json::from_value(audit["options"].clone())
        .map_err(|_| Error::bad("Invalid settings"))?;
    settings.validate()?;
    check(
        events[0]["data"]["options"] == audit["options"],
        "Settings do not match room creation",
    )?;
    let receipts = match audit.get("receipts") {
        Some(v) => v
            .as_array()
            .ok_or_else(|| Error::bad("Invalid receipts"))?
            .clone(),
        None => vec![],
    };
    for r in &receipts {
        check(
            r["roomId"] == audit["roomId"]
                && r["eventHash"].as_str().is_some_and(|h| hashes.contains(h)),
            "Saved receipt does not match history",
        )?;
    }
    let decks = audit["decks"]
        .as_array()
        .ok_or_else(|| Error::bad("Missing decks"))?;
    let commits: Vec<_> = events
        .iter()
        .filter(|e| e["type"] == "deck.committed")
        .collect();
    check(commits.len() == decks.len(), "Missing deck commitment")?;
    let deals: Vec<DealRecord> = serde_json::from_value(audit["deals"].clone())
        .map_err(|_| Error::bad("Invalid deal records"))?;
    let endings: Vec<_> = events
        .iter()
        .filter(|e| e["type"] == "deal.settled" || e["type"] == "deal.cancelled")
        .collect();
    check(
        endings.len() == deals.len()
            && decks.len() >= deals.len()
            && decks.len() <= deals.len() + 1,
        "Missing deal records",
    )?;
    let commands: Vec<MoveProof> = serde_json::from_value(audit["commands"].clone())
        .map_err(|_| Error::bad("Invalid move proofs"))?;
    let move_events: Vec<_> = events
        .iter()
        .filter(|e| {
            e["type"] == "move.committed"
                && e["data"]["deal"]
                    .as_u64()
                    .is_some_and(|d| d <= deals.len() as u64)
        })
        .collect();
    check(commands.len() == move_events.len(), "Missing move proofs")?;
    for (command, event) in commands.iter().zip(&move_events) {
        check(
            event["data"]["commitment"] == hash(command)?
                && event["data"]["deal"] == serde_json::to_value(command.deal_id)?,
            "Move commitment mismatch",
        )?;
    }
    for (i, deck) in decks.iter().enumerate() {
        check(
            deck["shuffleId"] == i + 1
                && commits[i]["data"]["shuffleId"] == i + 1
                && deck["commitment"] == commits[i]["data"]["commitment"],
            "Wrong deck identity",
        )?;
        if i >= deals.len() {
            check(
                deck.get("proof").is_none(),
                "Unfinished deal must not reveal its deck",
            )?;
            continue;
        }
        let proof: DeckProof = serde_json::from_value(deck["proof"].clone())
            .map_err(|_| Error::bad("Missing deck proof"))?;
        check(
            proof.room_id == audit["roomId"].as_str().unwrap_or("")
                && proof.shuffle_id as usize == i + 1
                && proof.protocol == "pastimer-deck-v1"
                && deck["commitment"] == hash(&proof)?,
            "Deck commitment mismatch",
        )?;
        let record = &deals[i];
        check(
            record.deal as usize == i + 1
                && endings[i]["data"]["deal"] == record.deal
                && endings[i]["data"]["recordHash"] == hash(record)?,
            "Deal record commitment mismatch",
        )?;
        if i > 0 {
            check(
                record.before == deals[i - 1].after
                    && record.sticks_before == deals[i - 1].sticks_after,
                "Deal checkpoints do not connect",
            )?;
        } else {
            check(
                record.before.totals.values().all(|v| *v == 0)
                    && record.sticks_before.values().all(|v| *v == 0),
                "Initial scores must be zero",
            )?;
        }
        let reveal = events
            .iter()
            .position(|e| e == endings[i])
            .ok_or_else(Error::internal)?;
        let commit = events
            .iter()
            .position(|e| e == commits[i])
            .ok_or_else(Error::internal)?;
        check(commit < reveal, "Deck must be committed before settlement")?;
        crate::thousand::verify::record(
            record,
            &proof.cards,
            &settings,
            endings[i]["data"]["player"].as_str(),
        )?;
    }
    replay_thousand(audit, &settings, &deals, &commands)?;
    Ok(
        json!({"valid":true,"events":events.len(),"decks":deals.len(),"moves":commands.len(),"deals":deals.len(),"anchoredReceipts":receipts.len()}),
    )
}

/// Replay the revealed commands and compare every emitted public rule event.
fn replay_thousand(
    audit: &Value,
    settings: &crate::thousand::Settings,
    deals: &[crate::thousand::DealRecord],
    commands: &[crate::model::MoveProof],
) -> Result<()> {
    use crate::{
        game::{Outcome, PublicEvent},
        model::{Action, DeckProof, Player},
        thousand::{Action as Move, State},
    };
    use std::collections::VecDeque;
    let events = audit["events"].as_array().ok_or_else(Error::internal)?;
    let players: Vec<Player> = serde_json::from_value(audit["players"].clone())?;
    let mut joined = vec![serde_json::from_value::<Player>(
        events[0]["data"]["player"].clone(),
    )?];
    let mut computers = BTreeSet::new();
    for event in events.iter().filter(|e| e["type"] == "player.joined") {
        let player: Player = serde_json::from_value(event["data"]["player"].clone())?;
        if event["data"]["computer"] == true {
            computers.insert(player.id.clone());
        }
        joined.push(player);
    }
    check(
        players == joined
            && players.len() == usize::from(settings.player_count)
            && players.iter().map(|p| &p.id).collect::<BTreeSet<_>>().len() == players.len(),
        "Invalid seating roster",
    )?;
    let dealer = events
        .iter()
        .find(|e| e["type"] == "dealer.selected")
        .and_then(|e| players.iter().position(|p| e["data"]["player"] == p.id))
        .ok_or_else(|| Error::bad("Missing first dealer"))?;
    let mut state = State::new(&players, settings.clone(), dealer)?;
    if deals.is_empty() {
        return Ok(());
    }
    let mut pending_deck = false;
    let mut requested_computer = None;
    let mut command_index = 0;
    let mut settled = 0;
    let mut expected: VecDeque<(String, PublicEvent)> = VecDeque::new();
    fn take(
        outcome: Outcome<State>,
        actor: &str,
        expected: &mut VecDeque<(String, PublicEvent)>,
    ) -> (State, bool) {
        let (state, pending, events) = match outcome {
            Outcome::Continue { state, events } | Outcome::Finished { state, events, .. } => {
                (state, false, events)
            }
            Outcome::Shuffle { state, events, .. } => (state, true, events),
        };
        expected.extend(events.into_iter().map(|e| (actor.into(), e)));
        (state, pending)
    }
    for event in events {
        match event["type"].as_str() {
            Some("room.created" | "player.joined" | "game.started" | "dealer.selected") => {
                check(command_index == 0, "Unexpected setup event")?;
                continue;
            }
            Some("computer.deal.requested") => {
                let dealer = &players[state.dealer].id;
                check(
                    expected.is_empty()
                        && !pending_deck
                        && requested_computer.is_none()
                        && matches!(state.phase, crate::thousand::Phase::AwaitShuffle)
                        && computers.contains(dealer)
                        && event["data"]["player"] == *dealer
                        && event["actor"].as_str().is_some_and(|id| {
                            players.iter().any(|p| p.id == id) && !computers.contains(id)
                        }),
                    "Invalid request for a computer dealer",
                )?;
                requested_computer = Some(dealer.clone());
                continue;
            }
            Some("move.committed") => {
                check(expected.is_empty(), "Missing public rule events")?;
                let command = commands
                    .get(command_index)
                    .ok_or_else(|| Error::bad("Missing replay command"))?;
                command_index += 1;
                let Action::Thousand { action } = &command.action else {
                    return Err(Error::bad("Wrong game command"));
                };
                let requested = requested_computer.take();
                check(
                    if computers.contains(&command.actor) && matches!(action, Move::Shuffle) {
                        requested.as_ref() == Some(&command.actor)
                    } else {
                        requested.is_none()
                    },
                    "Computer shuffle must follow a human request",
                )?;
                check(
                    !pending_deck
                        && command.deal_id
                            == Some(state.deal + u32::from(matches!(action, Move::Shuffle))),
                    "Wrong command deal",
                )?;
                (state, pending_deck) = take(
                    state
                        .apply(&command.actor, action.clone(), &players, 0)
                        .map_err(|_| Error::bad("Illegal action in audit replay"))?,
                    &command.actor,
                    &mut expected,
                );
                continue;
            }
            Some("deck.committed") => {
                check(
                    pending_deck && expected.is_empty(),
                    "Unexpected deck in replay",
                )?;
                let index = state.deal as usize;
                let proof: DeckProof =
                    serde_json::from_value(audit["decks"][index]["proof"].clone())?;
                let actor = players[state.dealer].id.clone();
                check(event["actor"] == actor, "Wrong shuffle actor")?;
                (state, pending_deck) = take(
                    state.deal_cards(proof.cards, &players)?,
                    &actor,
                    &mut expected,
                );
                continue;
            }
            Some("trick.closed") => {
                check(expected.is_empty(), "Unexpected trick closure")?;
                (state, pending_deck) = take(
                    state.close(&players, crate::thousand::TRICK_DELAY_MS)?,
                    "system",
                    &mut expected,
                );
            }
            _ => {}
        }
        let (actor, rule_event) = expected
            .pop_front()
            .ok_or_else(|| Error::bad("Unexpected public rule event"))?;
        let mut actual = event["data"].clone();
        let mut expected_data = rule_event.data;
        // Replay uses a logical clock; the live server's persisted deadline is checked by runtime tests.
        if rule_event.kind == "trick.completed" {
            actual
                .as_object_mut()
                .ok_or_else(Error::internal)?
                .remove("closeAt");
            expected_data
                .as_object_mut()
                .ok_or_else(Error::internal)?
                .remove("closeAt");
        }
        check(
            event["type"] == rule_event.kind && event["actor"] == actor && actual == expected_data,
            "Public event does not match rule replay",
        )?;
        if event["type"] == "deal.settled" || event["type"] == "deal.cancelled" {
            let record = deals.get(settled).ok_or_else(Error::internal)?;
            let replayed = state
                .archives
                .last()
                .ok_or_else(|| Error::bad("Missing replay settlement"))?;
            check(
                serde_json::to_value(replayed)? == serde_json::to_value(record)?,
                "Replay does not match deal record",
            )?;
            settled += 1;
            if settled == deals.len() {
                break;
            }
        }
    }
    check(
        command_index == commands.len() && settled == deals.len() && expected.is_empty(),
        "Incomplete replay",
    )
}
