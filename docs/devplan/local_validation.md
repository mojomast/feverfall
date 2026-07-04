# Local Validation

Full Checkpoint 4 release validation commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p feverfall_game -- --smoke-full
cargo run -p replay_runner
cargo run -p replay_runner -- --replay tests/golden_replays/vertical_slice_feel_fan.replay.json
cargo run -p replay_runner -- --replay tests/golden_replays/act1_twobboard_run.replay.json
cargo run -p replay_runner -- --replay tests/golden_replays/rpg_chapter1_smoke.replay.json
cargo run -p replay_runner -- --replay tests/golden_replays/rpg_ch1_smoke.replay.json
cargo run -p replay_runner -- --replay tests/golden_replays/roguelite_3act_smoke.replay.json
cargo run -p replay_runner -- --replay tests/golden_replays/roguelite_act1to3_smoke.replay.json
cargo run -p content_linter
cargo run -p board_validator
cargo run -p seed_browser -- --mode rpg --chapter 1 --archetype fan --count 3
cargo run -p feverfall_game -- --smoke
cargo run -p feverfall_game -- --smoke --mode rpg --chapter 1
cargo run -p feverfall_game -- --smoke --mode roguelite --acts 3
cargo check -p feverfall_game --features bevy_feel_test
cargo clippy -p feverfall_game --features bevy_feel_test --all-targets -- -D warnings
cargo run -p feverfall_game --features bevy_feel_test -- --smoke
```

Current Checkpoint 4 expected outputs:

- Toolchain: Rust 1.95.0 via `rust-toolchain.toml`.
- Optional Bevy feel-test dependency: Bevy 0.19.
- Default replay hash: `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`.
- Vertical-slice replay hash: `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`.
- Act 1 two-board replay hash: `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`.
- RPG Chapter 1 smoke replay hash: `8e566217ee6cddee3be784b3e359b3eda5708638ac8540bce759086e922a145f`.
- Roguelite 3-act smoke replay hash: `89c224a1ba8aae30965fa42f9547940036badc026b0a2f1bf50e6de15b86682b`.
- RPG Chapter 1 implementation smoke replay hash: `fc72b1144ad88e62bb27c3a1296cbb9b3fa51871a852b9b5ef561d7146033a58`.
- Roguelite Act 1-3 implementation smoke replay hash: `c5db0fb8d90e57c8be159bbb779c56ead19148f36de8bdc077711e59f9a4a36a`.
- C2 run summary hash: `18202124e6b686d8`.
- RPG Chapter 1 summary hash: `3364e243ba2065f4`.
- RPG campaign summary hash: `04029810211125c5`.
- Roguelite Act 1-3 summary hash: `e72374145338c3b3`.
- Roguelite Acts 1-4 full-run summary hash: `152fc850303d8356`.
- Bevy feel-test smoke hash: `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`.
- `cargo run -p content_linter` currently reports `content lint passed: 242 unique id(s)` with C4 content present.
- `cargo run -p board_validator` includes `PASS boards/act1_boss_01`.

Checkpoint 4 status note:

- `cargo run -p feverfall_game -- --smoke-full` is the release gate: exit code 0 means all built-in smoke segments ran, all golden replay hashes matched, and content/board validation passed.
- Windows artifact production is nonlocal and must be verified by GitHub Actions from a pushed ref containing the C4 workflow update.

Playable Bevy feel-test scene:

```bash
cargo run -p feverfall_game --features bevy_feel_test -- --feel-test
```

Feature-built feel-test binaries also launch the playable scene by default when no CLI args are passed. Use Left/Right or A/D to adjust aim and Space to fire a deterministic physics shot on the embedded `boards/feel_fan_01` authored board. Firing draws a cyan trajectory trail and yellow final ball marker. The workspace pins Rust 1.95.0 in `rust-toolchain.toml` and uses Bevy 0.19 for the optional Bevy feel-test build.

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
- `tests/golden_replays/act1_twobboard_run.replay.json` is the Act 1 two-board smoke fixture. It uses the replay runner's ordered `boards` fixture format to simulate one scripted shot on `feel_fan_01` and one scripted shot on `feel_wave_01`, then verifies hash `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`.
- `tests/golden_replays/rpg_chapter1_smoke.replay.json` is the defensive C3 RPG campaign fixture. It references a board by `board_path`, includes an `rpg_mode::CharacterState` snapshot, simulates one scripted shot, and verifies hash `8e566217ee6cddee3be784b3e359b3eda5708638ac8540bce759086e922a145f`.
- `tests/golden_replays/roguelite_3act_smoke.replay.json` is the defensive C3 roguelite 3-act smoke fixture. It uses the ordered `boards` replay format with three authored boards and verifies hash `89c224a1ba8aae30965fa42f9547940036badc026b0a2f1bf50e6de15b86682b`.
- `cargo run -p feverfall_game -- --smoke` is the integrated C2 smoke path for node progression, reward application, UI summaries, telemetry/run summary shape, and smoke run summary hash `0b36add9e9b3283c`.
- `cargo run -p feverfall_game -- --smoke --mode rpg --chapter 1` and `cargo run -p feverfall_game -- --smoke --mode roguelite --acts 3` are C3 defensive smoke commands. Current runtime code safely falls through to the generic smoke path until mode-specific feature code lands.
- `cargo run -p feverfall_game --features bevy_feel_test -- --smoke` preserves the non-interactive Bevy-feature smoke path and verifies feel-test smoke hash `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`.
- CI runs the replay hash gate unconditionally. Stricter changed-file gating for physics/content/rules-only replay enforcement is a next step.
- `board_validator` reads JSON boards from `game/assets/content/boards`, falling back to `minimal_test_board` when no board files exist.
- `content_linter` walks `game/assets/content` and top-level `content`, validates board JSON plus relic, ball, shop, RPG gear, RPG skill, and balance table content/schema ID conventions, and reports duplicate IDs. Current C3 content reports 58 unique IDs.
- `seed_browser` accepts `--mode`, `--act`, `--chapter`, `--archetype`, `--count`, and `--seed-start`/`--seed`. Use `cargo run -p seed_browser -- --mode rpg --chapter 1 --archetype fan --count 3` to generate and validate RPG Chapter 1 objective-tagged boards.
- `telemetry::JsonlTelemetryLogger` records local JSONL playtest events from copied event data only. It is validated by a workspace test that logs a shot and re-runs the same shot with the same replay hash.
- QA playtest artifacts live in `docs/playtesting/feel_survey.md` and `docs/qa/`. Use the determinism checklist before accepting physics, replay, or telemetry changes.
