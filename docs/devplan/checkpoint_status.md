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

Status: playable feel-test scene implemented; blocked on human physics-feel judgment before Checkpoint 2.

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
- `598ff57ca69b031c4e487fdd54d07d7c7f2667d20f66ad1be85351ae58ae0630`

Remaining before Checkpoint 1 exit:
- Human feel validation approval from the playable scene, or two additional tuning iterations with specific feedback before proceeding to Checkpoint 2.
- Bevy 0.19 remains blocked in this environment because it requires `rustc 1.95.0`; current validation uses Bevy 0.18 with `rustc 1.94.0`.

Playable feel-test command:
- `cargo run -p feverfall_game --features bevy_feel_test -- --feel-test`
- Controls: Left/Right or A/D adjusts aim; Space fires a deterministic shot.

Decision needed from human:
- Run the playable feel-test scene and approve Physics Feel Alpha for Checkpoint 2, or request tuning iteration 2 with a specific target: too floaty, too chaotic, catch too forgiving, catch too strict, or first bounce unreadable.
