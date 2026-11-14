# Thousand mobile UI previews

These previews show the interface at application commit `ba4ac97`: an illustrated deck, a felt table with relative player seats, a fanned sortable hand, transfer previews and separate inspectable trick piles.

Captured as full-page images at a 430 × 932 CSS-pixel viewport with a 2× device scale. The helper drives actual game actions against an isolated Rust server using signed synthetic Telegram identities, then verifies the exported audit. These are browser previews rather than captures from a live Telegram client. Card order, names' seating positions and table code vary between runs.

| Situation | Screenshot |
| --- | --- |
| Bidding and sealed stock | [01-bidding.png](01-bidding.png) |
| Choosing cards to give, with independent visibility | [02-distribution.png](02-distribution.png) |
| A trick in progress and a selected legal card | [03-playing-a-trick.png](03-playing-a-trick.png) |
| Deal results and each player's last captured trick | [04-collected-tricks.png](04-collected-tricks.png) |

Regenerate from the repository root after installing the optional browser dependencies described in the [handoff](../handoff.md):

```sh
cargo build --locked
node scripts/capture-screenshots.mjs
```

The helper writes to `artifacts/screenshots/`, which is ignored by Git. Review the resulting images and copy the four selected PNGs into this directory to update the committed gallery. No real Telegram bot token is required.
