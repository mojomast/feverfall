# Determinism Validation Checklist

Run this for Checkpoint 1 physics or telemetry changes.

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

## Failure Response

- Stop content/balance approval for affected boards.
- Save the replay fixture and JSONL log.
- File a bug with `docs/qa/bug_triage_template.md`.
- Mark whether the failure is deterministic wrong behavior or nondeterministic behavior.
