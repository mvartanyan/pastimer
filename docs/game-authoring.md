# Adding a game

Use concrete serializable state, action and public-view types, then extend `GameId`, `GameState` and the typed engine dispatch. The high-card demo is in `src/game.rs`; Thousand is in `src/thousand/`, with separate public projection and audit-validation modules. The existing demo's `GameRules` trait is one simple adapter; a multi-phase game may use its own constructor and transition methods as Thousand does.

All games return shared `Outcome<S>` transitions. `Outcome::map` adapts a concrete game state to the engine's tagged state enum. The engine has one shared transition path for events, score projection, commitment generation and snapshot lifecycle. New state variants require exhaustive dispatch updates; keep backward-compatible decoding for existing snapshots.

- Constructors validate their typed settings. Random first-player/dealer selection belongs to the engine and is logged as public context; rule functions remain deterministic.
- `Outcome::Shuffle` supplies unique cards and waiting state. The engine securely shuffles, publishes the commitment, and passes an owned deck copy to the game. Thousand deals immediately without acknowledgements; the demo retains its original confirmation gate for compatibility.
- `apply` consumes state, authorizes the actor, validates phase and move, and returns an outcome or error. Required clock values are explicit inputs. Invalid actions roll back the complete state/event/command-receipt transaction.
- `Outcome::Continue` keeps the match active. `Outcome::Finished` supplies final scores. Use `scoring::Ledger` for cumulative awards and deal checkpoints; the engine projects the authoritative totals into the common room score field.
- `PublicEvent` data is visible immediately to every room member. Do not place hidden hands, bids, salts or private transfer cards in it. Sealed action commitments are logged by the engine; reveal them only after their deal is safe to disclose.
- Define a public view type and add it to `TableView`. Keep private state out of it. Common fields can expose own hand, seats, scores and permitted controls; game-specific fields support bids, transfers, stock and trick piles. New games need explicit frontend rendering alongside the existing shared lobby, session, command and receipt logic.
- Timed transitions need persisted deadlines, idempotent resolution and restart recovery. They must not depend only on a live timer. Thousand's three-second trick closure demonstrates this pattern.

Thousand actions use the platform envelope `{"type":"thousand","action":{...}}`, preserving existing demo commands. Room creation accepts typed game options. Settings are fixed once a match starts, and versioned snapshots must not be silently interpreted under incompatible rules.

The core `Card` represents a validated standard rank/suit card. Thousand defines its own ranking and points rather than changing the demo's generic rank. Multi-deck or nonstandard games need explicit card-instance identities.

Thousand v2 audits reveal one settled deal at a time. Records bind the original deck, every movement, tricks, scores and cancellations to the event chain. Independent card-location validation complements full action replay. Keep the current deal private and preserve legacy v1 verifier behavior.

Tests should cover complete matches, illegal/out-of-turn actions, score boundaries, hidden-information projections, concurrent retries, reconnection, cancellation and every shuffle/deadline boundary. Keep pure rules tests separate from persistence/API and browser tests.
