# Thousand mobile UI previews

These previews show the interface after the gameplay update: larger suit indices, auction speech bubbles, hand-selected transfers and seat-based last-trick inspection.

Captured as viewport images at a 390 x 650 CSS-pixel viewport with a 1x device scale. The helper drives actual game actions against an isolated Rust server using signed synthetic Telegram identities, then verifies the exported audit. These are browser previews rather than captures from a live Telegram client. Card order, names' seating positions and table code vary between runs.

| Situation | Screenshot |
| --- | --- |
| Bidding and sealed stock | [01-bidding.png](01-bidding.png) |
| Choosing cards to give, with independent visibility | [02-distribution.png](02-distribution.png) |
| A completed trick before its three-second collection | [03-playing-a-trick.png](03-playing-a-trick.png) |
| Last captured trick opened from a player seat | [04-collected-tricks.png](04-collected-tricks.png) |

Regenerate from the repository root after installing the optional browser dependencies described in the [handoff](../handoff.md):

```sh
cargo build --locked
node scripts/capture-screenshots.mjs
```

The helper writes to `artifacts/screenshots/`, which is ignored by Git. Review the resulting images and copy the four selected PNGs into this directory to update the committed gallery. No real Telegram bot token is required.
