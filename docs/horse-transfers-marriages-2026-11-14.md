# Horse transfer and marriage update

Date: 2026-11-14. Origin: https://1000.vartanyan.com. Telegram: https://t.me/pastimerbot.
Previous release: [receipt/count fix](horse-receipts-2026-11-14.md).

## Behavior

Drag a card from your hand onto the recipient's seat/hand, using mouse or touch.
Staged cards sit beside those hands; the separate lower transfer panel is gone.
Tap a staged card to turn it face up for an open transfer or face down for a
secret transfer. Drag it back into your hand to cancel, or replace a choice with
another card. Invalid drops and cancelled touch gestures leave the card in place.
Both choices remain editable until the single Give two cards action on the table
commits them atomically through the existing rules. A tap-based selection fallback
remains available. The visible hand/count excludes locally staged cards.

Eligible marriages now default on for each eligible suit. Leading its highlighted
king or queen sends the announcement automatically; switching that suit's option
off preserves voluntary play without announcement. A new turn resets the default.
The server still decides eligibility: a prior won trick, the lead, and both court
cards held. Ineligible courts never announce automatically.

The owner asked to inspect their latest played deal. Read-only analysis confirmed
an eligible hearts court lead had been submitted without announcement because
the old UI toggle defaulted off. No historical game data or scores were changed.
No player identities or other players' private cards were exported for diagnosis.

The current trump is prominent on the felt: crossed gold rings beside a large
suit glyph. Before any announcement it reads NO TRUMP. Contract details remain
visible in the table/contractor seat. See the [transfer preview](screenshots/02-distribution.png)
and [hearts marker UI fixture](screenshots/06-hearts-trump.png).

## Release

- Application source: `60c380824d4b0b3f55d36a79db950c5eb905f637`.
- Drag implementation: `76064a6`; marriage defaults/marker: `60c3808`.
- Image: `docker.io/library/pastimer:commit-60c380824d4b0b3f55d36a79db950c5eb905f637-amd64`.
- Git archive SHA-256: `550f092f4c164035bfc0016050eae84b9aeb875381d2d8c6a1002c847b1220b9`.
- Manifest: `sha256:8763e94b2e823b08d270cfe7bc9171b1c17f9213f8c92545c7811b0a8e35d41d`.
- Runtime image/config ID: `sha256:dc2d92d42695423d6564522bf5ddf7916a30a8cd4426ebb5e908ceabb80b1ebe`.
- Pod: `pastimer-77479dd5bf-xgpfk` - Ready, zero restarts.

Only Pastimer's immutable image changes. The existing one-writer Recreate
strategy, `pastimer-data` PVC, Secret, ingress and TLS are retained. No backend
rule or schema changes were needed. The native amd64 image comes from a clean
Git archive, built with the isolated temporary Horse BuildKit worker.

## Checks

The full browser suite passes: legacy demo, complete three/four-human games,
solo/two-human computer games, private transfers, exact retries, fresh-storage
reconnection and audited outcomes. Transfer checks cover real pointer gestures
for mouse and touch, cancellation, invalid targets, dragging back, turning both
ways and removal of the lower panel, with controls/cards inside seven viewports.

Full human games select eligible court leads when available and verify that
trump changes. A controlled legal-view/command UI fixture independently verifies
multiple suit defaults, direct heart announcement, explicit opt-out, next-turn
reset, ineligible courts and the rings/suit indicator at seven screen sizes.
This fixture is distinct from the real rules-engine games and audit verification.
The unchanged Rust backend has its existing passing 41-test baseline; frontend
assets are rebuilt into the binary. JavaScript syntax and formatting checks pass.

Public HTTPS health, production authentication restrictions and all five assets
pass; assets match the committed source byte-for-byte. The running image ID
matches this release, with a Ready pod and zero restarts. Original sessions,
completed table and exact historical audit remain intact.

Live three/four-player browser games passed mouse/touch transfers, cancelled and
invalid drops, drag-back, face toggling, open/secret privacy, no lower panel,
fresh-storage recovery and seven viewport geometries. Each full proof passed
locally and inside the deployed verifier:
- 3 players: `MRR6JV5D`, one deal, 87 events, 30 moves.
- 4 players: `9M567FVN`, one deal, 91 events, 31 moves.

Eligible court leads are checked whenever available in those full games. The
focused fixture guarantees coverage of the marriage UI even when a random deal
has no eligible marriage.

## Persistence and operation

The consistent pre-upgrade SQLite backup passed integrity checking:
`/var/backups/pastimer/pastimer-20261114-pre-drag-marriage.sqlite` (1056768 bytes).
The earlier pre-drag backup and previous images/build archives are retained.
Preserve the PVC for any rollback.

The temporary native builder is stopped with no isolated build tasks running.
Assets use no-store, so no cache invalidation is required. Colima was not started.
Canonical manifests are in [deploy/horse](../deploy/horse/), mirrored to the local
`/Users/michael/dev/k3s-horse/pastimer/` project. Ignored release evidence is under
`artifacts/deployments/20261114-drag/`. Live browser identities are synthetic;
these checks are distinct from a manual Telegram-client playthrough.
