# Horse deal scores and dealer observation

Date: 2026-11-20. Live: https://1000.vartanyan.com. Telegram: https://t.me/pastimerbot.
Previous release: [Durak clarity and Thousand receipts](horse-clarity-2026-11-16.md).

## Behavior

Thousand seat counters show captured trick-card points only, including while the settled deal remains on the table. Deal results show the previous cumulative score, the deal's trick points plus individual marriages and any dealer bonus, the actual award, and the new cumulative score. A contract, barrel, limit, cancellation or dismissal adjustment is explicit when raw points differ from the award. Existing scoring rules are unchanged. Results names still open each player's last won trick. Compact layouts preserve the final-score laurel without clipping.

In four-player mode, the sitting-out human dealer observes only the current actor's hand, in the dealer's existing hand area. Ten stable slots, legal-card contrast and eligible-marriage highlights reproduce the active hand's dynamics. The cards are read-only and have no play handlers. Computer hands are automatically permitted; human hands remain face down unless that player explicitly trusts the named dealer before shuffling. Consent covers only the upcoming deal and clears on settlement or cancellation, including a same-dealer redeal. Other players and computer policies never receive these observer hands. All new controls and result text support English, Russian, German and Spanish.

See the synchronized [English](thousand-rules.md) and [Russian](thousand-rules.ru.md) rules and [visual previews](screenshots/README.md). Screenshot previews use controlled browser projections, not live Telegram captures.

## Release identity

- Source: `29337b633c96b0b2bd159e52ea86f2851f5b8a6b`.
- Image: `docker.io/library/pastimer:commit-29337b633c96b0b2bd159e52ea86f2851f5b8a6b-amd64`.
- Clean source archive SHA-256: `f6d13db99defcf66be41fdf272c5b8c1276aa3cbfcf620abedd2fbcb969ba428`.
- Manifest digest: `sha256:7bea050221cfa9a3c26bb3a5ff103cb7d956cdb241ecc42b0f5c57695fa5e5f2`.
- Runtime image/config ID: `sha256:8f03b9aa2260321980963b93ffb556e12e187391ced147ca99fa47977ef1b3e7`.
- Pod: `pastimer-85f75488bd-z9z8r`, Ready, zero restarts.

Built natively on Horse from the clean committed source archive, imported into k3s, and rolled out by changing only the Pastimer Deployment image. No database schema migration, secret, ingress, service, certificate or volume change.

## Verification

Formatting, strict Clippy and all 61 Rust tests pass. Coverage includes scoped consent, persistence, current-actor-only projection, computer-policy isolation, cancellation reset, truthful score decomposition, audit replay and rejection of rehashed invalid consent events. All browser-suite components pass on the final source: demo, three/four-human Thousand, marriages, forced-card timing, computer games, dealer observation, and both complete Durak variants with rematches and audits. The dealer/results suite covers seven viewport sizes down to 320x480 and four languages. Browser processes explicitly use `headless: true`.

A real signed-identity production four-human game, table `3E3EMC5R`, exercised pre-deal consent, hidden untrusted cards, the current actor's legal-card highlights, stable hand geometry, rejected spectator interaction, and reset at settlement. The deal included an 80-point diamond marriage: its player captured 45 trick points and earned 125; the unsuccessful contractor captured 60 and lost 100. The deployed verifier accepts the exported proof: one deal, one deck, 91 events and 30 moves. Exact `45 / 95+100+80=+275 / 320` and contract-adjustment layouts were separately verified using controlled browser projections. These fixtures are not claims about that live deal.

Public HTTPS health, production authentication/origin restrictions, catalogue and all eight frontend asset hashes match source. Empty-storage reauthentication restores all three original memberships and the exact historical audit. The original PVC remains Bound, TLS remains Ready, and the Deployment has one ready replica. Signed synthetic browser checks remain distinct from a manual Telegram-client acceptance playthrough.

## Operations

Use the canonical [Horse overlay](../deploy/horse/). Preserve the original `pastimer-data` PVC (`fe1cefbc-cfc7-4cdc-bef4-ae4840f2675d`), one SQLite writer and Recreate strategy. Consistent pre-release backup: `/var/backups/pastimer/pastimer-20261120-pre-deal-view.sqlite` (12173312 bytes; integrity check `ok`). Previous images, archives and backups are retained. The temporary native builder is stopped, with no running build tasks. Colima was not started. Local evidence is under ignored `artifacts/deployments/20261120-deal-view/`.

Fully close and reopen the Mini App to load the embedded assets. Table recovery is preserved. The isolated `codex/version-notice` work remains unmerged and is not part of this release.
