# Checkpoint Status

## Checkpoint 0: Contracts First

Status: complete for baseline shared contracts.

Completed baseline:
- Cargo workspace skeleton.
- Required crates and tool packages.
- Initial shared schema ownership.
- Serialization tests for board, physics event, replay metadata, run state, character state, and feedback event.
- Placeholder CI and local validation commands.

Validation completed:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p replay_runner`
- `cargo run -p board_validator`
- `cargo run -p content_linter`
- `cargo run -p seed_browser`
- `cargo run -p feverfall_game`

Next checkpoint:
- Checkpoint 1: Physics Feel Alpha.
