# Horse table cues update

Date: 2026-11-14. Origin: https://1000.vartanyan.com. Telegram: https://t.me/pastimerbot.
Previous release: [transfers and marriage defaults](horse-transfers-marriages-2026-11-14.md).

## Table behavior

Contract information appears only at its owner's seat/hand. The duplicate centre
contract/name and preparation contract text are removed. The owner's contract
also remains visible when that player is on the barrel or becomes the next dealer.

The marriage symbol uses two equal circular gold rings viewed from above. The
second ring visibly passes over the first at the crossings; there are no tilted
ellipses. An announcing king/queen carries a large version of those rings until
the trick is collected. The persistent trump marker stays at the top centre,
between the opponent seats when space permits, otherwise immediately below them.

Played cards are placed beside their owners: left, right and the lower player.
They retain those positions regardless of play order, including the sitting-out
dealer's four-seat view. Visible card-name/player captions are removed. Accessible
card descriptions retain the owner and first-card information.

The leader has a blue outline and LEAD badge while cards are being played. Once all three
cards are played, the winning hand and card glow gold with TAKES TRICK. During the
last 800 ms of the existing three-second closure window, all three cards shrink
and move into that hand. The next leader is the winner. Reduced-motion users get
the same explicit winner/card/status cues without flying cards.

The existing rules, point calculation and persisted closure deadline are unchanged.
The public view adds `trickLeader`, `trickWinner` and `announcingCard`, derived from
public plays/marriage events so the cues also work after reopening mid-trick.
No snapshot or database migration is required.

## Release identity

- Application source: `f54714d388be37749979e8fe206a323c73d7de9e`.
- Image: `docker.io/library/pastimer:commit-f54714d388be37749979e8fe206a323c73d7de9e-amd64`.
- Git archive SHA-256: `4f18501f3790cb9ac8b487f89b3f2defda784e5bb24b2b52ff522b2d945d65cc`.
- Manifest digest: `sha256:b342f0f711a527efab73ca350e2a00684d149c41d6fe5f2c6199505db540cd64`.
- Runtime image/config ID: `sha256:de8df5f00439111f4bd038b890fbd12e8dfcc6921997ae60792a82522f7b6af4`.
- Pod: `pastimer-7546b4fc4c-9pmbc` - Ready, zero restarts.

The immutable amd64 image was built natively on Horse from a clean Git archive.
Only Pastimer's image changes. The original one-writer Recreate strategy,
`pastimer-data` PVC, Secret, ingress and TLS remain in place.

## Verification

All 41 Rust tests, formatting and strict Clippy pass. Projection checks cover
first leader, announcing card, actual winner and next leader through full three-
and four-player deals. Existing privacy, audit, persistence and rule tests remain.

The complete browser suite passes: demo, three/four-human games, marriage fixtures,
solo/two-human computers, drag/turn transfers, retries, recovery and audit replay.
Cards/controls fit seven portrait/landscape sizes. Additional geometry checks
verify that played cards do not overlap seats, each other or the centred trump.
The rings render as equal circles. The collection fixture verifies all three
animation endpoints at the winner's hand and the reduced-motion alternative.
The sitting-out dealer's viewport checks run between plays, outside the timed
closure window. [Reviewed previews](screenshots/README.md) include the new table
and [marriage/winner fixture](screenshots/07-trick-owners.png).

Public HTTPS health, authentication restrictions and all five frontend assets
pass; assets match the committed source byte-for-byte. The pod is Ready with
zero restarts and the expected runtime image ID. Old sessions, the completed
table and its exact historical audit survived unchanged.

Full live three/four-player browser games passed owner positions, leader and
winning-hand cues, all three collection targets, privacy, transfers, exact
retries, fresh-storage recovery and viewport bounds. The sitting-out dealer view
also passed. Both proofs passed locally and inside the deployed verifier:
- 3 players: `CN24AF9M`, one deal, 87 events, 30 moves.
- 4 players: `Z5U28QDA`, one deal, 91 events, 31 moves.

## Operations

Consistent pre-upgrade SQLite backup (integrity check: ok):
`/var/backups/pastimer/pastimer-20261114-pre-table-cues.sqlite` (2641920 bytes).
Previous images, backups and native build archives remain available. Preserve the
PVC for rollback. The image includes the same production auth and no-store assets;
no cache invalidation or credential update is needed.

The temporary native builder is stopped with no isolated build tasks running.
Colima was not started. Canonical manifests are [deploy/horse](../deploy/horse/),
mirrored to `/Users/michael/dev/k3s-horse/pastimer/`. Ignored logs/proofs/screenshots
are under `artifacts/deployments/20261114-tricks/`. Controlled UI fixtures are
labelled separately from real rules-engine games. Live browser identities are
synthetic; a manual Telegram-client playthrough is a separate check.
