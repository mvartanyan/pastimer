# Horse receipt and hand-count fix

Date: 2026-11-14. Origin: https://1000.vartanyan.com. Telegram: https://t.me/pastimerbot.
Previous release: [gameplay update](horse-gameplay-2026-11-14.md).

## Change and diagnosis

Incoming cards have a gold outline/glow, with the ordinary hand dimmed and other
gameplay controls disabled. Sender and tap instructions are removed. Transfer recipient buttons now also
constrain long names to their tiles, including narrow landscape screens. Accessible
button labels remain. Secret cards still reveal on the first tap and sort into
the hand on the second; open cards start face up and sort on the first tap.

The hand-count problem was a stale receipt on reopening, not a five-card deal.
The old client read all historical transfers without checking whether play had
begun or the card remained held. With fresh browser storage, it resurrected an
already accepted gift. A played gift could also be counted on top of the real
hand, creating a ghost card and disabling normal play.

A read-only aggregate check on Horse returned initial sizes `[7,7,7]` in all eight
stored deals checked. The in-progress solo deal had four real cards remaining
and four already played. A resurrected played gift would make those four display
as five. The check exported no identities, cards, snapshots or table codes.

The client now takes the count directly from the server hand. Only a recipient
still holding eight cards, before their first play, can see an incoming receipt;
the transferred card must actually be in that hand. Reopening after play begins
restores the real hand without replaying old receipt steps. The server dealing,
card ownership, rules, persistence and audit code are unchanged.

## Release identity

- Source: `8458703b506fa31fd9ffab4f2d2455fe1a2af2ef`.
- Image: `docker.io/library/pastimer:commit-8458703b506fa31fd9ffab4f2d2455fe1a2af2ef-amd64`.
- Source archive SHA-256: `7b6aabc39db8f6813d6ca9f1208482e83765b80dd97bcfff7b43fe9f2c298064`.
- Manifest digest: `sha256:ba6c3116bd03dfb8920e886edb6836d9203d80ac92be5930cab17c80e2ef1344`.
- Runtime image/config ID: `sha256:c658330e0895a5b8bdbb74ac7b8bc6d5efbd7b571d9153c05beef76709bec7de`.
- Pod: `pastimer-65bb4567bc-4l8gf` - Ready, zero restarts.

## Validation

A browser regression failed against the previous binary: after reopening with
five actual cards, the UI counted six because of the stale receipt. It passes
with this fix. The browser suite explicitly checks seven initially dealt cards,
eight after distribution, a label-free highlighted receipt as the only enabled
gameplay action, then a five-card mid-deal reconnect with no incoming card.
Both three- and four-player tables run these checks across seven viewport sizes,
including landscape. Open/secret privacy, audit replay and lost-response retries
remain covered. The [receipt preview](screenshots/05-incoming-card.png) was visually
reviewed. Rust backend logic is unchanged from the passing 41-test gameplay release.

Formatting and JavaScript syntax checks pass. The complete browser suite passes
for the demo, full three/four-player games, solo/two-human computer games,
private transfers, audits, retries and fresh-storage recovery.
Live HTTPS health, production authentication restrictions and all five embedded
assets pass; assets match the final checkout byte-for-byte. The actual runtime
image/config ID matches the release, with a Ready pod and zero restarts. Old
sessions, their completed table and exact historical audit remain intact.

Full three- and four-player signed synthetic browser games passed on this image,
including initial/received hand counts, open/secret receipts without labels,
long-name geometry, mid-deal recovery without browser storage, command retries
and complete audits. Both exported proofs also passed inside the deployed
`verify-audit` binary:
- 3 players: `LNB6LSFG`, one deal, 87 events, 30 moves.
- 4 players: `84F3DZ7K`, one deal, 91 events, 31 moves.

## Deployment and persistence

Only Pastimer's immutable Deployment image changes, preserving the original
single-writer Recreate strategy and `pastimer-data` PVC. No schema migration,
secret update or cache invalidation is needed; asset responses use no-store.
A consistent online backup is taken before rollout. Prior images and archives
remain available for rollback; preserve the PVC.

The consistent pre-rollout backup passed integrity checking:
`/var/backups/pastimer/pastimer-20261114-pre-receipts-final.sqlite` (724992 bytes).
The earlier pre-receipt backup is also retained.
The temporary native builder is stopped, with no isolated build tasks running.
Colima was not started; no Colima cleanup was needed.

The canonical overlay is [deploy/horse](../deploy/horse/); its rendered mirror is
in `/Users/michael/dev/k3s-horse/pastimer/`. Ignored verification evidence is under
`artifacts/deployments/20261114-receipts/`. Live browser checks use signed synthetic
identities; they are distinct from a manual Telegram-client playthrough.
