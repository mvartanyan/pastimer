# Horse Durak deployment

Date: 2026-11-15. Live: https://1000.vartanyan.com. Telegram: https://t.me/pastimerbot.
Previous release: [resume and forced play](horse-resume-2026-11-15.md).

## Behavior

Both Podkidnoy and Perevodnoy Durak are available in the game selector, using the agreed [English](durak-rules.md) and [Russian](durak-rules.ru.md) rules. Choose two to six seats and zero to five computer opponents, with at least one human. For solo play, set the computer count to one fewer than the table size. Computers use only their own typed player view and public table information; their actions wait one second.

Helpers request permission using "I want to throw in" / "Хочу подкинуть", separately for each player and rank. The current attacker sees requests and may approve or ignore them. New ranks need fresh requests and approvals; clockwise order and attacker priority still apply. Transfers change the current attacker and clear earlier requests and permissions. The two-row hand scrolls horizontally, supports touch and mouse dragging, and preserves its position through updates without submitting cards on a swipe. It supports the whole 36-card deck. Interface additions are localized in English, Russian, German and Spanish.

The host can rematch after settlement with the same seats. Loss counts accumulate. The original attacker retains drawing priority even after transfers; the final defender draws last. Settled-game audits reveal decks, movements and completed attacks for independent replay, while an unfinished rematch stays sealed. Archive inspection remains read-only and member-authorized.

## Release identity

- Source: `06354f6c20a7cf2f922606c24369dec47b87efe8`.
- Image: `docker.io/library/pastimer:commit-06354f6c20a7cf2f922606c24369dec47b87efe8-amd64`.
- Clean Git archive SHA-256: `9df0fe21546c8bbba01760d980405fe1f419de223168b1c54c611d065fb6954b`.
- Manifest digest: `sha256:fc0e03fa1f5d1edad1adf28e83655517affe402863f8126ba8d40a9d529cddd1`.
- Runtime image/config ID: `sha256:dadcf21efc0c8a7c3aeef263517664283c8173c66c8584b17df0a17efae56020`.
- Pod: `pastimer-6f7777df7-kws4d`, Ready with zero restarts.

Built natively on Horse from the committed archive using the existing isolated `pastimer-build` worker. The image was imported into k3s and only the Pastimer Deployment image was changed. One replica, Recreate strategy, Secret, ingress, certificate and original PVC (`fe1cefbc-cfc7-4cdc-bef4-ae4840f2675d`) remain intact. No database schema migration was required.

## Verification

All 58 Rust tests, formatting, strict Clippy and the complete browser suite pass. Tests cover both variants at two to six seats, requests and rank-specific approval, turn order, transfers, pickups, draws, rematches, conservation/privacy, concurrent durable retries, SQLite reopening and full audit replay, including rehashed forged records. Browser checks additionally cover the 36-card two-row hand at four portrait/landscape sizes, native touch/mouse scrolling and disabled hands, four languages and all existing demo/Thousand behavior.

Public HTTPS health, production authentication/origin restrictions, both new catalogue entries and all eight frontend assets match the release source byte-for-byte. No cache invalidation was needed: assets use `Cache-Control: no-store`. Close and reopen the Mini App to load this version.

Real production browser games used signed synthetic Telegram identities and the normal one-second computer scheduling. Each included an actual card play through the deployed UI and fresh-session recovery with empty browser storage. Both completed, and the verifier inside the deployed pod accepted their audits.

| Variant | Table | Seats | Computers | Events | Moves | Approved requests | Transfers |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Podkidnoy | `LE7UVEP3` | 2 | 1 | 169 | 73 | 0 | 0 |
| Perevodnoy | `RJKE6DSF` | 6 | 5 | 426 | 196 | 13 | 5 |

Fresh Telegram authentication also recovered all three original historical synthetic memberships and the exact original Thousand audit unchanged, both before and after deployment. These signed browser checks are distinct from a manual playthrough in a real Telegram client.

## Operations

Consistent pre-rollout SQLite online backup, integrity check `ok`:
`/var/backups/pastimer/pastimer-20261115-pre-durak.sqlite` (7979008 bytes).

The existing PVC and database were preserved. Previous images, native build archives and backups are retained. The temporary native builder is stopped, with no running tasks in its isolated namespace. Colima was not used or started. Local acceptance logs, screenshots and proofs are under ignored `artifacts/deployments/20261115-durak/`.

Use the canonical [Horse overlay](../deploy/horse/). The pre-Durak image cannot decode new Durak room snapshots; prefer a compatible forward fix for those rooms rather than blindly reverting to it. Do not restore a database backup over post-release games without a separate data-loss decision. No automatic/off-host backup schedule was added.
