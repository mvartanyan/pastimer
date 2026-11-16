# Horse Durak clarity and Thousand receipt correction

Date: 2026-11-16. Live: https://1000.vartanyan.com. Telegram: https://t.me/pastimerbot.
Previous release: [Durak readability](horse-durak-readable-2026-11-15.md).

## Behavior

Durak has one two-line prompt describing the viewer's next action, including the selected card to beat, taking, finishing a throw-in opportunity, transfers and waiting for permission. Opponents fit a three-column roster instead of a horizontally hidden ribbon. Attack pairs have rank/suit captions. Hands retain two horizontally scrolling rows: separate full rows on tall screens, 76px exposed upper-row areas on medium screens, and 44px on compact screens. Cards remain at least 84px wide normally and 76px on compact screens. Landscape places the hand beside the table. A short screen can scroll the table vertically; selecting a defence target reveals it. The action dock stays stationary.

The action row has at most three buttons. "I want to throw in" opens rank choices; "Requests" opens approvals that the attacker can approve or ignore. Each new rank still requires its own request and approval. Sort and optional multi-card attacks are under Options. All new text is localized in English, Russian, German and Spanish. [Phone and landscape previews](screenshots/README.md) are controlled UI fixtures. Rules and computer policy are unchanged.

The Thousand screenshot's glowing nine was a pending card receipt, not the only legal move. Receipt prompts now end when trick play starts. All held cards enter the normal hand, with no receipt overlay dimming or disabling legal cards or pausing forced play. Open/secret inspection before play is retained. No diamonds and no trump against a diamond lead allows all eight cards; no automatic choice is made among them. Exactly one legal card still plays after two seconds through the durable command path, with stale-choice, visibility, inspection and retry guards intact.

## Release identity

- Source: `7d38925c3526041e64ae8e712ef14dba4fc30274`.
- Image: `docker.io/library/pastimer:commit-7d38925c3526041e64ae8e712ef14dba4fc30274-amd64`.
- Clean archive SHA-256: `6ff29fe929a27fd42b592f63b7ffc51b11500c701c28bbf1b02104abbf12f007`.

- Manifest digest: `sha256:81cc007683cb0bed1b82dd523ce82dffb1bc9fede4dd1ce34db10ccc5e1e8261`.
- Runtime image/config ID: `sha256:f96cb23099369dbd41a8ee1fda12c707783ec04ca1f5754f5acbe78e17f0f83b`.
- Pod: `pastimer-74f869675c-772dn`, Ready with zero restarts.

Built natively on Horse from the clean committed archive, imported into k3s and rolled out by changing only the Pastimer Deployment image. No Rust rules, database schema, secrets or volume configuration change.

## Verification

The screenshot fixture enables all eight cards against the diamond ace. The forced-play fixture has a club lead and exactly one club, with an unacknowledged delivery; that club is sent at exactly two seconds. Existing inspection/lobby pauses, deadline preservation through sorting, no double submission and marriage gestures pass. Durak fixtures pass small phone/landscape geometry, target visibility, 36-card scrolling hands, swipe safety, odd-hand full-face hit areas, rank requests, approval, multi-card opening, transfer mode and four languages.

The complete `npm run test:browser` suite passes: demo, human/computer Thousand, marriages, recovery/archive, screenshot and forced-card regressions, both complete Durak variants with independently verified audits, rematches and the revised controls. JavaScript syntax and whitespace checks pass. Rust logic is unchanged; the preceding rules release passed all 58 Rust tests.

Public HTTPS health, authentication/origin restrictions, catalogue and all eight frontend assets match the committed source byte-for-byte. Runtime image identity, one replica with Recreate strategy, the original Bound PVC and Ready TLS certificate pass. Fresh authentication with empty browser storage restores all three original Thousand memberships and the exact original audit. Both original completed Durak audits remain unchanged.

Signed production browser sessions at 390x650 created, started and played through both revised Durak interfaces:

| Variant | Table | Seats | Computers |
| --- | --- | --- | --- |
| Podkidnoy | `2EJUQVJ2` | 2 | 1 |
| Perevodnoy | `RU8RTFGN` | 6 | 5 |

The live interfaces have 84px hand cards, visible seat columns, a two-line next-action prompt, no horizontal page overflow and full-face tap areas for unpaired cards. Final live screenshots were visually reviewed. The live Thousand table `H3S2KD44` completed a deal against two computers, with three actually automatic human card plays. Receipt acknowledgements were deliberately left untouched. The verifier inside the deployed pod accepts its proof: one deal, 88 events and 30 moves. Reload preserves the settled result. Browser checks are explicitly headless. Manual play in a real Telegram client remains distinct from signed synthetic browser verification.

## Operations

Use the canonical [Horse overlay](../deploy/horse/). Keep one SQLite writer, Recreate strategy and the original PVC (`fe1cefbc-cfc7-4cdc-bef4-ae4840f2675d`). The release requires no migration. Consistent online SQLite backup: `/var/backups/pastimer/pastimer-20261116-pre-clarity.sqlite` (8294400 bytes, integrity check `ok`). Previous images, native archives and backups are retained. Assets use `Cache-Control: no-store`; fully close and reopen the Mini App to load them. The isolated temporary builder is stopped with no running tasks; images and cache are retained. Colima was not used or started. Local evidence is under ignored `artifacts/deployments/20261116-durak-clarity/`.
