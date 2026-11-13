use crate::{
    crypto::{hash, shuffle, token},
    error::{Error, Result, ensure},
    game::{GameRules, HighCard, HighCardState, Outcome},
    model::*,
    store::Store,
};
use axum::http::StatusCode;
use chrono::{SecondsFormat, Utc};
use rand::{Rng, rngs::OsRng};
use serde_json::{Value, json};
use sqlx::SqliteConnection;
use std::collections::BTreeMap;
use uuid::Uuid;
#[derive(Clone)]
pub struct Engine {
    pub store: Store,
}
impl Engine {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
    fn compatible(room: &Room) -> Result<()> {
        ensure(
            room.game_version == 1,
            Error::conflict("Game version unavailable"),
        )?;
        let matches = matches!(
            (&room.game, room.lifecycle.state()),
            (_, None)
                | (GameId::HighCard, Some(GameState::HighCard(_)))
                | (GameId::Thousand, Some(GameState::Thousand(_)))
        );
        ensure(matches, Error::internal())
    }
    pub fn event(room: &mut Room, actor: &str, kind: &str, data: Value) -> Result<()> {
        let body = json!({"roomId":room.id,"seq":room.events.len()+1,"at":Utc::now().to_rfc3339_opts(SecondsFormat::Millis,true),
            "actor":actor,"type":kind,"data":data,"previousHash":room.events.last().map(|e|e.hash.clone()).unwrap_or_else(||"0".repeat(64))});
        let mut value = body.clone();
        value["hash"] = json!(hash(&body)?);
        room.events.push(serde_json::from_value(value)?);
        Ok(())
    }
    pub fn outcome(room: &mut Room, result: Outcome<HighCardState>, actor: &str) -> Result<()> {
        Self::transition(room, result.map(GameState::HighCard), actor)
    }
    pub fn thousand_outcome(
        room: &mut Room,
        result: Outcome<crate::thousand::State>,
        actor: &str,
    ) -> Result<()> {
        Self::transition(
            room,
            result.map(|s| GameState::Thousand(Box::new(s))),
            actor,
        )
    }
    fn transition(room: &mut Room, result: Outcome<GameState>, actor: &str) -> Result<()> {
        let events = match &result {
            Outcome::Continue { events, .. }
            | Outcome::Shuffle { events, .. }
            | Outcome::Finished { events, .. } => events,
        };
        for e in events {
            Self::event(room, actor, e.kind, e.data.clone())?;
        }
        match result {
            Outcome::Continue { state, .. } => {
                if let GameState::Thousand(s) = &state {
                    room.scores = s.ledger.totals.clone();
                }
                room.lifecycle = Lifecycle::Playing(state);
            }
            Outcome::Finished { state, scores, .. } => {
                ensure(
                    scores
                        .keys()
                        .all(|id| room.players.iter().any(|p| p.id == *id)),
                    Error::internal(),
                )?;
                room.scores = scores;
                Self::event(room, actor, "game.finished", json!({"scores":room.scores}))?;
                room.lifecycle = Lifecycle::Finished(state);
            }
            Outcome::Shuffle {
                state, mut cards, ..
            } => {
                ensure(
                    !cards.is_empty()
                        && cards
                            .iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                            == cards.len(),
                    Error::internal(),
                )?;
                shuffle(&mut cards);
                let proof = DeckProof {
                    protocol: "pastimer-deck-v1".into(),
                    room_id: room.id.clone(),
                    shuffle_id: u32::try_from(room.decks.len() + 1)
                        .map_err(|_| Error::internal())?,
                    cards: cards.clone(),
                    salt: token(),
                };
                let commitment = hash(&proof)?;
                let mut data = json!({"protocol":proof.protocol,"shuffleId":proof.shuffle_id,"commitment":commitment,"count":proof.cards.len()});
                if let GameState::Thousand(s) = &state {
                    data["deal"] = json!(s.deal + 1);
                }
                Self::event(room, actor, "deck.committed", data)?;
                room.decks.push(Deck {
                    proof,
                    commitment,
                    acknowledgements: vec![],
                });
                match state {
                    GameState::HighCard(_) => room.lifecycle = Lifecycle::Committing(state),
                    GameState::Thousand(s) => {
                        let next = s.deal_cards(cards, &room.players)?;
                        Self::thousand_outcome(room, next, actor)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub async fn load(conn: &mut SqliteConnection, code: &str) -> Result<Room> {
        ensure(
            code.len() == 8
                && code
                    .bytes()
                    .all(|b| b.is_ascii_uppercase() || (b'2'..=b'9').contains(&b)),
            Error::bad("Invalid room code"),
        )?;
        Store::get(conn, code)
            .await?
            .ok_or_else(|| Error::new(StatusCode::NOT_FOUND, "Room not found"))
    }
    fn member(room: &Room, actor: &str) -> Result<()> {
        ensure(
            room.players.iter().any(|p| p.id == actor),
            Error::new(StatusCode::FORBIDDEN, "Join this room first"),
        )
    }
    pub async fn create(&self, player: Player, game: GameId) -> Result<Value> {
        self.create_configured(player, game, None).await
    }
    pub async fn create_configured(
        &self,
        player: Player,
        game: GameId,
        options: Option<crate::thousand::Settings>,
    ) -> Result<Value> {
        let options = match game {
            GameId::Thousand => {
                let options = options.unwrap_or_default();
                options.validate()?;
                Some(options)
            }
            GameId::HighCard => {
                ensure(
                    options.is_none(),
                    Error::bad("Settings are for Thousand rooms"),
                )?;
                None
            }
        };
        let mut tx = self.store.pool.begin_with("BEGIN IMMEDIATE").await?;
        let alphabet = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let code = loop {
            let code: String = (0..8)
                .map(|_| char::from(alphabet[OsRng.gen_range(0..alphabet.len())]))
                .collect();
            if Store::get(&mut tx, &code).await?.is_none() {
                break code;
            }
        };
        let mut room = Room {
            options,
            id: Uuid::new_v4().to_string(),
            code,
            host: player.id.clone(),
            game,
            game_version: 1,
            players: vec![player.clone()],
            version: 1,
            lifecycle: Lifecycle::Lobby,
            scores: BTreeMap::new(),
            decks: vec![],
            commands: vec![],
            events: vec![],
        };
        let creation = json!({"game":game,"gameVersion":1,"player":player,"options":room.options});
        Self::event(&mut room, &player.id, "room.created", creation)?;
        Store::save(&mut tx, &room).await?;
        let view = Self::view(&room, &player.id)?;
        tx.commit().await?;
        Ok(view)
    }
    pub async fn read(&self, code: &str, actor: &str) -> Result<Value> {
        self.advance_at(code, actor, Utc::now().timestamp_millis())
            .await?;
        let mut conn = self.store.pool.acquire().await?;
        let room = Self::load(&mut conn, code).await?;
        Self::member(&room, actor)?;
        Self::view(&room, actor)
    }
    pub async fn join(&self, code: &str, player: Player) -> Result<Value> {
        let mut tx = self.store.pool.begin_with("BEGIN IMMEDIATE").await?;
        let mut room = Self::load(&mut tx, code).await?;
        Self::compatible(&room)?;
        if !room.players.iter().any(|p| p.id == player.id) {
            ensure(
                matches!(room.lifecycle, Lifecycle::Lobby),
                Error::conflict("Game already started"),
            )?;
            ensure(
                room.players.len()
                    < room
                        .options
                        .as_ref()
                        .map_or(4, |o| usize::from(o.player_count)),
                Error::conflict("Room is full"),
            )?;
            room.players.push(player.clone());
            room.version += 1;
            Self::event(
                &mut room,
                &player.id,
                "player.joined",
                json!({"player":player}),
            )?;
            Store::save(&mut tx, &room).await?;
        }
        let view = Self::view(&room, &player.id)?;
        tx.commit().await?;
        Ok(view)
    }
    pub async fn command(&self, code: &str, actor: &str, command: Command) -> Result<Value> {
        self.advance_at(code, actor, Utc::now().timestamp_millis())
            .await?;
        ensure(
            (8..=80).contains(&command.request_id.len())
                && command
                    .request_id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
            Error::bad("Version and request ID required"),
        )?;
        let mut tx = self.store.pool.begin_with("BEGIN IMMEDIATE").await?;
        let mut room = Self::load(&mut tx, code).await?;
        Self::member(&room, actor)?;
        Self::compatible(&room)?;
        let fingerprint = hash(&json!({"version":command.version,"action":command.action}))?;
        let old = sqlx::query_scalar::<_, String>(
            "SELECT fingerprint FROM commands WHERE room_id=? AND actor=? AND request_id=?",
        )
        .bind(&room.id)
        .bind(actor)
        .bind(&command.request_id)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(old) = old {
            ensure(
                old == fingerprint,
                Error::conflict("Request ID already used for another command"),
            )?;
            let view = Self::view(&room, actor)?;
            tx.commit().await?;
            return Ok(view);
        }
        ensure(
            room.version == command.version,
            Error::conflict("State changed; refresh and retry"),
        )?;
        ensure(
            !matches!(room.lifecycle, Lifecycle::Finished(_)),
            Error::conflict("Game finished"),
        )?;
        match &command.action {
            Action::Start => {
                ensure(
                    room.host == actor,
                    Error::new(StatusCode::FORBIDDEN, "Only the host can start"),
                )?;
                ensure(
                    matches!(room.lifecycle, Lifecycle::Lobby) && room.players.len() >= 2,
                    Error::conflict("Not ready to start"),
                )?;
                Self::event(&mut room, actor, "game.started", json!({}))?;
                match room.game {
                    GameId::HighCard => {
                        let result = HighCard::initialize(&room.players);
                        Self::outcome(&mut room, result, actor)?;
                    }
                    GameId::Thousand => {
                        let options = room.options.clone().ok_or_else(Error::internal)?;
                        let dealer = OsRng.gen_range(0..room.players.len());
                        let state = crate::thousand::State::new(&room.players, options, dealer)?;
                        let dealer_id = room.players[dealer].id.clone();
                        Self::event(
                            &mut room,
                            actor,
                            "dealer.selected",
                            json!({"player":dealer_id}),
                        )?;
                        Self::thousand_outcome(
                            &mut room,
                            Outcome::Continue {
                                state,
                                events: vec![],
                            },
                            actor,
                        )?;
                    }
                }
            }
            Action::Acknowledge {
                shuffle_id,
                commitment,
            } => {
                let Lifecycle::Committing(GameState::HighCard(state)) = &room.lifecycle else {
                    return Err(Error::conflict("No shuffle to acknowledge"));
                };
                let state = state.clone();
                let deck = room.decks.last_mut().ok_or_else(Error::internal)?;
                ensure(
                    deck.proof.shuffle_id == *shuffle_id && deck.commitment == *commitment,
                    Error::conflict("Wrong deck commitment"),
                )?;
                ensure(
                    !deck.acknowledgements.iter().any(|id| id == actor),
                    Error::conflict("Already acknowledged"),
                )?;
                deck.acknowledgements.push(actor.into());
                let ready = deck.acknowledgements.len() == room.players.len();
                let cards = if ready {
                    Some(deck.proof.cards.clone())
                } else {
                    None
                };
                Self::event(
                    &mut room,
                    actor,
                    "deck.acknowledged",
                    json!({"shuffleId":shuffle_id,"commitment":commitment}),
                )?;
                if let Some(cards) = cards {
                    let result = HighCard::on_deck(state, cards, &room.players)?;
                    Self::event(
                        &mut room,
                        actor,
                        "deck.released-to-game",
                        json!({"shuffleId":shuffle_id}),
                    )?;
                    Self::outcome(&mut room, result, actor)?;
                }
            }
            Action::Thousand { action } => {
                let Lifecycle::Playing(GameState::Thousand(state)) = &room.lifecycle else {
                    return Err(Error::conflict("Thousand is not accepting moves"));
                };
                let deal =
                    state.deal + u32::from(matches!(action, crate::thousand::Action::Shuffle));
                let result = (**state).clone().apply(
                    actor,
                    action.clone(),
                    &room.players,
                    Utc::now().timestamp_millis(),
                )?;
                let proof = MoveProof {
                    deal_id: Some(deal),
                    actor: actor.into(),
                    version: command.version,
                    action: command.action.clone(),
                    salt: token(),
                };
                Self::event(
                    &mut room,
                    actor,
                    "move.committed",
                    json!({"deal":deal,"commitment":hash(&proof)?}),
                )?;
                room.commands.push(proof);
                Self::thousand_outcome(&mut room, result, actor)?;
            }

            Action::Play { card } => {
                let Lifecycle::Playing(GameState::HighCard(state)) = &room.lifecycle else {
                    return Err(Error::conflict("Game is not accepting moves"));
                };
                let result = HighCard::apply(state.clone(), actor, card.clone(), &room.players)?;
                let proof = MoveProof {
                    deal_id: None,
                    actor: actor.into(),
                    version: command.version,
                    action: command.action.clone(),
                    salt: token(),
                };
                Self::event(
                    &mut room,
                    actor,
                    "move.committed",
                    json!({"commitment":hash(&proof)?}),
                )?;
                room.commands.push(proof);
                Self::outcome(&mut room, result, actor)?;
            }
        }
        room.version += 1;
        Store::save(&mut tx, &room).await?;
        sqlx::query("INSERT INTO commands VALUES(?,?,?,?)")
            .bind(&room.id)
            .bind(actor)
            .bind(&command.request_id)
            .bind(fingerprint)
            .execute(&mut *tx)
            .await?;
        let view = Self::view(&room, actor)?;
        tx.commit().await?;
        Ok(view)
    }
    pub async fn audit(&self, code: &str, actor: &str) -> Result<Value> {
        self.advance_at(code, actor, Utc::now().timestamp_millis())
            .await?;
        let mut conn = self.store.pool.acquire().await?;
        let room = Self::load(&mut conn, code).await?;
        Self::member(&room, actor)?;
        if let Some(GameState::Thousand(state)) = room.lifecycle.state() {
            let count = state.archives.len();
            let decks: Vec<_> = room
                .decks
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    let mut value =
                        json!({"commitment":d.commitment,"shuffleId":d.proof.shuffle_id});
                    if i < count {
                        value["proof"] = json!(d.proof);
                    }
                    value
                })
                .collect();
            return Ok(
                json!({"protocol":"pastimer-audit-v2","game":"thousand","roomId":room.id,"phase":room.lifecycle.phase(),"options":state.settings,"players":room.players,"events":room.events,"decks":decks,"deals":state.archives,"commands":room.commands.iter().filter(|p|p.deal_id.is_some_and(|id|id as usize<=count)).collect::<Vec<_>>()}),
            );
        }
        let finished = matches!(room.lifecycle, Lifecycle::Finished(_));
        let decks: Vec<_> = room
            .decks
            .iter()
            .map(|d| {
                let mut value = json!({"commitment":d.commitment,"shuffleId":d.proof.shuffle_id});
                if finished {
                    value["proof"] = json!(d.proof)
                }
                value
            })
            .collect();
        let mut audit = json!({"protocol":"pastimer-audit-v1","roomId":room.id,"phase":room.lifecycle.phase(),"events":room.events,"decks":decks});
        if finished {
            audit["commands"] = json!(room.commands)
        }
        Ok(audit)
    }
    pub fn view(room: &Room, viewer: &str) -> Result<Value> {
        Self::compatible(room)?;
        let table =
            match room.lifecycle.state() {
                None => None,
                Some(GameState::HighCard(state)) => Some(TableView::HighCard(HighCard::view(
                    state,
                    viewer,
                    &room.players,
                ))),
                Some(GameState::Thousand(state)) => Some(TableView::Thousand(Box::new(
                    state.view(viewer, &room.players, Utc::now().timestamp_millis()),
                ))),
            };
        let deck = room.decks.last().map(|d| DeckView {
            shuffle_id: d.proof.shuffle_id,
            commitment: d.commitment.clone(),
            count: d.proof.cards.len(),
            acknowledgements: d.acknowledgements.clone(),
        });
        Ok(serde_json::to_value(RoomView {
            options: room.options.clone(),
            id: room.id.clone(),
            code: room.code.clone(),
            host: room.host.clone(),
            game: room.game,
            game_version: room.game_version,
            version: room.version,
            phase: room.lifecycle.phase(),
            players: room.players.clone(),
            scores: room.scores.clone(),
            latest_event_hash: room.events.last().map(|e| e.hash.clone()),
            deck,
            table,
        })?)
    }
    /// Persistent deadlines make closure recoverable after restart. Reads/commands materialize due transitions.
    pub async fn advance_at(&self, code: &str, actor: &str, now: i64) -> Result<()> {
        let mut conn = self.store.pool.acquire().await?;
        let room = Self::load(&mut conn, code).await?;
        Self::member(&room, actor)?;
        let due = matches!(room.lifecycle.state(),Some(GameState::Thousand(s))if s.due(now));
        drop(conn);
        if !due {
            return Ok(());
        }
        let mut tx = self.store.pool.begin_with("BEGIN IMMEDIATE").await?;
        let mut room = Self::load(&mut tx, code).await?;
        if let Some(GameState::Thousand(state)) = room.lifecycle.state()
            && state.due(now)
        {
            let result = (**state).clone().close(&room.players, now)?;
            Self::thousand_outcome(&mut room, result, "system")?;
            room.version += 1;
            Store::save(&mut tx, &room).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
