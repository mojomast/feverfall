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

## Checkpoint 1: Physics Feel Alpha

Status: complete. Human approved Physics Feel Alpha after the tuned native Windows build.

Completed:
- [A] Physics Core & Feel Agent implemented deterministic fixed-step shot simulation in `crates/physics_core`.
- [A] Added CCD for circle, capsule, segment, rect-edge, and bucket-rim collisions.
- [A] Added peg lit/clear timing, bucket catch detection, shot-ended summaries, SHA-256 replay hashes, 10,000-shot stress coverage, and first-bounce prediction API.
- [B] Board Generation Agent added 10 authored feel-test boards under `game/assets/content/boards`.
- [B] Added authored board loading, seeded generation, 512-angle geometric sampling, orange reachability checks, catch opportunity checks, and 15% dead-zone rejection.
- [G] Build / Tooling / CI Agent upgraded replay runner, board validator, content linter, seed browser, CI workflow, local validation docs, and golden replay fixture.
- [E] Feedback / VFX / Audio Agent added mocked feedback playback for all required Checkpoint 1 event kinds with accessibility reductions and no victory-like loss/near-miss feedback.
- [F] UI / HUD Agent added pure-Rust HUD/debug overlay state driven by shared run/RPG/physics contracts, including first-bounce aim data and replay hash display models.
- [I] QA / Telemetry Agent added `crates/telemetry`, JSONL logging, replay tagging docs, feel survey, bug triage template, and determinism checklist.
- [A] Bucket-feel diagnostics added authored-board catchability sampling. Diagnostic result: 8 authored boards met the 2+ catchable trajectory threshold.
- [G] Corrective pass wired mock plugin registration summaries into the game binary so strict clippy remains green without weakening lint rules.
- [A] Tuning iteration 1 adjusted bucket settings on `boards/feel_fan_cross_01` and `boards/feel_wave_gate_01` without changing physics constants or golden replay data.
- [G] Option C build path added feature-gated Bevy feel-test command while preserving non-interactive CI smoke.
- [F] Option C feel-test scene model added aim adjustment, deterministic shot simulation, first-bounce data, replay hash, HUD/debug summaries, and smoke tests.
- [E] Option C feedback wiring maps feel-test shot results to existing `FeedbackEvent` cue summaries with accessibility reductions.
- [F] Visible shot fix added deterministic trajectory sampling to render a cyan trail and yellow final ball marker when Space fires.
- [A] Tuning iteration 2 added implicit left/right board wall collisions and reduced default restitution/bounce energy.
- [F] Tuning iteration 2 added animated shot playback so the yellow ball moves over time and the cyan trail reveals progressively.

Authored board IDs:
- `boards/feel_fan_01`
- `boards/feel_wave_01`
- `boards/feel_clusters_01`
- `boards/feel_lanes_01`
- `boards/feel_rings_01`
- `boards/feel_spiral_01`
- `boards/feel_fortress_stone_01`
- `boards/feel_fan_cross_01`
- `boards/feel_wave_gate_01`
- `boards/feel_clusters_stone_01`

Validation completed:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p replay_runner`
- `cargo run -p board_validator`
- `cargo run -p content_linter`
- `cargo run -p seed_browser -- --act 1 --archetype fan --count 3`
- `cargo run -p feverfall_game`
- `cargo check -p feverfall_game --features bevy_feel_test`
- `cargo clippy -p feverfall_game --features bevy_feel_test --all-targets -- -D warnings`

Automated feel validation:
- First-bounce prediction exactness is covered by `physics_core::tests::first_bounce_prediction_matches_simulated_circle_peg_collision`.
- No-tunneling/stability is covered by `physics_core::tests::no_tunneling_at_max_speed_against_thin_segment` and `physics_core::tests::stress_10000_randomish_shots_do_not_stick_or_loop_forever`.
- Wall containment is covered by `physics_core::tests::board_walls_keep_sampled_trajectory_inside_horizontal_bounds_at_speed_cap`.
- Reduced-bounce wall behavior is covered by `physics_core::tests::board_wall_rebound_is_stable_and_damped`.
- Bucket catch opportunity is covered by `board_gen::tests::authored_board_bucket_catch_skillfulness_diagnostic_pre_human_feel_scene`; 8 of 10 authored boards met the automated catchability threshold.
- After tuning iteration 1, all 10 authored boards meet the automated 2+ catchable trajectory threshold.

Tuning iteration 1 catch counts:
- `boards/feel_clusters_01`: 4
- `boards/feel_clusters_stone_01`: 5
- `boards/feel_fan_01`: 4
- `boards/feel_fan_cross_01`: 3
- `boards/feel_fortress_stone_01`: 2
- `boards/feel_lanes_01`: 6
- `boards/feel_rings_01`: 2
- `boards/feel_spiral_01`: 2
- `boards/feel_wave_01`: 2
- `boards/feel_wave_gate_01`: 2

Last replay hash:
- `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`

Remaining before Checkpoint 1 exit:
- None. Physics Feel Alpha is approved.
- Bevy 0.19 remains blocked in this environment because it requires `rustc 1.95.0`; current validation uses Bevy 0.18 with `rustc 1.94.0`.

Playable feel-test command:
- `cargo run -p feverfall_game --features bevy_feel_test -- --feel-test`
- Controls: Left/Right or A/D adjusts aim; Space fires a deterministic shot, animates a yellow ball, and reveals a cyan shot trail progressively.

Decision from human:
- Physics Feel Alpha approved for Checkpoint 2.

## Checkpoint 2: Vertical Slice Alpha

Status: automated smoke slice implemented; playable run UI/reward flow remains next work.

Completed:
- [C2-A] Core vertical-slice gameplay loop added `game/src/vertical_slice.rs`.
- [C2-A] Smoke session loads authored `boards/feel_fan_01`, simulates a scripted shot with `physics_core`, promotes events with `game_rules`, updates `run_mode::RunState` and `rpg_mode::CharacterState`, and prints a deterministic summary from `cargo run -p feverfall_game -- --smoke`.
- [C2-B] Content/progression support added `RunState::act1_slice`, Act 1 slice run nodes, reward offers, `CharacterState::act1_slice`, starter stats, gear, skill, and tests.
- [C2-FE] Runtime UI/feedback added `SliceCompletionSummary`, score/orange/catch/replay/progression/feedback fields, feel-test smoke outcome details, and small Bevy completion markers while preserving animated shot playback.
- [C2-G] Tooling added `tests/golden_replays/vertical_slice_feel_fan.replay.json`, `board_path` fixture support in `replay_runner`, CI replay coverage, and local validation docs.
- [C2-I] QA/telemetry added vertical-slice shot result and score/progression telemetry mappings, replay labels, and QA/playtest doc updates.

Validation completed:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p replay_runner`
- `cargo run -p replay_runner -- --replay tests/golden_replays/vertical_slice_feel_fan.replay.json`
- `cargo run -p board_validator`
- `cargo run -p content_linter`
- `cargo run -p seed_browser -- --act 1 --archetype fan --count 3`
- `cargo run -p feverfall_game -- --smoke`
- `cargo check -p feverfall_game --features bevy_feel_test`
- `cargo clippy -p feverfall_game --features bevy_feel_test --all-targets -- -D warnings`

Checkpoint 2 hashes:
- Default minimal replay hash: `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`
- Vertical-slice replay fixture hash: `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`
- Current smoke-session vertical-slice replay hash: `a8112f9a7503ebb21431369ae0f354e7cf0687ba2b5576da3b7d43fa4b411a8a`

Next before Checkpoint 2 exit:
- Build a fuller interactive vertical slice around board/node progression and reward choice presentation.
- Optionally publish a native Windows build after the next playable vertical-slice increment.
