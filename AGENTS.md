# Working on Pastimer

Read [README.md](README.md) and [docs/handoff.md](docs/handoff.md) when starting a new session. The handoff records implemented work, validation, remaining deployment work and the owner's decisions. Update it when those facts change.

## Project decisions

- Use Rust for the backend (Axum, Tokio, SQLx) and SQLite for persistence. Node.js is development tooling for browser tests, not a backend runtime.
- Build common multiplayer, persistence, deck, audit, scoring and UI facilities for Telegram card games. The first game is the owner's variant of Thousand / тысяча.
- Treat [English rules](docs/thousand-rules.md) and [Russian rules](docs/thousand-rules.ru.md) as the agreed specification. Keep both synchronized if rules change; ask about genuine new ambiguities rather than silently adopting another Thousand variant. Keep the rules files about the rules.
- Thousand publishes the shuffled deck hash and deals immediately; players do not acknowledge it. The legacy high-card demo still uses acknowledgements.
- After three seconds a completed trick joins its winner's pile. Players can inspect the most recent trick won by **each player**, including themselves, throughout the current deal.
- Normal play fits the Mini App viewport. Cards play on one tap; eligible marriages are selected by default when leading a legal king/queen. Keep the voluntary opt-out before playing. Show the current trump prominently with crossed gold rings and the suit. Inspect last won tricks from the seat/hand display, without a separate pile row.
- Auction calls are paced three seconds apart and shown in seat speech bubbles. Keep each player's latest call until the auction ends, then show the contractor and contract. The log retains every call.
- Drag stock transfers from the hand onto the recipient seat/hand, with touch and mouse support. Tap a staged card to turn it face up (open) or face down (secret); drag it back to cancel. Keep one atomic Give action on the table and no separate lower transfer panel. Staged cards leave the visible hand/count until returned or committed. Recipients see open cards face up and secret cards face down; reveal a secret card, then tap again to sort it into the hand. Use the incoming card highlight without visible sender/tap instructions, and suppress old receipts after play begins. Hand counts come from the actual server hand. Taken stock disappears from the table.
- Computer bids use conservative guaranteed tricks/points and an attainable marriage; the policy sees only its own view and public played cards. Preserve the winning-card/announcement plan when distributing and playing.
- Table recovery must work with fresh browser storage after Telegram re-authentication. Preserve server-side membership lookup and actor-scoped durable command IDs.
- Preserve hidden information in public views and events. Reveal a settled or cancelled deal's full deck/movements/tricks in its audit, while keeping an unfinished deal private. See [the trust model](docs/architecture.md) for what the commitment actually establishes.

## Development

- Assets in `public/` are embedded in the Rust binary via `include_str!`; rebuild and restart after frontend edits. There is no frontend bundler.
- Local development: `DEV_AUTH=1 cargo run --locked`, then open `http://127.0.0.1:3000`. Use separate browser profiles for players. `.env.example` is a reference; the app does not automatically load `.env`.
- In the managed cloud only, `. scripts/rust-env.sh` activates its workspace-local Rust toolchain. On a local computer use its normal Rust installation and the pinned `rust-toolchain.toml`.
- Check Rust changes with `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked`. For UI/protocol changes also run `npm run test:browser` after installing the optional Node/Playwright dependencies.
- Browser suites launch isolated servers and temporary databases. `node scripts/capture-screenshots.mjs` captures real game flows using signed synthetic Telegram identities; it does not validate a live Telegram client. See [the screenshot gallery](docs/screenshots/README.md).

## Persistence and deployment

- Preserve existing local work and databases. `.data/`, SQLite/WAL files, `.env`, build output and credentials do not belong in Git. Take consistent backups before database/deployment changes.
- Keep command version checks, durable retry IDs, atomic snapshot/event/receipt transactions, explicit public DTOs and persisted trick deadlines intact. Preserve supported snapshot/audit compatibility.
- Horse is the owner's configured SSH server. Its established configuration is in the separate local **k3s horse** project; inspect that project and local SSH/kube configuration before choosing deployment settings. Its files and credentials were unavailable in the cloud session.
- The checked-in k3s manifests and BotFather guide are preparation, not evidence of deployment. Follow [docs/deployment.md](docs/deployment.md), confirm the target cluster, use one SQLite writer, and preserve its PVC. Ingress values are examples until resolved against Horse's setup.
