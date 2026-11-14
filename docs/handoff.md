# Local-session handoff

Updated 2026-11-14. Repository: `mvartanyan/pastimer`, branch `main`.

## Where the work stands

The Rust multiplayer platform, the owner's complete Thousand rules, audit verifier and redesigned card-table UI are implemented and committed. The next intended operational step is deploying to **Horse**, using the owner's existing local **k3s horse** project, and connecting a Telegram bot. Horse deployment has **not** been performed or verified from this cloud session. There is no chosen public domain or confirmed bot configuration in this repository.

Relevant implementation commits:

- `e6c4e87` — Rust platform, Thousand rules, tests and audit verification.
- `5dbc456` — container/k3s preparation, deployment instructions and screenshot capture helper.
- `ba4ac97` — illustrated playing cards and redesigned mobile card-table UI, with browser checks for sorting, clipboard invitation codes and transfer previews/duplicate prevention.
- `0516a91` — keep local card selection responsive during background refresh, with a regression check that holds the refresh response open.

This handoff, root `AGENTS.md` and the checked-in screenshot gallery provide context without needing the cloud conversation. The sidebar placement of a chat has no bearing on the Git commits available to a local checkout.

During handoff validation, a browser rerun exposed a shared UI timing issue: card selection used the network-operation busy guard, so a tap during a background refresh could be ignored. Selection now updates locally while the refresh is in flight. A regression check deliberately holds the refresh response open and verifies that the card becomes selected and its play action enabled. The fix is committed in `0516a91`; it does not change card artwork or game rules.

## Owner's direction and agreed behavior

Pastimer is a Telegram Mini App server for multiple minigames, primarily card games. Reusable facilities should cover rooms joined by code, storage, decks, shuffling, event logs/audits, scores, hands and table UX. The owner explicitly chose Rust for concurrency, performance and compile-time checks, and SQLite for this stage. The frontend is browser JavaScript/CSS with locally bundled SVG card artwork.

The full specification is [English](thousand-rules.md) and [Russian](thousand-rules.ru.md); neither should contain unresolved rule choices. Important variant details that differ from other versions of Thousand:

- Card points are **10, 10, 5, 5, 5, 0** for A, 10, K, Q, J, 9. Three or four seats; only three play a deal. Four-player dealer bonuses, barrel restrictions and four-nines rollback are specified in the rules.
- An auction raise is at least 10, with the resulting bid divisible by 5. Saying “here” immediately after a pass takes the current bid. No marriage is required to bid. Final order increases after distribution may be in steps of 5.
- Stock visibility depends on **active** barrel players. A sitting-out dealer on the barrel does not force barrel play, receive a stock bonus or open the stock on their own. For the confirmed A/B-at-900 example where A sits out, B plays compulsory 100 without an auction.
- An untouched compulsory contract remains compulsory after a final-order increase. Barrel contracts are compulsory. Noncompulsory contracts can be dismissed before the opening lead; each third stick costs 150. An eligible dealer keeps the bonus on dismissal.
- Marriages are voluntary and require a prior won trick. Follow suit, then trump, with no obligation to overtake.
- Four nines can cancel the deal at any moment while all four remain held, undoing score effects including dealer bonuses; the same dealer redeals. Cancelled history remains auditable.
- Publish each shuffled deck hash and immediately deal; **no player confirmation step** for Thousand. The dealer's click initiates secure OS-backed randomization; its timestamp is not the random seed.
- A completed trick closes after three seconds. Anyone may inspect **each player's own last captured trick**, not just the last trick globally. New deals clear those piles.
- Each settled/cancelled deal's audit reveals the initial shuffled deck, proof salt, every card movement and all tricks. An ongoing deal remains private. Disconnection preserves seats/hands and waits for the player's action.

## Source map

| Area | Entry points |
| --- | --- |
| Startup/configuration and embedded HTTP assets | `src/main.rs`, `src/http.rs`, `.env.example` |
| Room lifecycle, typed dispatch and command processing | `src/engine.rs`, `src/model.rs`, `src/game.rs` |
| SQLite snapshots, events, sessions and durable retry receipts | `src/store.rs` |
| Thousand transitions, settings and scoring | `src/thousand/mod.rs`, `src/scoring.rs` |
| Explicit player-visible Thousand projection | `src/thousand/view.rs` |
| Deck commitments, audit assembly and replay | `src/crypto.rs`, `src/audit.rs`, `src/thousand/verify.rs` |
| Offline verification CLI | `src/bin/verify-audit.rs` |
| Telegram signed identity validation | `src/telegram.rs` |
| Shared lobby/session/room/hand UI | `public/app.js`, `public/index.html`, `public/style.css` |
| Card SVG artwork, sorting and hand layout | `public/cards.js` |
| Auction, transfers, play, deadlines and trick-pile UI | `public/thousand.js` |
| Rules tests | `src/thousand/tests.rs` |
| Persistence/API/concurrency/compatibility tests | `tests/platform.rs`, `tests/thousand.rs`, `tests/fixtures/node-v1.json` |
| Browser flows and visual capture | `tests/browser-smoke.mjs`, `tests/thousand-browser.mjs`, `scripts/capture-screenshots.mjs` |
| Deployment preparation | `deploy/Dockerfile`, `deploy/k3s/`, [deployment guide](deployment.md) |

Read [architecture.md](architecture.md) for transaction, privacy and cryptographic boundaries; [game-authoring.md](game-authoring.md) explains adding games. Games use concrete serializable state/action/view types and exhaustive typed dispatch. The older high-card demo and Node-era snapshot/audit fixtures are retained for compatibility.

## Start locally

First inspect local Git status and preserve any existing changes. Fetch and fast-forward `main` from GitHub when appropriate; do not reset a local checkout to discard work. All needed application source and lockfiles are committed.

Install Rust through rustup; the repository pins **1.93.1** with rustfmt and Clippy. Install a platform C toolchain and pkg-config; SQLite is bundled by the Rust dependency. From the repository root, in a POSIX shell:

```sh
cargo build --locked
DEV_AUTH=1 cargo run --locked
```

Open `http://127.0.0.1:3000`. Use separate browser profiles/contexts to represent multiple players. No Telegram bot is required for this loopback-only development login. `.env.example` documents settings but `.env` files are **not loaded automatically**. Node is unnecessary to run the backend.

Every frontend change requires recompiling and restarting the server because assets are embedded. There is no separate frontend build/dev server. A server already running in the cloud may be using an older binary; its process is not an application deployment or part of the local handoff.

SQLite defaults to `.data/pastimer.sqlite`. The cloud development database, login sessions and browser-local receipts are not transferred through Git. A fresh local run starts with its own database; retain an existing local database if present. `scripts/install-cloud.sh` and `scripts/rust-env.sh` are cloud conveniences, not required local setup.

For browser checks and screenshots, install Node.js **22+** and the locked development dependencies:

```sh
npm ci
# If no supported system Chromium is available:
npx playwright install chromium
npm run test:browser
node scripts/capture-screenshots.mjs
```

The scripts use system `/usr/bin/chromium` when available, otherwise Playwright's installed Chromium. `CHROMIUM_PATH` selects another executable. Install the browser's OS dependencies if needed on a fresh Linux system. Both browser suites and the capture helper launch isolated servers and temporary databases, so they do not need a manually running server. The capture helper expects binaries from `cargo build --locked`; `PASTIMER_BIN` can select another server binary with `verify-audit` alongside it.

The [gallery](screenshots/README.md) contains committed previews of the redesigned UI. Fresh captures go to ignored `artifacts/screenshots/`; copy reviewed replacements to `docs/screenshots/` when updating the gallery. Previews use signed synthetic Telegram identities against the real server, not a live Telegram client.

## Verification

The following checks passed in the managed Linux cloud on 2026-11-14 (Rust checks on `ba4ac97`, then the complete browser suite with the card-selection fix in `0516a91`; Rust logic was unchanged):

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
npm run test:browser
```

The Rust suite has 30 tests covering complete deals/matches, the attainable 380-point maximum, barrel behavior, sticks, privacy, rollback, four nines, per-player trick piles, restart recovery, rule replay, proof tampering, command races, retry idempotency, Telegram signatures and legacy compatibility. Browser checks cover the demo, full three-player Thousand play with real three-second closure, four-player here bidding/mixed-visibility distribution/dismissal, audit verification, reload/retry, card rendering, sorting, clipboard copying, mobile overflow and card selection during an in-flight refresh.

Release builds and the four signed-identity screenshot flows also passed during the UI work. A container build/runtime check was completed before `ba4ac97`; **rebuild the image from the latest checkout** for deployment so it contains the redesigned assets. Native local-platform and real Telegram/HTTPS behavior still need verification in the target environment.

To verify a downloaded audit independently:

```sh
cargo run --locked --bin verify-audit -- path/to/pastimer-XXXXXXXX.json
```

## Remaining work: Horse and Telegram

The owner requested commit/push, Horse deployment, bot connection instructions and game screenshots. Source, instructions and browser screenshots are delivered. The operational deployment and real Telegram validation remain pending.

**Horse is an existing configured SSH server.** The owner specifically directed using the separate **k3s horse** project for its established setup. That local project, SSH credentials and kubeconfig were unavailable to the cloud task. A local session should inspect them before asking the owner to supply information already present there. Do not infer Horse's domain, ingress class, certificate issuer, storage class, CPU architecture or Kubernetes context from placeholder manifests.

Suggested sequence, using [deployment.md](deployment.md) for commands:

1. Locate the local `k3s horse` project and read its own instructions. Inspect the existing `horse` SSH alias and intended Kubernetes context; establish what is already deployed and whether Pastimer has existing data/resources.
2. Confirm the target architecture, public HTTPS hostname, ingress/certificate arrangement and persistent storage. Use the existing cluster conventions rather than installing replacement infrastructure.
3. Build the latest immutable image/tag for Horse's architecture, transfer/import it or use the established registry, and set the deployment to that exact image. The checked-in `pastimer:horse` tag is a starting value.
4. Supply the real bot token securely as Secret `pastimer-telegram`, key `bot-token`. Production needs `PASTIMER_ENV=production`, with `DEV_AUTH` unset. There is no bot webhook or update-polling worker; all interaction is in the Mini App.
5. Deploy one SQLite writer with the existing Recreate strategy and persistent PVC. Preserve any existing database with a consistent backup. The ingress example is deliberately excluded from the base kustomization until its real values are supplied.
6. Configure BotFather's menu button with the same-origin HTTPS root URL. Validate actual Telegram identities, multiplayer joining/play, persistence across restart, TLS and completed-deal audit export. A passing health probe alone does not establish those flows.
7. Record the resulting URL, image reference, deployment commands and checks in this repository without committing secrets or local credentials.

The cloud workspace is development infrastructure. No local SSH keys or production secrets have been committed. Kubernetes token/identity setup is documented in [deployment.md](deployment.md); the local session should use the owner's available authorized configuration.
