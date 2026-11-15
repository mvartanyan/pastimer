# Horse Durak readability correction

Date: 2026-11-15. Live: https://1000.vartanyan.com. Telegram: https://t.me/pastimerbot.
Previous release: [Durak variants and computers](horse-durak-2026-11-15.md).

## Behavior

The initial Durak layout was too small to use comfortably. Hand cards are now 84-104px wide normally and 76px on short screens. Two overlapping rows expose at least 44px of the upper row, while a final unpaired card is tappable across its whole visible face. Overflow scrolls horizontally without moving the action area. Primary controls use 15-16px text and 46-48px hit areas. Seat names use 12-14px text in a scrolling ribbon.

Played cards use the central space in a horizontally scrolling row. Selecting a defence target reveals it, and refreshes preserve inspection scrolling. Trump rank and suit are explicit text. Compact portrait screens put sorting in the action row. The waiting room has no fake hand placeholders and shows clear waiting/ready instructions and a full-width Start game button. All instructions are localized in English, Russian, German and Spanish. Rules and computer policy are unchanged.

## Release identity

- Source: `5b27aba9a63c341a98dca9a8f61fcf49ac8b287a`.
- Image: `docker.io/library/pastimer:commit-5b27aba9a63c341a98dca9a8f61fcf49ac8b287a-amd64`.
- Clean Git archive SHA-256: `d4ee221a52880d37ca9b736dec9439e2167f2960a4cf41760b0ea6502318d0b0`.
- Manifest digest: `sha256:db963be69966483102350b2f893c923174be0e73842752814b05fe5da1cf193e`.
- Runtime image/config ID: `sha256:870315f267b232acecc3854b29e4879bcec628f6d515c5934a96c9190ad27669`.
- Pod: `pastimer-b5b46cf6-kphf8`, Ready with zero restarts.

Built natively on Horse from a committed archive and imported into k3s. Only the Pastimer Deployment image changes. One replica, Recreate strategy, Secret, ingress, certificate and original PVC (`fe1cefbc-cfc7-4cdc-bef4-ae4840f2675d`) are preserved. No database migration is required.

## Verification

The browser suite passed its demo, Thousand, marriage, archive, computer and both complete Durak game checks. A subsequent fixture geometry check raced the page reload; it now waits for the intended fixture version and target. The complete Durak suite then passed, including both games and independent audits, rematches, 36-card hands, touch/mouse scrolling, request approvals, four languages, small-screen readability and bottom-of-face taps on odd hands of one, three and five cards. Geometry checks cover 320x480, 390x650, 390x844, 844x390 and a 1024x768 large hand. Formatting and strict Clippy passed during this correction. Rust gameplay did not change; the initial Durak release had all 58 Rust tests passing.

Public HTTPS health, production authentication/origin restrictions and all eight frontend assets match the final source byte-for-byte. Runtime image identity, one Ready replica with zero restarts, Recreate strategy, original Bound PVC and Ready TLS certificate are verified. Fresh Telegram authentication with empty browser storage recovers all three original historical memberships and the exact original Thousand audit.

Signed production browser sessions at 390x650 created, started and played real cards in both enlarged interfaces:

| Variant | Table | Seats | Computers | Hand card width | Seat name text |
| --- | --- | --- | --- | --- | --- |
| Podkidnoy | `X6KPU2UJ` | 2 | 1 | 89.69px | 14px |
| Perevodnoy | `YM8JYJ9Y` | 6 | 5 | 89.69px | 14px |

The Start button spans over 300px with a minimum 44px hit height. No horizontal page overflow or browser errors occurred, and the full face of each unpaired hand card is tappable. Final live screenshots were visually reviewed. Both original completed Durak audits (`LE7UVEP3`, `RJKE6DSF`) remain byte-equivalent as parsed JSON. These signed browser flows are distinct from a manual real Telegram-client playthrough.

## Operations

Consistent online SQLite backups, both 7979008 bytes with integrity check `ok`:

- `/var/backups/pastimer/pastimer-20261115-pre-durak-readable.sqlite`.
- `/var/backups/pastimer/pastimer-20261115-pre-durak-readable-final.sqlite`.

The first readability image used source `16a9bd1cd7ac8ff645b6dadd5f2d395157c33e1d`. Live checks exposed the unpaired-card hit-area issue, corrected in the final source above. Previous images, native archives and backups are retained. The isolated temporary builder is stopped with no running tasks; images and cache are retained. Colima was not used or started. Acceptance logs and screenshots are under ignored `artifacts/deployments/20261115-durak-readable/`.

Use the canonical [Horse overlay](../deploy/horse/). Assets use `Cache-Control: no-store`, so cache invalidation is unnecessary. Fully close and reopen the Telegram Mini App to load the correction. The preceding Durak image can read the same snapshots; preserve the PVC and post-release games during any rollback.
