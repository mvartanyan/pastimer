# Horse follow-up: mobile layout and computer opponents

Date: 2026-11-14. Origin: https://1000.vartanyan.com.
Telegram bot: https://t.me/pastimerbot.

This is a historical release record. See the [current gameplay release](horse-gameplay-2026-11-14.md). The [initial deployment record](horse-2026-11-14.md)
retains the infrastructure discovery, original release and refresh-race fix.

## Release identity

- Application source: `fe12ef5981f88f2cb99136442fe2db0d2a882d0f`.
- Layout correction: `4a9119c`; computer implementation: `fe12ef5`.
- Image: `docker.io/library/pastimer:commit-fe12ef5981f88f2cb99136442fe2db0d2a882d0f-amd64`.
- Image manifest: `sha256:6c7f197a66f61470cd765ce9d1947fec7798867914e45a26881076a4424c5f9d`.
- Runtime image/config ID: `sha256:5df88912a5b95046d075998f2f85a8e912b236637a6bc68eec5b673e7f43902f`.
- Source archive SHA-256: `4a45b70c8fa99cef705b5a7ee0376ba525e37c931f43b41cf09d5fff88c6df38`.
- Pod: `pastimer-844c4f7458-j74cr`, Ready, zero restarts.

The image was rebuilt natively for linux/amd64 on Horse with the existing
temporary BuildKit worker and imported into k3s. Only Pastimer's Deployment
image changed. The one-replica Recreate strategy, Secret, ingress, TLS and
existing 5Gi `pastimer-data` PVC were retained. No schema migration was needed.

## User-facing changes

The lobby now stacks its cards below 650px instead of squeezing both into a
phone-sized row. Native selects and number inputs use the same 46px height;
labels, fields and buttons align, and form spacing is consistent. The layout
was visually inspected at 320, 390, 520, 768 and 1024px without horizontal
overflow, including the computer-seat control.

Create a Thousand table with three seats and choose:

- `2 computers - solo` for one human.
- `1 computer - 2 people` for two humans.
- `No computers` for the original friends-only game.

Four-seat tables also support computers, up to three computer seats. Computers
occupy normal seats and appear as `Computer 1`, `Computer 2`, etc. When a computer
is the dealer, a human presses `Ask computer to deal`. Completed results and
trick piles remain available until the next deal is requested.

The strategy is intentionally basic: pass in the auction, choose private
transfers from the computer's own hand, retain the current order, and choose
legal plays, preferring an allowed marriage announcement. The policy receives
only its own typed player view. Every action passes through the existing rules
validator and sealed audit-proof code. No external AI service is used.

Computer turns run in bounded bursts inside the normal SQLite transaction.
Human request IDs retain their durable retry behavior. The persisted roster
defaults to empty for old snapshots. Room reads materialize due trick closures
and resume ready computers after restart, just as they resolve existing trick
deadlines; with no active client polling, progression waits for the next read.

## Verification

All checks passed on the committed source:

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
npm run test:browser
```

The Rust suite has 33 tests. New tests complete four audited deals for each
combination of three/four seats and one/two humans. They cover legal computer
distribution/play/shuffling, private unfinished-deal proofs, invalid capacities,
old snapshot decoding, restart recovery, concurrent trick closure, durable
retries and rejection of re-hashed forged computer-dealer requests.

The complete browser suite includes the original demo, human-only three- and
four-player Thousand, and new solo/two-human computer flows. Both new flows
exercise the actual lobby, automatic turns, a complete deal, audit verification
and reload. The preceding refresh/action regression still passes.

Public HTTPS checks passed for health, game catalogue, production login
restrictions and all five embedded assets. Every asset matches this source
checkout byte-for-byte. The running image/config ID matches the release above.

The original three signed sessions, completed room and exact exported audit
remained valid after this upgrade. Existing human-only games retain their
original rules and data. Before this follow-up the owner also confirmed real
Telegram launch, signed-in identity and table creation with the configured menu
button. Updated computer-game browser checks use signed synthetic identities;
they do not claim a separate manual Telegram-client playthrough.

Both live HTTPS flows completed and reloaded successfully:

- One human and two computers: one deal, 88 events, 30 moves, `valid: true`.
- Two humans and one computer: one deal, 87 events, 30 moves, `valid: true`.

These proofs were checked by the **deployed** `verify-audit` binary as well as the
local verifier. The extra solo event records a human request for a computer
dealer; the two-human flow happened to have a human dealer. Logs, screenshots,
proofs and result summaries are under the ignored local
`artifacts/deployments/20261114-computers/` directory.

## Backup and operation

A consistent online SQLite backup was taken before the upgrade:
`/var/backups/pastimer/pastimer-20261114-pre-computers.sqlite` (225280 bytes).
Its `PRAGMA integrity_check` result is `ok`. The existing PVC and all previous
backups were preserved.

The canonical overlay is `deploy/horse/`. The rendered manifest and operating
notes are mirrored in `/Users/michael/dev/k3s-horse/pastimer/`. Continue to import
each tested immutable image before applying that overlay. Never delete the PVC
as an update or rollback shortcut.

Previous images and archives are retained for rollback. A pre-computer image
can read the existing data but cannot drive computer turns in new computer
rooms; those rooms will wait until a version with computer support is restored.
No rollback or database restore was performed.

Responses use `Cache-Control: no-store`; no cache invalidation is required.
Colima was not started for this follow-up. The temporary native builder is
stopped, and its isolated containerd namespace has no running build tasks. Scheduled/off-host database backups remain
outside this initial setup.
