# Orchestrator State

## Current Checkpoint

Checkpoint 1: Physics Feel Alpha.

## Completed Since Last Session

- Established Cargo workspace skeleton.
- Added required shared crates, game skeleton, tool skeletons, docs directories, test directories, and CI workflow.
- Added initial shared contracts and serialization tests.
- Fixed the minimal test board so board validation includes an orange peg and passes baseline validation.
- [A] Physics Core & Feel implemented deterministic fixed-step simulator, CCD collision primitives, peg lit/clear timing, bucket catch events, SHA-256 replay hashing, first-bounce prediction, and physics tests.
- [B] Board Generation added 10 authored feel-test boards, seeded generator updates, authored-board loading, 512-angle validation, catch opportunity checks, and 15% dead-zone rejection.
- [G] Build / Tooling / CI upgraded replay runner, board validator, content linter, seed browser, CI gates, local validation docs, and golden replay fixture.
- [A/B] Corrective pass added first-bounce prediction validation and corrected the dead-zone threshold to 15%.
- [E] Feedback / VFX / Audio added mocked Checkpoint 1 playback state covering peg hits, bucket catch, near miss, combo, final orange, Extreme Fever, loss, and accessibility flags.
- [F] UI / HUD added pure-Rust aim HUD, score HUD, progression HUD, and debug overlay models using shared physics/run/RPG contracts.
- [I] QA / Telemetry added the `telemetry` crate, JSONL logger, replay tagging, feel survey, bug triage template, determinism checklist, and replay-hash safety test.
- [A] Bucket-feel diagnostics sampled authored boards and found 8 boards with at least 2 catchable trajectories.
- [G] Corrective pass wired mock plugin registration summaries into the game binary to satisfy strict clippy.
- [A] Tuning iteration 1 improved `boards/feel_fan_cross_01` and `boards/feel_wave_gate_01` bucket opportunities. All 10 authored boards now have at least 2 catchable sampled trajectories.
- [G] Option C implemented a feature-gated Bevy feel-test build path and preserved the default non-interactive game smoke path.
- [F] Option C implemented the feel-test scene model with aim adjustment, deterministic shot simulation, first-bounce data, replay hash, HUD/debug summaries, and tests.
- [E] Option C wired feel-test shot results to existing feedback VFX/audio cue summaries with accessibility reductions.
- Built Windows x86_64 feel-test binary for human testing from the current Checkpoint 1 source.

## Active Workstreams

- Physics Core & Feel: automated simulator, first-bounce, no-tunneling, and bucket diagnostics passing.
- Feedback / VFX / Audio: mocked playback scene passing; Bevy runtime wiring deferred to vertical slice.
- UI / HUD: pure-Rust HUD/debug models passing; Bevy runtime wiring deferred to vertical slice.
- QA / Telemetry: telemetry/logger/docs passing; optional playtest CLI wiring remains future work.
- Checkpoint gate: playable feel-test scene is available; human feel validation is required before Checkpoint 2 unless two more tuning iterations are requested with specific feedback.

## Subagents Dispatched

- Contracts / Architecture review subagent inspected artefacts and recommended explicit ownership for shared contracts.
- [A] Physics Core & Feel Agent: Checkpoint 1 simulator and corrective first-bounce prediction pass.
- [B] Board Generation Agent: authored boards/validator and corrective 15% dead-zone pass.
- [G] Build / Tooling / CI Agent: functional replay/content/board/seed tooling and CI update.
- [E] Feedback / VFX / Audio Agent: mocked playback scene.
- [F] UI / HUD Agent: HUD/debug overlay models.
- [I] QA / Telemetry Agent: telemetry crate and QA docs.
- [A] Physics Core & Feel Agent: bucket catch diagnostics.
- [G] Build / Tooling / CI Agent: strict clippy plugin-registration fix.
- [A] Physics Core & Feel Agent: tuning iteration 1 for bucket catch robustness.
- [G] Build / Tooling / CI Agent: Option C Bevy feel-test build path.
- [F] UI / HUD Agent: Option C feel-test scene shell and HUD/debug models.
- [E] Feedback / VFX / Audio Agent: Option C shot feedback wiring.

## Files Changed

- `Cargo.toml`
- `.gitignore`
- `.github/workflows/ci.yml`
- `crates/content_schema/*`
- `crates/physics_core/*`
- `crates/game_rules/*`
- `crates/board_gen/*`
- `crates/run_mode/*`
- `crates/rpg_mode/*`
- `crates/feedback_events/*`
- `game/*`
- `tools/replay_runner/*`
- `tools/board_validator/*`
- `tools/content_linter/*`
- `tools/seed_browser/*`
- `docs/technical/shared_contracts.md`
- `docs/devplan/local_validation.md`
- `docs/devplan/checkpoint_status.md`
- `Cargo.lock`
- `game/assets/content/boards/*.json`
- `game/src/feel_test.rs`
- `game/src/plugins/feel_test.rs`
- `game/src/plugins/feedback.rs`
- `tests/golden_replays/minimal_test.replay.json`
- `crates/telemetry/*`
- `docs/playtesting/feel_survey.md`
- `docs/qa/bug_triage_template.md`
- `docs/qa/determinism_checklist.md`
- `docs/qa/replay_tagging.md`

## Validation Commands Run

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p replay_runner`
- `cargo run -p board_validator`
- `cargo run -p content_linter`
- `cargo run -p seed_browser`
- `cargo run -p seed_browser -- --act 1 --archetype fan --count 3`
- `cargo run -p feverfall_game`
- `cargo check -p feverfall_game --features bevy_feel_test`
- `cargo clippy -p feverfall_game --features bevy_feel_test --all-targets -- -D warnings`
- Docker Windows cross-build: `cargo build -p feverfall_game --features bevy_feel_test --release --target x86_64-pc-windows-gnu`

## Passing Validation

- Formatting, strict clippy, workspace tests, replay runner, board validator, content linter, seed browser smoke, and game smoke all pass after A/B/G integration.
- `cargo test --workspace` includes 13 `physics_core` tests, including first-bounce prediction matching simulation, no tunneling, no NaN, bucket catch, peg clear timing, and 10,000 random-ish stress shots.
- `cargo run -p board_validator` passes all 10 authored boards.
- `cargo run -p replay_runner` matches the golden replay hash.
- `cargo run -p feverfall_game` prints plugin registration summary: `ui(first_bounce=true, balls=10, equipped_skills=1, power=75%), audio(cues=13, high_freq=5), vfx(events=10, cues=22, shake=3), debug(collisions=486, first_bounce=true, reused_aim=true)`.
- Automated bucket diagnostics after tuning iteration 1: all 10 authored boards meet the 2+ catchable trajectory threshold.
- Default game smoke prints a deterministic feel-test summary and exits.
- Feature-gated playable Bevy feel-test compiles and passes clippy with `bevy_feel_test` enabled.
- Windows binary built successfully at `target/x86_64-pc-windows-gnu/release/feverfall_game.exe`.
- Windows binary SHA-256: `dac381bb4cbd8c764a779cf9a9bac80cb2f26f505ac4f26e8428701f1ef5b652`.

## Failing Validation

- None.

## Environment Notes

- Bevy 0.19 is blocked in the current validation environment because it requires `rustc 1.95.0`; the environment has `rustc 1.94.0`.
- The feature-gated playable scene uses Bevy 0.18 until the toolchain can be raised.
- Playable feel-test command: `cargo run -p feverfall_game --features bevy_feel_test -- --feel-test`.

## Blockers

- None.

## Decisions Needed From Human

- Run the playable feel-test scene and either approve Physics Feel Alpha for Checkpoint 2 or request tuning iteration 2 with a specific target: too floaty, too chaotic, catch too forgiving, catch too strict, or first bounce unreadable.

## Last Replay Hash

- `598ff57ca69b031c4e487fdd54d07d7c7f2667d20f66ad1be85351ae58ae0630`

## Next Integration Target

- Human judgment from playable feel-test scene.

## Next Parallel Dispatch

- Pending human decision after running `cargo run -p feverfall_game --features bevy_feel_test -- --feel-test`: approve Checkpoint 2 dispatch or request tuning iteration 2 with a specific target.
