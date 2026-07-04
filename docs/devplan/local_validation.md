# Local Validation

Checkpoint 0 baseline commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p replay_runner
cargo run -p replay_runner -- --replay tests/golden_replays/vertical_slice_feel_fan.replay.json
cargo run -p board_validator
cargo run -p content_linter
cargo run -p seed_browser -- --act 1 --archetype fan --count 3
cargo run -p feverfall_game
```

Playable Bevy feel-test scene:

```bash
cargo run -p feverfall_game --features bevy_feel_test -- --feel-test
```

Feature-built feel-test binaries also launch the playable scene by default when no CLI args are passed. Use Left/Right or A/D to adjust aim and Space to fire a deterministic physics shot on the embedded `boards/feel_fan_01` authored board. Firing draws a cyan trajectory trail and yellow final ball marker. The scene uses Bevy 0.18 because Bevy 0.19 declares `rust-version = 1.95.0`, while the current validation environment has `rustc 1.94.0`.

To force the non-interactive smoke path in a feature-built binary for CI or debugging:

```bash
cargo run -p feverfall_game --features bevy_feel_test -- --smoke
```

Native Windows feel-test build:

- Use GitHub Actions workflow `Windows Feel-Test Build` (`.github/workflows/windows-feel-test.yml`) to build the playable Windows binary natively on `windows-latest`.
- Trigger it manually from GitHub Actions with `Run workflow`, or let it run on pushes touching Cargo, game, physics, board-generation/content-schema, or workflow files.
- The workflow runs `cargo build -p feverfall_game --features bevy_feel_test --release` and uploads `feverfall_game-windows-x86_64-native` plus `feverfall_game-windows-x86_64-native-sha256` artifacts.
- The uploaded `feverfall_game.exe` launches the playable scene directly when double-clicked with no CLI args. Run it from a terminal with `--smoke` only when the non-interactive smoke path is desired.
- This native Windows build exists because the earlier Linux Docker cross-compiled `.exe` was flagged by Windows Defender as `Trojan:Win32/Wacatac.B!ml`, likely due to ML/reputation heuristics. Building on Microsoft-hosted Windows should provide a cleaner provenance path for human feel testing.

Previous local cross-compiled Windows binary checksum:

```text
dac381bb4cbd8c764a779cf9a9bac80cb2f26f505ac4f26e8428701f1ef5b652  feverfall_game.exe
```

Checkpoint 1 tooling notes:

- `replay_runner` reads `tests/golden_replays/minimal_test.replay.json` by default, or `--replay FILE`. It uses shared `BoardDefinition` and `ShotInput` schemas, runs `physics_core::simulate_shot`, emits a deterministic replay hash, and compares it with `expected_hash`. Fixtures can temporarily set `pending_simulator: true` with no `expected_hash` only if simulator integration regresses or changes API.
- `tests/golden_replays/vertical_slice_feel_fan.replay.json` is the Checkpoint 2 vertical-slice smoke replay. It references the authored `game/assets/content/boards/feel_fan_01.json` board instead of duplicating board data and uses the same deterministic seed, launch speed, and two-step-left aim as the non-interactive feel-test smoke scene.
- CI runs the replay hash gate unconditionally. Stricter changed-file gating for physics/content/rules-only replay enforcement is a next step.
- `board_validator` reads JSON boards from `game/assets/content/boards`, falling back to `minimal_test_board` when no board files exist.
- `content_linter` walks `game/assets/content`, validates board JSON schemas and content ID conventions, and reports duplicate IDs.
- `seed_browser` accepts `--act`, `--archetype`, `--count`, and `--seed-start`/`--seed`.
- `telemetry::JsonlTelemetryLogger` records local JSONL playtest events from copied event data only. It is validated by a workspace test that logs a shot and re-runs the same shot with the same replay hash.
- QA playtest artifacts live in `docs/playtesting/feel_survey.md` and `docs/qa/`. Use the determinism checklist before accepting physics, replay, or telemetry changes.
