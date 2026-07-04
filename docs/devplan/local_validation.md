# Local Validation

Checkpoint 0 baseline commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p replay_runner
cargo run -p board_validator
cargo run -p content_linter
cargo run -p seed_browser -- --act 1 --archetype fan --count 3
cargo run -p feverfall_game
```

Playable Bevy feel-test scene:

```bash
cargo run -p feverfall_game --features bevy_feel_test -- --feel-test
```

Use Left/Right or A/D to adjust aim and Space to fire a deterministic physics shot on the embedded `boards/feel_fan_01` authored board. The scene uses Bevy 0.18 because Bevy 0.19 declares `rust-version = 1.95.0`, while the current validation environment has `rustc 1.94.0`.

Windows test build command used by the orchestrator:

```bash
docker run --rm -v "$PWD":/project -w /project ghcr.io/cross-rs/x86_64-pc-windows-gnu:main bash -lc 'apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.94.0 --target x86_64-pc-windows-gnu && . "$HOME/.cargo/env" && cargo build -p feverfall_game --features bevy_feel_test --release --target x86_64-pc-windows-gnu'
```

Latest local Windows binary checksum:

```text
dac381bb4cbd8c764a779cf9a9bac80cb2f26f505ac4f26e8428701f1ef5b652  feverfall_game.exe
```

Checkpoint 1 tooling notes:

- `replay_runner` reads `tests/golden_replays/minimal_test.replay.json` by default, or `--replay FILE`. It uses shared `BoardDefinition` and `ShotInput` schemas, runs `physics_core::simulate_shot`, emits a deterministic replay hash, and compares it with `expected_hash`. Fixtures can temporarily set `pending_simulator: true` with no `expected_hash` only if simulator integration regresses or changes API.
- CI runs the replay hash gate unconditionally. Stricter changed-file gating for physics/content/rules-only replay enforcement is a next step.
- `board_validator` reads JSON boards from `game/assets/content/boards`, falling back to `minimal_test_board` when no board files exist.
- `content_linter` walks `game/assets/content`, validates board JSON schemas and content ID conventions, and reports duplicate IDs.
- `seed_browser` accepts `--act`, `--archetype`, `--count`, and `--seed-start`/`--seed`.
- `telemetry::JsonlTelemetryLogger` records local JSONL playtest events from copied event data only. It is validated by a workspace test that logs a shot and re-runs the same shot with the same replay hash.
- QA playtest artifacts live in `docs/playtesting/feel_survey.md` and `docs/qa/`. Use the determinism checklist before accepting physics, replay, or telemetry changes.
