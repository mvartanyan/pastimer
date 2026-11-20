# Thousand mobile UI previews

These previews show the interface after the gameplay update: larger suit indices, auction speech bubbles, hand-selected transfers and seat-based last-trick inspection.

Captured as viewport images at a 390 x 650 CSS-pixel viewport with a 1x device scale. The helper drives actual game actions against an isolated Rust server using signed synthetic Telegram identities, then verifies the exported audit. These are browser previews rather than captures from a live Telegram client. Card order, names' seating positions and table code vary between runs.

| Situation | Screenshot |
| --- | --- |
| Bidding and sealed stock | [01-bidding.png](01-bidding.png) |
| Cards dragged to recipient hands, with tap-to-turn visibility | [02-distribution.png](02-distribution.png) |
| A completed trick before its three-second collection | [03-playing-a-trick.png](03-playing-a-trick.png) |
| Last captured trick opened from a player seat | [04-collected-tricks.png](04-collected-tricks.png) |
| Highlighted secret receipt with no instructional text | [05-incoming-card.png](05-incoming-card.png) |
| Top-view rings, marriage card and centred trump (UI fixture) | [06-hearts-trump.png](06-hearts-trump.png) |
| Cards by owner, leader and trick winner highlighted (UI fixture) | [07-trick-owners.png](07-trick-owners.png) |
| Empty hand retains the same field and ten pale slots (UI fixture, November 15) | [08-empty-hand.png](08-empty-hand.png) |
| Finished-match laurel and next actions (UI fixture, November 15) | [09-victory.png](09-victory.png) |
| Russian lobby and direct solo buttons (full-page UI fixture, November 15) | [10-lobby-ru.png](10-lobby-ru.png) |
| Transparent marriage rings and inset hand-info borders (UI fixture, November 15) | [11-transparent-marriage.png](11-transparent-marriage.png) |

The November 20 changes are shown in [the read-only current-player dealer hand](12-dealer-observer.png), [previous score / deal breakdown / new score](13-deal-progression.png), and [the winner's progression](14-deal-winner.png). These are controlled headless UI fixtures at 390x650. The seven-size checks include 320x480 and both landscape sizes.

Regenerate from the repository root after installing the optional browser dependencies described in the [handoff](../handoff.md):

```sh
cargo build --locked
node scripts/capture-screenshots.mjs
```

The trump-marker preview uses the controlled legal-view fixture in `tests/marriage-browser.mjs`. The incoming-card preview comes from the receipt step in `tests/thousand-browser.mjs`. The capture helper writes the other four previews to `artifacts/screenshots/`, which is ignored by Git. Review the resulting images and copy the selected PNGs into this directory to update the committed gallery. No real Telegram bot token is required.

The first seven images record the November 14 interface. The November 15 update adds fixed hand slots, captured-point scores, icon-only barrel markers and a persistent language switch. Run `node tests/marriage-browser.mjs` for the latest empty-hand and victory fixtures in `/tmp/pastimer-empty-hand.png` and `/tmp/pastimer-victory.png`.

## Durak usability correction - November 16

Headless browser fixtures of a six-seat defence with six attack cards. The turn prompt names the selected card and the legal response. The normal phone roster shows all seats, requests use dialogs, and landscape puts the hand beside the table. These are controlled UI projections, not a real Telegram-client capture. On very short screens, the table scrolls vertically to keep the selected defence target visible.

- [Phone, 390x650](durak-defence-mobile.png).
- [Landscape, 844x390](durak-defence-landscape.png).
