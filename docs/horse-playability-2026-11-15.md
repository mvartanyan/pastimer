# Horse playability, archive and language update

Date: 2026-11-15. Live: https://1000.vartanyan.com. Telegram: https://t.me/pastimerbot.
Previous release: [table cues and collection](horse-table-cues-2026-11-14.md).

## Behavior

- During a deal, player blocks show points in captured cards. The tooltip also contains the full deal total and match score; Record and between-deal blocks retain cumulative scores. A small barrel drawing replaces the width-changing BARREL label.
- The table, hand and action row keep fixed dimensions. Ten hand slots preserve the positions of remaining cards, leaving pale placeholders as cards are played. Empty hands and match endings retain the same field footprint. Sorting or adding stock/received cards can deliberately repack the hand.
- Marriage announcements remain automatic when eligible, with voluntary opt-out through the fixed ring button beside Sort. The lower jumping bonus controls are removed. Existing trump/played-card/leader/winner cues remain.
- Legal cards have a stronger gold outline; unavailable cards on your turn have heavier grayscale/dimming. Computer card plays wait one second. Auction calls and trick collection retain three seconds; opened computer stock retains its presentation window.
- Direct solo buttons create and deal against two computers (three seats) or three computers (four seats). Larger seat buttons also make custom table size explicit. Choosing three computers automatically selects four seats.
- Computers combine the guaranteed-point floor with a modest estimate of likely tricks and an attainable marriage. They reassess their final contract after stock distribution. Distribution preserves useful winners/marriages; play still realizes winning leads and announces marriages. The policy only receives its own typed player view, never a Room or other hands.
- A finished Thousand match shows a gold laurel around the winner's score and clear New table / Played games actions. The private archive includes players, scores, start/end times, searchable public events, chained event hashes, deck commitments and downloadable audits. Unfinished private proofs remain sealed. History, record and audit reads never advance an unattended game.
- A persistent English/Russian/German/Spanish switch localizes the interface, date formatting, controls and ordinary errors. Raw event types/data/hashes remain auditable source data. The three rejected promotional lines are removed. Decorative cards no longer overlap translated mobile lobby headings.

## Release identity

- Final source: `7534a259b64c2b5623a81b716fb761ae68b2cf26`.
- Main implementation: `9851b8c284decbcd828b08a617b3149b844236fd`; final source adds the mobile heading correction and its regression check.
- Image: `docker.io/library/pastimer:commit-7534a259b64c2b5623a81b716fb761ae68b2cf26-amd64`.
- Clean Git archive SHA-256: `44facf61304231ad05dedebf9f914f9d5e0d75e69f2a84154de640898a6e9a46`.
- Manifest digest: `sha256:bfbbac49a2a43b7dbf0e00040100d29ef0542bdf7bae5eda3634d40558ccbe10`.
- Runtime image/config ID: `sha256:bae881b83fd9e3935b5831628afa17878867825d69449e03ae4f01c7083ab489`.
- Final pod: `pastimer-7b8f878565-wdzpg`, Ready with zero restarts.

Both immutable amd64 images were built natively on Horse from committed source archives. Only the Pastimer image changed. The one-writer Recreate strategy, original `pastimer-data` PVC (`pvc-fe1cefbc-cfc7-4cdc-bef4-ae4840f2675d`), Secret, ingress and certificate remain in place. No schema migration or bot credential/menu change was required.

## Verification

All 46 Rust tests pass, with formatting and strict Clippy. Added coverage includes member-scoped archive pagination, finished-game dates/scores/hashes, private-proof isolation, read-only archives/audits, exact one-second computer play scheduling, stronger bidding/final orders and a hidden-information invariant. Redistributing unseen opposing hands and sealed stock leaves the computer's view and action unchanged.

The complete browser suite passes: demo; three/four-human deals; one human/two computers; two humans/one computer; one human/three computers. It verifies rules-engine actions, transfers, receipt reveal/add, retries, recovery and audit replay. Seven viewport fixtures cover marriage/owner/leader/collection cues, 7/1/0-card fixed geometry, captured scores/barrel icons, laurels, contrast, all four languages and persistent selection across reload. All four mobile lobby versions were visually reviewed. [Reviewed previews](screenshots/README.md) distinguish controlled fixtures from real game flows.

The main image also passed the same full live human/computer paths with signed synthetic identities. The final image changes only mobile decorative-card CSS; final public assets, translated lobby/archive, recovery and deployed proof replay were checked again. Live accepted proofs, replayed inside the final deployed image:

| Scenario | Table | Deals | Events | Moves |
| --- | --- | --- | --- | --- |
| 1 human / 2 computers | `2NAZ57JC` | 1 | 87 | 30 |
| 2 humans / 1 computer | `QP5CPABS` | 1 | 88 | 30 |
| 1 human / 3 computers | `Y3VP55B9` | 1 | 89 | 30 |
| Finished demo / archive | `NJHA332J` | 1 | 23 | 10 |
| 3 human players | `LCWNDWFZ` | 1 | 88 | 30 |
| 4 human players | `4XPHHDSP` | 1 | 92 | 31 |
| Final-image demo / archive | `UZZXN4RB` | 1 | 23 | 10 |

Public HTTPS health, production authentication restrictions and all seven embedded assets match the final source byte-for-byte. Archive/history/record access requires membership/authentication. Original historical table and exact audit are preserved. Original cookies eventually exceeded their fixed 24-hour TTL; recovery was checked through fresh Telegram authentication with empty browser storage, separately from session longevity. Synthetic browser tests do not replace a manual Telegram-client playthrough.

## Operations

Consistent pre-upgrade SQLite backups, both integrity-checked:

- `/var/backups/pastimer/pastimer-20261115-pre-playability.sqlite` (5980160 bytes).
- `/var/backups/pastimer/pastimer-20261115-pre-playability-final.sqlite` (5980160 bytes).

Preserve the PVC and backups for rollback. Previous image/archive versions remain available. Scheduled off-host backups are still outside this setup.

Canonical manifests are [deploy/horse](../deploy/horse/), mirrored with this record to `/Users/michael/dev/k3s-horse/pastimer/`. Ignored logs, proofs and screenshots are in `artifacts/deployments/20261115-playability/`. The temporary native BuildKit worker is stopped with no isolated build tasks left running. Colima was not started. Assets use no-store, so no cache invalidation is required; reopen the Mini App to load the updated interface.
