# Local-session handoff

Updated 2026-11-20. Repository: `mvartanyan/pastimer`. Local Horse continuation: branch `codex/horse-deployment`; pulled `main` at `6376528`.

## Where the work stands

The Rust multiplayer platform, the owner's complete Thousand rules, audit verifier and redesigned card-table UI are implemented. **Pastimer is deployed and verified on Horse at https://1000.vartanyan.com**, using the established local `/Users/michael/dev/k3s-horse` setup. [@pastimerbot](https://t.me/pastimerbot) has a verified default `Play / Играть` menu button pointing to that origin.

This checkout now implements Podkidnoy and Perevodnoy Durak from the synchronized [English](durak-rules.md) and [Russian](durak-rules.ru.md) specifications, including two to six seats, computer opponents, per-player/per-rank throw-in requests, rematches, independent audit replay and a two-row scrolling hand. Both variants are deployed and live-verified on Horse; see [the Durak release record](horse-durak-2026-11-15.md). See the Durak section below for implementation and validation.

The current release source is `29337b633c96b0b2bd159e52ea86f2851f5b8a6b`, adding trick-only seat counters, explicit deal score progression and private read-only current-hand observation for the sitting-out dealer. [The deal-view release record](horse-deal-view-2026-11-20.md) contains image identity, backup and live acceptance. Preceding source `7d38925c3526041e64ae8e712ef14dba4fc30274` clarified Durak turns/layout and prevented Thousand receipts from disabling legal cards or pausing forced play. Preceding source `5b27aba9a63c341a98dca9a8f61fcf49ac8b287a` enlarged Durak cards and corrected unpaired-card hit areas. Source `06354f6c20a7cf2f922606c24369dec47b87efe8` introduced both Durak variants and their computer opponents. The preceding source `99a8723a27b44fd36a36b8cdbf7c3847e742dd14` introduced the following shared behavior: In-progress archive entries now resume play directly, with Record separate; resuming also remembers the selected older table for server-side reconnect. A sole legal human card is played automatically after two seconds through the normal durable command path. The timer survives local repaints, revalidates the live view, and pauses for inspection or incoming-card handling. It cancels when leaving/changing the table or while hidden/pending a command.

Seat counters now show trick-card points only, including immediately after settlement. The result panel shows the previous cumulative score, trick points plus each marriage/dealer bonus, any rule adjustment and the new cumulative score. This supersedes the earlier full-deal seat counter. Player panes sit inside the table rail without the crossing inner decorative border. Announcing-card rings have a transparent background and reduced opacity, with a mask for ring crossings. Long-press (550 ms) or right-click an eligible court to open Card options, then choose Play without announcing. Normal taps retain the default announcement, and the fixed ring options remain available. Stable hand slots, transfer behavior, one-second computer card plays, three-second auction/trick timing, four languages and all prior recovery/audit boundaries remain.

See [the preceding resume/forced-play release record](horse-resume-2026-11-15.md) for identity, backups and verification. [The preceding archive/language release](horse-playability-2026-11-15.md) records the original archive, languages, solo buttons, winner laurels and stronger computer bidding. Computer strategy still receives only its own typed view and public information. Archive/history/audit reads remain member-authorized and read-only; only an explicit resume updates the reconnect target.

Relevant implementation commits:

- `99a8723` - direct archive resume, two-second forced human cards, explicit marriage gestures, full deal totals and inset player panes.

- `7534a25` - clear translated mobile lobby headings; four-language lobby regression.
- `9851b8c` - stable playing field, captured points/barrel icons, game archive, winner celebration, solo shortcuts, four languages, stronger legal-card contrast and faster/more willing computer play.

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

## Agreed Durak rules - November 15

The canonical Durak specifications are [English](durak-rules.md) and [Russian](durak-rules.ru.md). Keep them synchronized. Both variants are selectable in the lobby as `durak-podkidnoy` and `durak-perevodnoy`. Their shared Rust state machine is in `src/durak/mod.rs`; `src/durak/verify.rs` replays settled games; `public/durak.js` renders the table, requests and hand. Existing demo and Thousand state decoding and commands remain supported.

- Use 36 cards, two to six individual players and six cards each. With six players, expose the dealer's last card as trump and leave it in that hand. Multiple equal-rank cards may open an attack.
- At most six attacking cards, including the first attack, capped by the current defender's hand size when becoming the defender. A transfer establishes the new defender's cap.
- The current attacker has priority. Helpers use an "I want to throw in" / "Хочу подкинуть" button; the attacker sees who requested and may approve or ignore each request. Requests and approvals are specific to a player and rank: each new rank requires a fresh request and approval, while approval for an existing rank lasts for that attack against that defender. Approved helpers act clockwise; a new defence rank restores current-attacker priority. Pending requests do not block closing the attack. Legal throw-ins remain possible after "Take" / "беру".
- A transfer requires playing a matching-rank card before any card has been beaten or pickup declared. Transfers are allowed from the first attack and may chain or return to an earlier attacker if the next defender has enough cards. The transferor becomes current attacker and controls requests against the new defender; previous requests and approvals are cleared.
- Draw from the original attacker clockwise, leaving the final defender until last, even when the original attacker became the defender. No trump-six exchange or "проездной".
- The lowest trump holder starts the first game and games after a draw; if no trump was dealt, start clockwise after the dealer. Rematches after a loss begin by attacking the previous loser. The final two finishing in the same successful defence is a draw.

Durak readability correction (owner feedback after the initial rollout): hand faces are now at least 76px wide on compact screens and 84-104px normally, with two overlapping rows and at least 44px of exposed upper-row hit area. Primary controls use 15-16px text and 46-48px hit areas; seat names use 12-14px text in a scrolling ribbon. Trump rank and suit are explicit text. Played pairs use the central space and horizontal scrolling, with the selected defence target kept visible and inspection scroll preserved across refreshes. Short portrait screens move sorting to the action row. Readability checks now cover card-face and text sizes, six-card attacks, clipping and a defence tap at 320x480, 390x650, 390x844 and 844x390. Odd hands of one, three and five cards also exercise taps near the bottom of the unpaired card. The correction is deployed and verified: both variants passed live create/start/card-play checks at 390x650, original Durak audits and historical Thousand recovery remain unchanged, and runtime/asset checks pass. See [the readability rollout](horse-durak-readable-2026-11-15.md).

Durak hand UX: use a fixed-height area with two rows of cards and horizontal scrolling for overflow, keeping the table and action buttons stationary as the hand grows. Support the full 36-card deck without the existing ten-slot limit. An expandable hand overview is not planned. The implemented hand preserves its scroll position across updates, defers repaints while the hand is being touched or dragged, and prevents a swipe from submitting a card. The layout is separate from the game rules.

Durak shares the existing durable command transactions, membership recovery, computer scheduling and read-only archive paths. Creation accepts `options: {"playerCount": 2}` (2-6) and optional `computerCount`. Commands use the `durak` envelope; `request` and `approve` carry a rank, and approvals also identify the requesting player. An approved helper still waits for attacker priority and clockwise order. The host can start a rematch after settlement; seats persist, loss counts accumulate, and the original attacker remains distinct from the current attacker for drawing. Durak computer card/control actions wait one second and accept only their own typed view.

The new `pastimer-durak-audit-v1` format reveals each settled game's full deck, movements, bouts and sealed commands while keeping an unfinished rematch private. The existing verifier validates every emitted game event and final record by replaying the rules, including requests, approvals and transfers. No database schema migration or production deployment is part of this implementation.

Validation: all 58 Rust tests pass, including nine pure Durak tests and three Durak integration tests. Coverage includes both variants at two to six seats, card conservation, six-card and defender-hand caps, pickup throw-ins, transfer restrictions/chains, drawing order, draws/rematches, per-player/per-rank requests, clockwise priority, hidden-information policy invariance, durable concurrent retries, SQLite reopening, sealed rematches and rejection of rehashed forged audit records. Formatting and strict Clippy pass.

The complete `npm run test:browser` suite passes, covering the demo and prior Thousand/marriage/archive/computer behavior plus both real three-player Durak games with offline-verified audits, reload and rematch. Controlled Durak views check a 36-card hand at 320x480, 390x844, 844x390 and 1024x768, exactly two rows, no page overflow, 44px minimum hand targets, scroll preservation, native touch and mouse dragging without card submission, dragging a disabled hand, new-rank request/approval buttons and English/Russian/German/Spanish localization. The controlled large-hand/request screenshots at `/tmp/pastimer-durak-large-hand.png` and `/tmp/pastimer-durak-mobile.png` were visually reviewed; these are browser test projections, not live Telegram captures. An initial loaded-run Thousand deadline timeout passed in a focused rerun and in the final complete suite; the new hidden settings label conflict found by regression testing was corrected.

The initial Durak rollout used source `06354f6`; signed production browser games completed against one computer (two seats) and five computers (six seats), with deployed audit verification and fresh-session recovery. The six-seat game exercised 13 approvals and five transfers. All eight public assets match source, production auth/origin checks pass, and the original historical memberships and exact audit survived. The original PVC/database were preserved with an integrity-checked pre-rollout backup. A manual real Telegram-client Durak playthrough remains separate from the verified signed browser flows. The pre-existing untracked root `thousand-rules.ru.md` is preserved.

## November 20 deal scores and dealer observation

The owner changed the counter preference to trick points only, excluding marriages and cumulative scores even on the settled table. Results instead show previous score, the full earning/loss arithmetic and the new total. When contract/barrel/limit/cancellation rules alter raw earnings, the adjustment is explicit. Names in results remain buttons for last-won-trick inspection. Final results retain winner laurels and fit compact viewports.

The sitting-out human dealer in a four-seat game sees only the current actor's hand, using their own empty hand area with stable slots and legal-card/marriage highlights. It is strictly read-only. Computers are always permitted. Humans must grant consent before shuffling, only for the upcoming deal and named dealer; no consent is the default. Settlement and cancellation clear it, including same-dealer redeals. Other viewers and computer policies receive no extra hidden cards. Consent is persisted and audited without changing old deal record formats.

All 61 Rust tests, formatting, strict Clippy and every browser-suite component pass. Dealer/results checks cover seven viewport sizes and all four languages. The real production four-human deal `3E3EMC5R` passed consent/privacy/highlight/no-play/reset checks; the deployed verifier accepted 91 events and 30 moves. Public assets, original fresh-auth memberships, exact historical audit, one ready replica, original PVC and TLS all pass. [Release details](horse-deal-view-2026-11-20.md). The version-notice branch remains isolated and undeployed. Preserve the unrelated untracked root Thousand file.

## November 16 usability correction

Owner feedback: Durak remained cluttered and unclear despite larger cards. The new UI has a dedicated two-line turn prompt based on the viewer's legal controls, names the selected attack card, and explains tap/Take/finish/transfer/wait states. All seats fit a three-column roster (two columns for two players), rather than disappearing into a horizontal strip. The hand uses separate rows when height permits, with 76px or 44px exposed upper-row areas on shorter screens. Landscape places the table beside the hand. Short screens can scroll the table vertically; a selected defence target is revealed automatically. Requests and approvals use one action button and an accessible dialog; sort and optional multiple-card attacks live under Options. Rules, server legal controls and computer policy are unchanged. All new copy is localized in four languages.

The Thousand screenshot showed a glowing received card and a dimmed, disabled hand even after trick play began. Receipt prompts now end when the phase becomes `trick`; all held cards become normal legal-card candidates, and receipts cannot pause the forced-card timer during play. Pre-play open/secret receipt inspection is retained. The reproduced no-trump/diamond-ace fixture enables all eight non-diamond cards without auto-selecting one. A clubs-led fixture with exactly one legal club and an unacknowledged delivery submits that club exactly at two seconds. Tests still cover stale choices, inspection/visibility, leaving the table, deadline preservation and durable commands.

Browser work is headless-only. The full browser suite passes, including complete games and audits for both Durak variants. The correction is deployed; both Durak interfaces pass live create/start/card-play checks and asset/runtime/persistence checks. Live Thousand table `H3S2KD44` completed against two computers with three automatic human plays and untouched receipt acknowledgements; the deployed verifier accepts its 88-event/30-move proof. Reload, original memberships and prior exact audits are preserved. See [the release record](horse-clarity-2026-11-16.md). Preserve the unrelated untracked root Thousand file.

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
| Language catalog and private game archive | `public/i18n.js`, `public/archive.js`, `src/engine.rs`, `src/store.rs` |
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

Release builds and the four signed-identity screenshot flows also passed during the UI work. A container build/runtime check was completed before `ba4ac97`; **rebuild the image from the latest checkout** for deployment so it contains the redesigned assets. The subsequent local continuation verified native local-platform checks, public HTTPS and Horse runtime behavior; the owner has since confirmed actual Telegram launch, identity and table creation. See [the November 14 release record](horse-table-cues-2026-11-14.md) for the computer follow-up and live browser checks.

To verify a downloaded audit independently:

```sh
cargo run --locked --bin verify-audit -- path/to/pastimer-XXXXXXXX.json
```

## Initial Horse continuation - November 14 history

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

Reopen the Mini App to load the new assets, then create a computer table to test alone. A manual real-client computer-game playthrough is distinct from the passing signed synthetic browser flows. A Main Mini App/profile launch button remains optional; the default menu button is configured. Scheduled/off-host backups are not established by this setup. See [the November 14 release record](horse-table-cues-2026-11-14.md).

## Gameplay follow-up - November 14 history

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

## Receipt/count follow-up - November 14 history

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
See [the November 14 release record](horse-table-cues-2026-11-14.md) for live proof.

The final image also constrains transfer buttons when recipient names are long,
including narrow landscape. Browser checks now use long player names and wait
for controls to become actionable before recording command versions.

## Transfers and marriage default - November 14 history

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

## Table clarity and trick collection - November 14 history

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

## November 15 continuation

All 46 Rust tests, formatting and strict Clippy pass. The complete browser suite covers demo, three/four-human deals, one/two-human computer games and solo play against three computers, with verified audits. Controlled fixtures check seven viewport sizes, unchanged field geometry with 7/1/0 cards, barrel/captured-point rendering, default/optional marriages, winner laurels, stronger disabled-card dimming, all four languages and persistence across reload. Translated mobile lobby headings are visually reviewed without overlapping decoration. The full Horse evidence is in the linked November 15 release record.

Keep the original untracked root `thousand-rules.ru.md` untouched. The canonical agreed rules remain the two files under `docs/`.

After the final pod replacement, fresh Telegram authentication with empty browser storage restored all three original historical memberships and the exact audit. The initial cookies had crossed their 24-hour expiry; this was verified separately from room persistence.

## Resume and forced-play continuation

The focused browser suite uses real room creation, history, resume and reconnect, followed by clearly labelled UI fixtures with a controlled clock. It verifies exactly one automatic command at two seconds, a deadline preserved through sorting, no selection among multiple legal cards, stale-choice cancellation, inspection/lobby/receipt pauses, and right-click/550-ms hold without a release-click play. Controlled fixtures wait for the preceding command's acknowledgement before reload; a pending durable command correctly blocks automatic play.

Seven viewport fixtures verify player-border insets, transparent marriage rings and one combined 125-point value for 25 trick points plus a 100-point marriage. All existing human/computer game suites and 46 Rust tests pass. The new release record separates live rules-engine deals from controlled UI fixtures.

Live acceptance completed three deals across three/four seats, including three automatically played human cards in each active-human scenario. The deployed verifier accepts all proofs. The four-seat second deal, exact audit, fresh Telegram recovery and Record -> Resume game also pass.
