# Local-session handoff

Updated 2026-11-14. Repository: `mvartanyan/pastimer`. Local Horse continuation: branch `codex/horse-deployment`; pulled `main` at `6376528`.

## Where the work stands

The Rust multiplayer platform, the owner's complete Thousand rules, audit verifier and redesigned card-table UI are implemented. **Pastimer is deployed and verified on Horse at https://1000.vartanyan.com**, using the established local `/Users/michael/dev/k3s-horse` setup. [@pastimerbot](https://t.me/pastimerbot) has a verified default `Play / Играть` menu button pointing to that origin.

The running application source is `f54714d388be37749979e8fe206a323c73d7de9e`. The table now has one centred top-view trump symbol, large rings on announcing cards, owner-anchored played cards, a persistent leader cue and animated collection to the winning hand. Contract information appears only at its owner's seat. Transfers now drag to player hands and turn on tap; eligible marriages default on, and the current trump uses crossed gold rings plus its suit. Incoming cards now use a highlight without sender/tap labels, and reopening a deal no longer replays stale receipts or miscounts the hand. The current gameplay update includes paced auction calls, stronger computer seats, one-screen play, readable cards, hand-selected transfers and table recovery after reopening. The owner confirmed actual Telegram launch, identity and table creation. Live browser checks also completed solo/two-human computer games and verified their proofs with the deployed audit binary. Original sessions, completed room and exact audit survived both pod replacement and this upgrade. See [the current Horse release record](horse-table-cues-2026-11-14.md) for image digests, backups and checks; [the initial record](horse-2026-11-14.md) retains the first deployment history.

Relevant implementation commits:

- `e6c4e87` — Rust platform, Thousand rules, tests and audit verification.
- `5dbc456` — container/k3s preparation, deployment instructions and screenshot capture helper.
- `ba4ac97` — illustrated playing cards and redesigned mobile card-table UI, with browser checks for sorting, clipboard invitation codes and transfer previews/duplicate prevention.
- `0516a91` — keep local card selection responsive during background refresh, with a regression check that holds the refresh response open.
- `f3fe9a9` - keep submitted actions responsive during background refresh and reject stale room snapshots; local regression and live Horse validation passed.
- `4a9119c` - stack mobile lobby cards and normalize native form control heights and spacing.
- `f54714d` - equal circular rings, declaring-card overlay, owner positions, leader/winner cues and trick collection.
- `76064a6` - mouse/touch distribution onto recipient hands, tap to turn, drag back to cancel and no lower panel.
- `60c3808` - default eligible marriages on, optional opt-out and prominent rings/suit trump marker.
- `8458703` - constrain long recipient names in landscape; wait for actionable controls in browser checks.
- `92d75bd` - highlight-only incoming cards, authoritative hand counts and no stale receipts after reopening.
- `5517348` - paced auction bubbles, viewport gameplay, hand-selected transfers/receipts, readable suits, direct play, fresh-storage recovery and stronger computers.
- `fe12ef5` - legal computer opponents for one/two humans, computer-dealer requests, audited automatic turns, persistence and browser acceptance.

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
| Computer policy, transactional automatic turns and roster | `src/computer.rs`, `src/engine.rs`, `src/model.rs` |
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
| Browser flows and visual capture | `tests/browser-smoke.mjs`, `tests/thousand-browser.mjs`, `tests/computer-browser.mjs`, `scripts/capture-screenshots.mjs` |
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

Release builds and the four signed-identity screenshot flows also passed during the UI work. A container build/runtime check was completed before `ba4ac97`; **rebuild the image from the latest checkout** for deployment so it contains the redesigned assets. The subsequent local continuation verified native local-platform checks, public HTTPS and Horse runtime behavior; the owner has since confirmed actual Telegram launch, identity and table creation. See [the current release record](horse-table-cues-2026-11-14.md) for the computer follow-up and live browser checks.

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
- `/var/backups/pastimer/pastimer-20261114-pre-computers.sqlite`
- `/var/backups/pastimer/pastimer-20261114-pre-gameplay.sqlite`
- `/var/backups/pastimer/pastimer-20261114-pre-receipts.sqlite`
- `/var/backups/pastimer/pastimer-20261114-pre-receipts-final.sqlite`
- `/var/backups/pastimer/pastimer-20261114-pre-drag-marriage.sqlite`
- `/var/backups/pastimer/pastimer-20261114-pre-table-cues.sqlite`

The prior image and build archives are retained for rollback. The temporary native builder is stopped, with no build tasks left running. Colima's unused failed-build cache was pruned (2.335GB); containers and volumes remained at zero, guest fstrim ran, temporary VM DNS was restored and `dotint-build` returned to Stopped. The original Docker context remains `colima`.

## Earlier layout and computer-opponent follow-up

The owner confirmed the real Telegram launch/identity/create-table check, then requested a mobile layout correction and computer opponents so one or two humans can test without recruiting a full table. Both changes were deployed in `fe12ef5`, before the gameplay update below.

The lobby stacks below 650px and uses aligned 46px form controls, including normalized native selects. New `Computer seats` options include `2 computers - solo` and `1 computer - 2 people` for a three-seat table. Four-seat tables support up to three computers. Computer settings are platform metadata, separate from the unchanged Thousand rules. Existing rooms decode with an empty computer roster.

Computers pass in auctions, choose private transfers, retain the current order and play legal cards, preferring an available marriage announcement. The policy sees only its own typed public view; every action is validated, committed and audited through the same code as human moves. Computers progress until a human decision or persisted trick deadline, in bounded SQLite transactions. Room reads resume ready computer turns after restart. When a computer deals, a human presses `Ask computer to deal`; settled results stay available until that request.

All 33 Rust tests, formatting, strict Clippy and the complete browser suites pass. Tests cover one/two humans at both table sizes, four audited deals per configuration, privacy, capacity validation, old snapshots, restart/concurrent closure, durable retries and re-hashed forged dealer requests. Browser suites cover the original human-only flows plus solo/two-human games. Public HTTPS live flows also completed both computer games; the deployed verifier accepted one deal/30 moves in each (88 events solo, 87 with two humans). Old sessions and the exact original audit remained valid after the upgrade. New live screenshots and proofs are in ignored `artifacts/deployments/20261114-computers/`.

That release image manifest digest was `sha256:6c7f197a66f61470cd765ce9d1947fec7798867914e45a26881076a4424c5f9d`; its runtime image/config ID was `sha256:5df88912a5b95046d075998f2f85a8e912b236637a6bc68eec5b673e7f43902f`. Pod `pastimer-844c4f7458-j74cr` was Ready with zero restarts on the same PVC. The new pre-upgrade backup passed SQLite integrity checking. Native build archives and previous images are retained, the temporary builder is stopped, and Colima was not started for this follow-up.

Reopen the Mini App to load the new assets, then create a computer table to test alone. A manual real-client computer-game playthrough is distinct from the passing signed synthetic browser flows. A Main Mini App/profile launch button remains optional; the default menu button is configured. Scheduled/off-host backups are not established by this setup. See [the current release record](horse-table-cues-2026-11-14.md).

## Gameplay follow-up

Source `5517348` replaces the earlier pass-only/burst computer policy with
conservative guaranteed-point bidding, useful distribution, ace/winner leads
and deliberate marriages. Each computer action has a persisted three-second
deadline; after the final call, stock remains visible for an additional three
seconds. Auction bubbles clear when the auction presentation ends; only the
winner and contract remain. The full auction stays in the record.

The Mini App fits its visible viewport. Cards play on one tap. Larger suit
indices/stock, hand-selected transfers with open/secret receipt steps, hidden
taken stock and seat-based last-trick inspection address the owner's layout
requests. Server-side latest-table membership and durable actor-scoped command
IDs recover the table after reopening, including fresh browser storage.

All 41 Rust tests, strict Clippy and full browser suites pass. Three/four-human
deals and solo/two-human computer games complete, audits verify, and normal
controls/cards fit seven portrait/landscape sizes from 320x480 to 1024x768.
Reviewed screenshots are updated. See the [current release record](horse-table-cues-2026-11-14.md)
for live verification and deployment identity.

## Receipt/count follow-up

The owner requested no sender or tap instructions on incoming cards. The card
itself now glows, the hand dims, sorting hides and other gameplay controls stay
disabled until acceptance. Accessible labels remain. Open/secret reveal steps
are unchanged.

A fresh browser could previously resurrect any old transferred card, including
one already played, and count that ghost alongside the real hand. The server
still dealt seven cards correctly. Read-only aggregate production counts showed
seven initially for all eight deals checked; the live solo hand had four left
after four plays. A stale played gift explains the displayed five. Source
`92d75bd` counts the server hand directly and only shows receipts for held cards
before the recipient's first play. A failing-before/passing-after regression
reopens a five-card hand with empty browser storage. The complete browser suite
passes, including the new receipt UI/count checks at three and four seats.
See [the current release record](horse-table-cues-2026-11-14.md) for live proof.

The final image also constrains transfer buttons when recipient names are long,
including narrow landscape. Browser checks now use long player names and wait
for controls to become actionable before recording command versions.

## Transfers and marriage default

Drag distribution cards onto the recipient seat/hand with mouse or touch. Tap a
staged card to turn face up for open transfer or face down for secret transfer;
drag back into your hand to cancel. Staged cards leave the visible hand/count.
The lower panel is removed; one Give two cards action on the felt commits both
atomically. Invalid/cancelled gestures do not commit. Tap selection remains as
an accessible fallback.

The owner reported a missed hearts marriage. Scoped read-only analysis of their
requested last deal confirmed an eligible court lead was sent with announce=false
because the UI defaulted off. Eligible marriages now default on independently
for each suit; tap either highlighted court to announce, or turn its toggle off
first to opt out. Next turns reset the default. Backend eligibility/rules are
unchanged. Current trump is prominent via crossed gold rings and a large suit.

The complete browser suite, focused multiple-marriage UI fixtures and actual
human games with default marriage leads pass. Mouse/touch/cancel/invalid-drop,
drag-back, face toggling, no lower panel, privacy, retries, audits and seven
viewport geometries are covered. See the current release record for live proof.

## Table clarity and trick collection

Remove the central contract/name and preparation contract text. Contract order
is shown at the owner's seat, including barrel/next-dealer cases. Marriage rings
are equal circles viewed from above with an over-under crossing. Large rings
sit on the announcing court card; the persistent trump stays top-centred.

Played cards remain by their owners (left/right/lower), independently of play
order and including the sitting-out dealer view. The leader stays highlighted.
A completed trick highlights the winning hand/card and flies into that hand in
the final part of the same three-second closure interval. Reduced motion keeps
static winner/status cues. Public `trickLeader`, `trickWinner`, `announcingCard`
fields are derived from existing public state and restore after reopening.

All 41 Rust tests, strict Clippy and the full browser suite pass. Focused geometry
checks verify round equal rings, no card/seat/trump overlap across seven sizes,
collection endpoints and reduced motion. The real four-player view is tested
with the dealer sitting out. See the current release record for image identity,
live verification and persistence proof.
