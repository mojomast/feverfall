# Contributing to FeverFall

FeverFall is currently a deterministic Rust prototype moving from Checkpoint 4 feature-complete status into a Checkpoint 5 alpha-readiness pass. Contributions should preserve deterministic simulation, replay stability, and clear ownership boundaries.

## Quick Start

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p feverfall_game -- --smoke-full
```

Use `cargo run -p feverfall_game --features bevy_feel_test -- --feel-test` for the optional desktop feel-test path.

## Contribution Expectations

- Keep gameplay simulation deterministic; replay hash changes must be intentional and documented.
- Run the validation commands relevant to the files you changed before requesting review.
- Keep content IDs stable and unique; validate authored content with the workspace tools described in the README.
- Document player-facing or release-facing changes in `CHANGELOG.md` under `Unreleased`.
- Preserve file ownership boundaries in active checkpoint plans under `docs/agent/`.

## Documentation and Releases

- Public setup and status information belongs in `README.md`.
- Release notes belong in `CHANGELOG.md`.
- Agent prompts, checkpoint plans, and subagent scaffolding belong in `docs/agent/`.
- Release candidates should pass the full workspace gate and the `--smoke-full` game gate before tagging.

## License Status

The workspace package metadata currently declares `UNLICENSED`, and no root `LICENSE` file is present. Do not import third-party code or assets unless their license is explicitly compatible and their provenance can be recorded by the owning task.
