# Architecture decisions and trust boundaries

## Initial target

A modular Rust application using Axum, a multithreaded Tokio runtime, SQLx with an eight-connection SQLite WAL pool, and a same-origin Mini App. The production server is a compiled binary with embedded frontend assets and has no Node.js dependency. No external database, Redis or bot webhook is needed to develop the platform. This minimizes deployment moving parts while keeping Telegram outside the rule engine. Postgres, a transactionally written event/outbox table and WebSocket fan-out are the likely next step for multiple workers; avoid a broker before a workload needs it.

The existing tables contain room snapshots (including private state, complete event history, shuffle secrets and hidden action proofs), deduplication receipts and hashed session tokens. Schema creation is idempotent for the initial schema; production schema changes will need versioned migrations. This is a working first schema, not a complete long-term data model. Events are logically append-only through the engine, embedded in snapshots; they are not a database-enforced immutable ledger.

## Types, concurrency and limits

Serde decodes commands into tagged action enums and rejects malformed card IDs, negative versions, missing fields and unexpected action fields. Game modules use concrete state, action and view types with typed `GameId`/`GameState` dispatch; the legacy demo also exposes its original `GameRules` trait. `Outcome` makes continue, shuffle and finish mutually exclusive; finishing requires an explicit score map. `Lifecycle` distinguishes lobby, committing, playing and finished states. Public response DTOs cannot directly substitute private state for a table view. Unsafe code is forbidden in this crate; Clippy denies unwrap/expect. These protections do not statically prove game legality, prevent all logical races, or remove the need for runtime validation.

Tokio tasks handle requests concurrently. SQLx uses SQLite worker threads behind async calls and a bounded connection pool (eight connections for files, one for isolated in-memory tests). No blocking SQLite call is made directly on a Tokio request worker. WAL allows readers alongside the writer; `BEGIN IMMEDIATE` serializes write transactions before loading their snapshot, avoiding stale-read upgrades and lost updates. Concurrent duplicate and conflicting-command tests use file-backed pooled connections on a multithreaded runtime. There is no process-wide game lock. The rate-limit map uses a short synchronous mutex without awaiting under it.

This is a correctness-tested concurrency design, not a throughput benchmark. SQLite still has one writer per file. Every room snapshot currently includes its event history; reading and rewriting long histories costs more as games grow. Normalized append-only event storage, retention and workload measurements should precede high-volume deployment. SQL statement shapes are exercised by integration tests; SQLx runtime queries are not compile-time SQL validation. Heavy future simulations belong on bounded worker tasks, outside transactions and request workers.

Original Node schema/table names and canonical hash encoding are retained. A Node-generated fixture verifies lobby/committing/finished state reads, continued play, completed audits and command deduplication. The new release does not reset or export the live database. Back up before deploying and avoid concurrent old/new writers. Current protocol payloads use integer numbers and ASCII field names; a future cross-language protocol with floats or arbitrary keys needs a formal canonicalization version.

## Commands and persistence

A mutation runs within `BEGIN IMMEDIATE`: load room, authorize, check `(room, actor, requestId)`, check expected version, validate rules, append events, save snapshot and command receipt, commit. Identical retries return the latest authorized projection; reuse of the ID with a different payload is rejected. A version conflict requires refreshing and issuing a new command ID. The browser persists unconfirmed commands in session storage and retries them with the exact same ID and payload, including across a reload. Only a definite version rejection permits one refreshed retry with a new ID; network/server failures preserve the pending command. Clients must not automatically repeat an unconfirmed move under a new ID.

Joining is idempotent by authenticated player identity. The initial implementation supports lobby joins, host-only start and no seat changes after start. It has no host transfer, timeout/forfeit, spectators, user deletion, automatic room expiry or match restart. Thousand waits for the required player to reconnect without timeouts or automatic moves. Its shuffle requires the dealer’s action, not acknowledgements from all players.

Plugins are trusted server code. They must be deterministic, synchronous, validate every action and avoid side effects. Snapshots retain plugin ID and version; an incompatible deployment must retain the prior version or migrate explicitly. The typed game dispatch supports a single version per game and rejects mismatches rather than interpreting old state with new rules.

Private state is never serialized directly to clients: the explicit per-game public `view` is the boundary. Tests for each real game must verify that its view and public events omit every other hand, unrevealed bid, future deck and secret salt. Public event payloads are deliberately plugin-defined; the engine cannot infer whether a plugin accidentally leaks secrets.

## Deck commitments and per-deal audits

1. Use `rand` Fisher–Yates shuffling with `OsRng` and unbiased bounded draws.
2. Generate a fresh 256-bit salt. Construct `{protocol:"pastimer-deck-v1", roomId, shuffleId, cards, salt}`. Index zero is the first card drawn.
3. Canonicalize JSON with sorted ASCII field names and preserved array order; SHA-256 the UTF-8 bytes. The room and shuffle ID bind the proof to one epoch; the salt prevents guessing small arrangements.
4. For Thousand, the dealer triggers the shuffle. Persist `deck.committed` before the deal events, then deal immediately in the same atomic transition. No player confirmation is required. Clients retain observed hashes and event receipts independently of the server.
5. After each completed or cancelled deal, release its initial deck, salt, sealed command proofs and complete `DealRecord`. A new shuffle uses a fresh independent permutation and salt. The ongoing deal remains sealed.

The v2 audit contains the public chain, settled-deal records and revealed commands. A record includes every card movement between deck, stock, hand, table and named trick pile, all closed tricks and a pending completed trick if cancellation interrupted closure, marriages, contract, dealer bonus, scores and sticks before/after settlement. A `recordHash` in the event chain commits the whole record. Four-nines cancellation restores the checkpoint without erasing the old transcript.

The Rust verifier checks receipts, commitments, card conservation, required suit/trump, winners, score arithmetic and cross-deal checkpoints. It then replays every revealed player command and compares the emitted public events and final record, including auctions and cancellations. Replay uses a logical clock; live three-second behavior is checked separately by runtime and browser tests.

The legacy high-card demo retains its v1 acknowledgement gate and match-end proof disclosure for existing room/proof compatibility. New snapshots explicitly tag game state; the decoder still accepts the original untagged demo state. Existing audit and move-proof hashes remain unchanged.

## Scores and trick deadlines

`scoring::Ledger` provides cumulative scores and bounded awards. Thousand saves a ledger/stick checkpoint at each deal and restores it on four-nines cancellation. `Room.scores` is updated from the authoritative game ledger in the same transaction, giving the shared client a score projection after every accepted action. Contracts, barrel eligibility and stick rules stay in the game module.

A third card creates a typed closing phase with a persisted deadline of server time plus 3000 ms. Further card plays are rejected until closure. Reads and commands materialize any due closure transactionally and exactly once; no in-memory timer is required to survive a restart. The browser schedules a deadline refresh in addition to its normal polling. Offline rooms are materialized on their next access. Each player has a separate collected-trick pile; public views include only the top trick of each pile and its count. New deals start empty piles.

## What this establishes

A player who retains an observed hash/event receipt can detect a revealed deck inconsistent with that commitment, altered logged moves, or a rewritten event prefix. The offline verifier checks all recorded deck/move commitments and receipt hash anchors. Completed logs expose discarded/undealt cards too; users must understand that all secret play becomes auditable after the deal.

A chain kept only by its operator is rewritable. Browser receipts provide local external anchors, not a public transparency service. There is no signed server identity, cross-player receipt gossip or external witness yet. A malicious server can equivocate between players, suppress an ending, omit logging an action, or selectively abort unfavorable games. A hash commitment **does not prove unbiased randomness**, protect against an operator who knows every hand, or by itself establish correct rule execution/dealing. The Thousand verifier reconstructs dealing and replays the published transcript. This establishes consistency of the revealed record, not a guarantee that every client actually received the same messages during live play.

For stronger fairness, add a published multi-party commit/reveal seed protocol: commit server seed before receiving player seed commitments, reveal player contributions, derive a reproducible unbiased shuffle, and reveal server material after hidden information is safe. Specify contribution deadlines, dropped-player handling and abort penalties; otherwise selective abort remains possible. Merkle commitments/per-card proofs could verify dealt positions during play. Signed/witnessed receipts and multi-party shuffle contributions remain additional work.

## Operational boundaries

- Telegram initData is accepted only with a valid bot HMAC and a timestamp within five minutes (30 seconds future clock tolerance). Authentication issues a one-day opaque session; reopening inside Telegram can renew it. Bot tokens never enter the browser or audit record.
- Cookie sessions are HTTP-only, SameSite Strict and Secure in production. Mutations require JSON; foreign Origin and cross-site fetch requests are rejected. API responses are not publicly cached. Use trusted TLS termination and correct Host routing.
- Basic per-IP request throttling exists in process. Room creation, database growth and anonymous authentication still need stronger quotas/retention before public launch. Proxy-aware IP handling must be designed for the actual deployment; arbitrary forwarded headers are not trusted here.
- SQLite includes the deck and hand secrets in plaintext at rest. Apply filesystem permissions, encrypted storage and restricted backups at deployment. Anyone with database access can see them.
- Polling is adequate for a first card game. Transport failures never authorize speculative client moves. No money handling or wagering is implemented.

## Thousand rules

The implemented house variant is documented in [English](thousand-rules.md) and [Russian](thousand-rules.ru.md). Rule tests include a legal 380-point construction, a complete match through the barrel, per-player top tricks, voluntary marriage announcements, stock visibility and bonus rollback. Other Thousand conventions are not implicitly enabled.
