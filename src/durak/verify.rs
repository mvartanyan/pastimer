use super::{Action as Move, Record, Settings, State};
use crate::{
    crypto::hash,
    error::{Error, Result, ensure},
    game::{Outcome, PublicEvent},
    model::{Action, DeckProof, GameId, MoveProof, Player},
};
use serde_json::{Value, json};
use std::collections::{BTreeSet, VecDeque};
fn check(value: bool, message: &str) -> Result<()> {
    ensure(value, Error::bad(message))
}
pub fn audit(audit: &Value) -> Result<Value> {
    let game: GameId = serde_json::from_value(audit["game"].clone())?;
    let variant = game
        .durak_variant()
        .ok_or_else(|| Error::bad("Invalid Durak audit"))?;
    let settings: Settings = serde_json::from_value(audit["options"].clone())?;
    settings.validate()?;
    let players: Vec<Player> = serde_json::from_value(audit["players"].clone())?;
    let events = audit["events"]
        .as_array()
        .ok_or_else(|| Error::bad("Missing event log"))?;
    let mut previous = "0".repeat(64);
    let mut hashes = BTreeSet::new();
    for (i, event) in events.iter().enumerate() {
        let mut body = event
            .as_object()
            .cloned()
            .ok_or_else(|| Error::bad("Invalid event"))?;
        let digest = body
            .remove("hash")
            .ok_or_else(|| Error::bad("Missing hash"))?;
        check(
            event["seq"] == i + 1
                && event["roomId"] == audit["roomId"]
                && event["previousHash"] == previous
                && digest == hash(&body)?,
            "Invalid event chain",
        )?;
        previous = digest
            .as_str()
            .ok_or_else(|| Error::bad("Invalid hash"))?
            .into();
        hashes.insert(previous.clone());
    }
    let creation = events
        .first()
        .ok_or_else(|| Error::bad("Missing room creation"))?;
    check(
        creation["type"] == "room.created"
            && creation["data"]["game"] == audit["game"]
            && creation["data"]["gameVersion"] == 1
            && creation["data"]["options"] == audit["options"],
        "Invalid room creation",
    )?;
    let host: Player = serde_json::from_value(creation["data"]["player"].clone())?;
    let mut roster = vec![host.clone()];
    for event in events.iter().filter(|e| e["type"] == "player.joined") {
        roster.push(serde_json::from_value(event["data"]["player"].clone())?);
    }
    check(
        roster == players
            && players.iter().map(|p| &p.id).collect::<BTreeSet<_>>().len() == players.len(),
        "Invalid seating roster",
    )?;
    let receipts = audit
        .get("receipts")
        .map(|v| v.as_array().ok_or_else(|| Error::bad("Invalid receipts")))
        .transpose()?
        .cloned()
        .unwrap_or_default();
    for r in &receipts {
        check(
            r["roomId"] == audit["roomId"]
                && r["eventHash"].as_str().is_some_and(|h| hashes.contains(h)),
            "Saved receipt does not match history",
        )?;
    }
    let records: Vec<Record> = serde_json::from_value(audit["deals"].clone())?;
    let commands: Vec<MoveProof> = serde_json::from_value(audit["commands"].clone())?;
    let decks = audit["decks"]
        .as_array()
        .ok_or_else(|| Error::bad("Missing decks"))?;
    let commits: Vec<_> = events
        .iter()
        .filter(|e| e["type"] == "deck.committed")
        .collect();
    check(
        decks.len() == commits.len()
            && decks.len() >= records.len()
            && decks.len() <= records.len() + 1,
        "Missing deck commitments",
    )?;
    check(
        match audit["phase"].as_str() {
            Some("finished") => !records.is_empty() && decks.len() == records.len(),
            Some("playing") => decks.len() == records.len() + 1,
            _ => false,
        },
        "Game phase does not match disclosed deals",
    )?;
    let mut proofs = vec![];
    for (i, deck) in decks.iter().enumerate() {
        check(
            deck["shuffleId"] == i + 1
                && commits[i]["data"]["shuffleId"] == i + 1
                && commits[i]["data"]["count"] == 36
                && deck["commitment"] == commits[i]["data"]["commitment"],
            "Wrong deck identity",
        )?;
        if i < records.len() {
            let proof: DeckProof = serde_json::from_value(deck["proof"].clone())?;
            check(
                proof.room_id == audit["roomId"].as_str().unwrap_or("")
                    && proof.shuffle_id as usize == i + 1
                    && proof.protocol == "pastimer-deck-v1"
                    && deck["commitment"] == hash(&proof)?,
                "Deck commitment mismatch",
            )?;
            proofs.push(proof);
        } else {
            check(
                deck.get("proof").is_none(),
                "Unfinished game must not reveal its deck",
            )?;
        }
    }
    let moves: Vec<_> = events
        .iter()
        .filter(|e| {
            e["type"] == "move.committed"
                && e["data"]["deal"]
                    .as_u64()
                    .is_some_and(|n| n <= records.len() as u64)
        })
        .collect();
    check(moves.len() == commands.len(), "Missing move proofs")?;
    let mut version = 0;
    for (command, event) in commands.iter().zip(&moves) {
        check(
            event["data"]["commitment"] == hash(command)?
                && event["data"]["deal"] == serde_json::to_value(command.deal_id)?
                && event["actor"] == command.actor
                && command.version > version,
            "Move commitment mismatch",
        )?;
        version = command.version;
    }
    check(
        events
            .iter()
            .filter(|e| e["type"] == "durak.settled")
            .count()
            == records.len()
            && events
                .iter()
                .filter(|e| e["type"] == "game.finished")
                .count()
                == records.len(),
        "Missing settled games",
    )?;
    if records.is_empty() {
        check(
            commands.is_empty(),
            "Unfinished commands must remain sealed",
        )?;
    } else {
        let dealer = events
            .iter()
            .find(|e| e["type"] == "dealer.selected")
            .and_then(|e| players.iter().position(|p| e["data"]["player"] == p.id))
            .ok_or_else(|| Error::bad("Missing dealer"))?;
        let mut state = State::new(&players, settings, variant, dealer)?;
        let mut expected: VecDeque<(String, PublicEvent)> = VecDeque::new();
        let mut pending = true;
        let mut command_index = 0;
        let mut settled = 0;
        let mut last_actor = host.id.clone();
        let mut started = false;
        let mut selected = false;
        fn unpack(
            out: Outcome<State>,
            actor: &str,
            expected: &mut VecDeque<(String, PublicEvent)>,
        ) -> (State, bool) {
            let (state, pending, events) = match out {
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
                Some("room.created" | "player.joined") => {
                    check(!started, "Late setup event")?;
                }
                Some("game.started") => {
                    check(
                        event["actor"] == host.id
                            && expected.is_empty()
                            && (state.deal == 0 || pending),
                        "Invalid game start",
                    )?;
                    started = true;
                }
                Some("dealer.selected") => {
                    check(
                        started
                            && !selected
                            && state.deal == 0
                            && event["actor"] == host.id
                            && event["data"]["player"] == players[dealer].id,
                        "Invalid dealer selection",
                    )?;
                    selected = true;
                }
                Some("deck.committed") => {
                    check(
                        started
                            && selected
                            && pending
                            && expected.is_empty()
                            && event["actor"] == last_actor,
                        "Unexpected deck",
                    )?;
                    let proof = proofs
                        .get(state.deal as usize)
                        .ok_or_else(|| Error::bad("Missing replay deck"))?;
                    (state, pending) = unpack(
                        state.deal_cards(proof.cards.clone(), &players)?,
                        &last_actor,
                        &mut expected,
                    );
                }
                Some("move.committed") => {
                    check(!pending && expected.is_empty(), "Missing rule events")?;
                    let proof = commands
                        .get(command_index)
                        .ok_or_else(|| Error::bad("Missing replay move"))?;
                    command_index += 1;
                    let Action::Durak { action } = &proof.action else {
                        return Err(Error::bad("Wrong game command"));
                    };
                    check(
                        proof.deal_id
                            == Some(state.deal + u32::from(matches!(action, Move::Rematch)))
                            && (!matches!(action, Move::Rematch) || proof.actor == host.id),
                        "Wrong command game or actor",
                    )?;
                    last_actor = proof.actor.clone();
                    (state, pending) = unpack(
                        state
                            .apply(&proof.actor, action.clone(), &players)
                            .map_err(|_| Error::bad("Illegal Durak action in audit replay"))?,
                        &last_actor,
                        &mut expected,
                    );
                }
                Some("game.finished") => {
                    check(
                        expected.is_empty()
                            && state.finished
                            && event["actor"] == last_actor
                            && event["data"]["scores"] == json!(state.ledger.totals),
                        "Invalid game result",
                    )?;
                    settled += 1;
                    check(
                        state.archives == records[..settled],
                        "Game record does not match replay",
                    )?;
                    if settled == records.len() {
                        break;
                    }
                }
                _ => {
                    let (actor, rule) = expected
                        .pop_front()
                        .ok_or_else(|| Error::bad("Unexpected rule event"))?;
                    check(
                        event["type"] == rule.kind
                            && event["actor"] == actor
                            && event["data"] == rule.data,
                        "Rule event does not match replay",
                    )?;
                }
            }
        }
        check(
            settled == records.len() && command_index == commands.len() && expected.is_empty(),
            "Incomplete replay",
        )?;
    }
    Ok(
        json!({"valid":true,"events":events.len(),"deals":records.len(),"decks":proofs.len(),"moves":commands.len(),"anchoredReceipts":receipts.len()}),
    )
}
