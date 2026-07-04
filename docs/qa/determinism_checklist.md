# Determinism Validation Checklist

Run this for physics or telemetry changes. For Checkpoint 2 vertical-slice QA, also capture one short end-to-end run.

## Required Commands

- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo run -p replay_runner`

## Manual Checks

- Same replay fixture produces the same hash across repeated local runs.
- Telemetry logging is write-only and does not take `&mut BoardDefinition`, `&mut ShotResult`, or mutable physics state.
- Replay hash is recorded in telemetry as data, not recomputed from telemetry output.
- JSONL logs contain no player name, email, IP address, or machine-specific path.
- Bug reports include board ID, seed, shot index, replay hash, and build ID.
- Any changed golden replay hash has an accepted physics/rules reason documented in review notes.
- Short vertical-slice run logs shot result via `ShotResolved`: board ID, shot index, tick count, peg count, bucket catch flag, exit flag, and replay hash.
- Score/progression outcome is logged from `game_rules::GameEvent` using `ShotScoreResolved`, `BoardWon`, or `BoardLost` telemetry.
- Failure/catch notes are attached to the replay hash with `ReplayTagged` labels, not stored in private QA-only schemas.

## Failure Response

- Stop content/balance approval for affected boards.
- Save the replay fixture and JSONL log.
- File a bug with `docs/qa/bug_triage_template.md`.
- Mark whether the failure is deterministic wrong behavior or nondeterministic behavior.
- Tag the replay with `VerticalSliceFailure`, `BucketCatchMissed`, `BucketCatchSatisfying`, or `PhysicsFeltUnfair` as applicable.
