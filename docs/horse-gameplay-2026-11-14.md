# Horse gameplay update

Date: 2026-11-14. Origin: https://1000.vartanyan.com. Telegram: https://t.me/pastimerbot.

This is a historical release record. See the [current receipt fix](horse-transfers-marriages-2026-11-14.md). Earlier releases remain documented in the
[initial deployment](horse-2026-11-14.md) and [computer-seat rollout](horse-computers-2026-11-14.md).

## Release identity

- Application source: `5517348481d3cab2afddc221e0ba3798d38762cf`.
- Image: `docker.io/library/pastimer:commit-5517348481d3cab2afddc221e0ba3798d38762cf-amd64`.
- Manifest digest: `sha256:db8739910def121c03b8cdc403a6f7bc0df416214d222440a24493c30bce4c3b`.
- Runtime image/config ID: `sha256:7df5f3f14793e2360919337d14ab75665468c47a7735f7964ba7781643ff8d4a`.
- Git source archive SHA-256: `30c51b8c3cf837594c512fc7b5c15695068dbe43ff1eb8abf01060423ac96d0c`.
- Pod: `pastimer-5ff9589f49-5ph28`.

The image was built natively for linux/amd64 on Horse from a Git archive and
imported into k3s. The existing single-writer Recreate deployment and original
`pastimer-data` PVC were preserved. Ingress, TLS, credentials and unrelated
workloads were unchanged. The only database migration is the additive
`player_rooms` membership index; existing snapshots remain compatible.

## Gameplay changes

- Latest bid/pass/here calls appear in seat speech bubbles, three seconds apart.
  The final call gets its three-second display, then bubbles clear and only the
  contract winner and amount remain. Every call stays in the record.
- Opened stock uses larger cards and disappears after being taken. All cards
  have substantially larger rank and suit indices, including court cards.
- Normal gameplay uses the visible Mini App viewport with no page scrolling.
  Last won tricks open from player seats/hand indicators; the separate pile row
  is removed. Record and trick details open only on demand.
- Distribution selects two cards directly from the hand, with independent
  open/secret choices. Recipients see the source and visibility explicitly.
  Secret receipts start face down: tap to reveal, then tap to sort into the hand.
  Open receipts start face up and take one tap to sort. Private cards are absent
  from other players' projections.
- Tap a legal card to play immediately. Marriage declaration remains voluntary:
  enable its toggle before tapping the king or queen.
- Tables restore after authenticated reopening, including an empty browser
  storage context. Actor-scoped local command IDs still support exact retries.
  The server remembers the latest created/joined table and falls back to older
  stored membership. Telegram closing confirmation is enabled where supported.
- Computers estimate conservative guaranteed points and attainable marriages,
  retain those combinations when giving cards, cash unbeatable leads before
  declaring trump, and take affordable winning tricks. They see only their own
  view and public played cards. Computer deadlines persist across restart and
  execute one action at a time, never a burst of overdue calls.

The stock transfer and play rules are unchanged. Receipt acknowledgment is a
presentation step; both cards still transfer atomically through the rules engine.
Computer actions use the same validated, transactional, audited command path as
human actions. No external AI service is involved.

## Validation

Formatting, strict Clippy, all 41 Rust tests and the complete browser suites pass:

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
npm run test:browser
```

Browser checks complete three- and four-human deals, solo/two-human computer
games, audits, lost-response retries, fresh-window recovery and mixed-visibility
card receipts. Geometry assertions check cards and controls inside the viewport
at 320x480, 320x568, 390x650, 430x932, 568x320, 844x390 and 1024x768, across
auction, stock, distribution and play. The [reviewed gallery](screenshots/README.md)
uses 390x650 captures from the real local game with signed synthetic identities.

Live HTTPS checks passed for health, game catalogue, authentication restrictions
and all five embedded assets; assets match the committed source byte-for-byte.
The running image ID matches this record, with a Ready pod and zero restarts.
Original sessions, the completed table and the exact historical audit survived.

Both live signed-identity browser games completed, reloaded and recovered their
tables in fresh contexts containing cookies but no local/session storage:

- Solo with two computers: table `84D2NZ7C`, one deal, 87 events, 30 moves.
- Two humans with one computer: table `NTA8J9YP`, one deal, 88 events, 30 moves.

Both proofs passed locally and inside the deployed `verify-audit` binary.
Computer auction timestamp gaps were checked against the three-second pacing.
No browser page errors occurred.

The owner confirmed actual Telegram launch/identity/table creation before this
release. Signed synthetic browser tests are distinct from a manual Telegram
client playthrough of the new UI. Telegram viewport/closing APIs follow the
[official Mini Apps documentation](https://core.telegram.org/bots/webapps).

## Persistence and rollback

A consistent online SQLite backup passed `PRAGMA integrity_check` before rollout:
`/var/backups/pastimer/pastimer-20261114-pre-gameplay.sqlite` (425984 bytes).
The previous release image and native build archives remain available. Preserve
the PVC when rolling back. Old versions ignore the additive table, but the prior
computer policy has different pacing/strategy and does not implement this UI.

The canonical overlay is [deploy/horse](../deploy/horse/). Its rendered mirror and
operating notes are in `/Users/michael/dev/k3s-horse/pastimer/`. Build/rollout logs,
HTTPS results, synthetic session states and audits are ignored local evidence
under `artifacts/deployments/20261114-gameplay/`; session files are mode 0600.

The temporary native builder is stopped and its isolated task list is empty.
Source, cache and rollback archives are retained. Colima was not started, so no
Colima cleanup or cache invalidation was needed. Frontend responses use no-store.
Scheduled/off-host backups are not established by this update.
