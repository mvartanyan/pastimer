# Local-session handoff

Updated 2026-11-14. Repository: `mvartanyan/pastimer`. Local Horse continuation: branch `codex/horse-deployment`; pulled `main` at `6376528`.

## Where the work stands

The Rust multiplayer platform, the owner's complete Thousand rules, audit verifier and redesigned card-table UI are implemented. **Pastimer is deployed and verified on Horse at https://1000.vartanyan.com**, using the established local `/Users/michael/dev/k3s-horse` setup. [@pastimerbot](https://t.me/pastimerbot) has a verified default `Play / Играть` menu button pointing to that origin.

The running application source is `f3fe9a915567163acf0c7ec71047002c51377a91`. The live three-player browser deal, deployed audit verifier, production authentication restrictions, HTTPS, asset hashes and persistence across pod replacement pass. The browser uses signed synthetic Telegram identities; actual Telegram-client identity and multiplayer acceptance remain a separate check. See [the Horse deployment record](horse-2026-11-14.md) for image digests, backups and operating details.

Relevant implementation commits:

- `e6c4e87` — Rust platform, Thousand rules, tests and audit verification.
- `5dbc456` — container/k3s preparation, deployment instructions and screenshot capture helper.
- `ba4ac97` — illustrated playing cards and redesigned mobile card-table UI, with browser checks for sorting, clipboard invitation codes and transfer previews/duplicate prevention.
- `0516a91` — keep local card selection responsive during background refresh, with a regression check that holds the refresh response open.
- `f3fe9a9` - keep submitted actions responsive during background refresh and reject stale room snapshots; local regression and live Horse validation passed.

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

Release builds and the four signed-identity screenshot flows also passed during the UI work. A container build/runtime check was completed before `ba4ac97`; **rebuild the image from the latest checkout** for deployment so it contains the redesigned assets. The subsequent local continuation verified native local-platform checks, public HTTPS and Horse runtime behavior; actual Telegram-client acceptance remains pending. See [the deployment record](horse-2026-11-14.md).

To verify a downloaded audit independently:

```sh
cargo run --locked --bin verify-audit -- path/to/pastimer-XXXXXXXX.json
```

## Horse continuation completed locally

The local continuation pulled `main`, preserved the pre-existing untracked root `thousand-rules.ru.md`, and read the local k3s project, SSH configuration and live cluster. Horse is a single amd64 node using its existing Traefik ingress, `letsencrypt-prod` issuer and `local-path` storage. The owner chose `1000.vartanyan.com` and supplied the bot token in an ignored local `.env`, now mode 0600. Credentials were installed as Secret `pastimer-telegram`, key `bot-token`, without being printed or committed.

Production has one replica, Recreate updates and the bound 5Gi `pastimer-data` PVC. The canonical [Horse overlay](../deploy/horse/) and the rendered local `/Users/michael/dev/k3s-horse/pastimer/pastimer.yaml` describe the actual deployment. Do not delete or replace the PVC for updates.

The original Colima/QEMU image build failed in the linker. The image was then built natively on Horse from a Git source archive with a temporary, separate `pastimer-build` containerd namespace. The initial live browser game exposed Play clicks being swallowed by the shared background-refresh guard. Commit `f3fe9a9` fixes that race and stale-response rollback, with a failing-before/passing-after regression. The full browser suites passed against the final commit. The corrected image was rebuilt, imported into the existing k3s image store and rolled out with the same database.

Live checks passed:

- Public health and game catalogue, trusted HTTPS and HTTP redirect.
- All five embedded frontend assets match the deployed source byte-for-byte.
- Local login, invalid Telegram signatures and cross-origin writes are rejected in production. Session cookies are Secure, HttpOnly and SameSite=Strict.
- Three signed synthetic identities joined and completed a full Thousand deal through independent browsers. The deployed verifier accepted its audit: one deal, one deck, 87 events and 30 moves.
- After a consistent SQLite backup and actual pod replacement, all three sessions, the room, completed deal and exact exported audit survived unchanged.
- The replacement pod is Ready, running the expected image/config ID with zero restarts. The TLS certificate is Ready.
- Telegram's default menu button was set and verified directly from Horse. No webhook or polling worker was added.

Consistent backups with successful SQLite integrity checks are on Horse:

- `/var/backups/pastimer/pastimer-20261114-pre-refresh-fix.sqlite`
- `/var/backups/pastimer/pastimer-20261114-pre-restart.sqlite`

The prior image and build archives are retained for rollback. The temporary native builder is stopped, with no build tasks left running. Colima's unused failed-build cache was pruned (2.335GB); containers and volumes remained at zero, guest fstrim ran, temporary VM DNS was restored and `dotint-build` returned to Stopped. The original Docker context remains `colima`.

Remaining acceptance: open [@pastimerbot](https://t.me/pastimerbot) in actual Telegram, press Start and `Play / Играть`, confirm the signed-in identity, then join/play from other Telegram accounts. Synthetic identities do not establish that real-client check. A Main Mini App/profile launch button is optional and separate from the configured menu button. Scheduled/off-host database backups are not established by this initial deployment.
