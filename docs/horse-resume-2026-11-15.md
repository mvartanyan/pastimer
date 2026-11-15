# Horse resume and forced-play update

Date: 2026-11-15. Live: https://1000.vartanyan.com. Telegram: https://t.me/pastimerbot.
Previous release: [playability, archive and languages](horse-playability-2026-11-15.md).

## Behavior

Tapping an in-progress archive entry now resumes the table directly. Record is a separate action, and an unfinished record has a prominent Resume game button. The existing join endpoint recognizes the member and remembers the chosen older table, so a fresh browser with the same authenticated identity recovers it. History/record/audit inspection itself remains read-only.

When a human has exactly one legal card, the visible, unobscured table plays it after two seconds. The deadline survives sorting/local repaint. The browser fetches and revalidates the current view before sending the normal durable play command. Multiple legal cards never trigger a choice. Leaving/hiding the table or changing its state cancels the timer; open inspection dialogs, pending commands and received-card reveal/add handling pause it. A lost command response retains the normal exact retry behavior. The server remains authoritative about legality.

Player scores during a deal now use the full current-deal total, including marriages and any dealer bonus, in one sum. Cumulative match totals remain in Record and between deals. Player panes are inset inside the rounded table rail, with a narrower short-screen self pane; the inner decorative line that crossed their borders is removed. Stable hand slots and transfer/reveal behavior are retained.

Marriage rings on the announcing card have a transparent background and partial opacity. An SVG mask preserves the crossing without an opaque patch over the card artwork. Normal taps still announce an eligible marriage by default. Long-press for 550 ms or right-click the eligible king/queen, then choose Play without announcing for that card. The release click cannot play through the dialog. The fixed ring options remain available. New interface text is translated into English, Russian, German and Spanish.

## Release identity

- Source: `99a8723a27b44fd36a36b8cdbf7c3847e742dd14`.
- Image: `docker.io/library/pastimer:commit-99a8723a27b44fd36a36b8cdbf7c3847e742dd14-amd64`.
- Clean Git archive SHA-256: `b8cdcad89596e4237d865f0b2f06166ded9ed2339061ff33a29da22101bb941c`.
- Manifest digest: `sha256:35fcc79ee99b03ea1d4dfeb42fd22764baab51786c37ff5d8c3c7975b58f742b`.
- Runtime image/config ID: `sha256:3a2b52e1ae320d414a282df4101ecd9cfb34a642cd33f5f3455b20257134e114`.
- Pod: `pastimer-6467769497-p5l78`, Ready with zero restarts.

Built natively on Horse from committed source. Only the Pastimer image changed. The original one-writer Recreate deployment, `pastimer-data` PVC (`pvc-fe1cefbc-cfc7-4cdc-bef4-ae4840f2675d`), Secret, ingress and certificate remain. No database/schema, scoring-rule or bot-credential change was needed.

## Verification

All 46 Rust tests, formatting and strict Clippy pass. Every browser suite passes: demo, three/four-human games, marriage/layout fixtures, the new resume/forced-play suite, and solo/two-human computer games including four seats. Seven portrait/landscape sizes verify that hand-info borders stay inside the table, played cards remain separated, rings are transparent/equal circles, and empty hands retain their footprint. A 25-point trick plus 100-point marriage displays as 125. [Reviewed preview](screenshots/11-transparent-marriage.png).

The new suite uses real creation/history/resume/reconnect endpoints before explicitly controlled UI fixtures. Its controlled clock verifies exactly one automatic command at two seconds, the unchanged deadline through sorting, multiple/stale-choice handling, inspection/lobby/receipt pauses, and right-click/long-press opt-out without an accidental release click. Fixtures wait for command acknowledgement before reload, preserving the pending-command safety gate.

On public HTTPS, the same resume/reconnect and controlled interaction checks pass with signed synthetic identities. Real three/four-seat solo deals additionally exercise automatic human plays through the actual server, and their audits are replayed by the deployed verifier. The four-seat table covers both a sitting-out human dealer and an active human in the next deal. Controlled UI fixtures and actual rules-engine deals are distinct. These browser checks do not replace a manual Telegram-client playthrough.


| Live table | Completed deals | Automatic human cards | Audit events | Audit moves |
| --- | --- | --- | --- | --- |
| `C832N339` - 3 seats | 1 | 3 | 88 | 30 |
| `VMXJ8QXN` - 4 seats | 2 | 3 in the active deal | 173 | 60 |

Public health/auth restrictions and all seven frontend assets match source byte-for-byte. Fresh Telegram authentication with empty browser storage still restores the original historical table for all three original synthetic members, with its exact audit unchanged after this upgrade. The original test cookies exceeded their fixed 24-hour TTL; membership recovery is independent of that expiry.

## Operations

Consistent SQLite backup, integrity check passed:
`/var/backups/pastimer/pastimer-20261115-pre-resume.sqlite` (7979008 bytes).

Previous images, native archives and backups remain available; preserve the PVC. Canonical manifests are [deploy/horse](../deploy/horse/), mirrored with this record to `/Users/michael/dev/k3s-horse/pastimer/`. Ignored acceptance logs/proofs are under `artifacts/deployments/20261115-resume/`. The temporary native BuildKit worker is stopped with no isolated tasks running. Colima was not started. Assets use no-store; reopen the Mini App to load this version.
