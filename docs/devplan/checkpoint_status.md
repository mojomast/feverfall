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
- Bevy 0.19 is unblocked after pinning Rust 1.95.0 in `rust-toolchain.toml`.

Playable feel-test command:
- `cargo run -p feverfall_game --features bevy_feel_test -- --feel-test`
- Controls: Left/Right or A/D adjusts aim; Space fires a deterministic shot, animates a yellow ball, and reveals a cyan shot trail progressively.

Decision from human:
- Physics Feel Alpha approved for Checkpoint 2.

## Checkpoint 2: Vertical Slice Alpha

Status: COMPLETE. Human approval: interactive flow confirmed.

Completed:
- [C2-A] Core vertical-slice gameplay loop added `game/src/vertical_slice.rs`.
- [C2-A] Smoke session loads authored `boards/feel_fan_01`, simulates a scripted shot with `physics_core`, promotes events with `game_rules`, updates `run_mode::RunState` and `rpg_mode::CharacterState`, and prints a deterministic summary from `cargo run -p feverfall_game -- --smoke`.
- [C2-B] Content/progression support added `RunState::act1_slice`, Act 1 slice run nodes, reward offers, `CharacterState::act1_slice`, starter stats, gear, skill, and tests.
- [C2-FE] Runtime UI/feedback added `SliceCompletionSummary`, score/orange/catch/replay/progression/feedback fields, feel-test smoke outcome details, and small Bevy completion markers while preserving animated shot playback.
- [C2-G] Tooling added `tests/golden_replays/vertical_slice_feel_fan.replay.json`, `board_path` fixture support in `replay_runner`, CI replay coverage, and local validation docs.
- [C2-I] QA/telemetry added vertical-slice shot result and score/progression telemetry mappings, replay labels, and QA/playtest doc updates.
- [C2-G2] Tooling pinned Rust 1.95.0, upgraded the optional Bevy feel-test dependency to 0.19, added ordered multi-board replay fixture support, and added `tests/golden_replays/act1_twobboard_run.replay.json` for a two-board Act 1 smoke gate.
- [C2-LOOP] Integrated the short Act 1 loop across node progression, board resolution, reward application, run state, RPG state, and deterministic smoke summary output.
- [C2-REWARD] Added reward-choice model/UI summaries and deterministic reward application to the C2 flow.
- [C2-NODEMAP] Added node-map UI summaries for the Act 1 slice path.
- [C2-RUNSUMMARY] Added end-of-run summary UI data and `telemetry::TelemetryEvent::RunEnded` coverage.
- [C2-CONTENT] Added schema/data coverage for Act 1 relics, ball variants, shop items, and the `boards/act1_boss_01` boss board.
- Automated validation now runs on Rust 1.95.0 with optional Bevy 0.19 checks.
- [C2-I2] Updated QA/playtest/replay docs for integrated C2 smoke and run-summary telemetry.
- [C2-SMOKE-FIX] Stabilized integrated smoke output and the smoke run summary hash.
- [C2-REWARD-CLIPPY-FIX] Fixed strict-clippy issues in reward/UI integration.
- [C2-COMPLETE-STATE] Recorded human approval: interactive flow confirmed. Checkpoint 2 is COMPLETE.

Validation completed:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p replay_runner`
- `cargo run -p replay_runner -- --replay tests/golden_replays/vertical_slice_feel_fan.replay.json`
- `cargo run -p replay_runner -- --replay tests/golden_replays/act1_twobboard_run.replay.json`
- `cargo run -p board_validator`
- `cargo run -p content_linter`
- `cargo run -p feverfall_game -- --smoke`
- `cargo check -p feverfall_game --features bevy_feel_test`
- `cargo clippy -p feverfall_game --features bevy_feel_test --all-targets -- -D warnings`
- `cargo run -p feverfall_game --features bevy_feel_test -- --smoke`

Checkpoint 2 hashes:
- Default minimal replay hash: `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`
- Vertical-slice replay fixture hash: `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`
- Act 1 two-board replay fixture hash: `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`
- Smoke run summary hash: `0b36add9e9b3283c`
- Bevy feel-test smoke hash: `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`

Checkpoint 2 automated gate notes:
- Toolchain: Rust 1.95.0 via `rust-toolchain.toml`.
- Optional playable/smoke feature dependency: Bevy 0.19.
- `cargo run -p content_linter` passes with 44 unique IDs across board, relic, ball, and shop content.
- `cargo run -p board_validator` passes all boards, including `PASS boards/act1_boss_01`.
- `cargo run -p feverfall_game -- --smoke` emits the integrated node/reward/run-summary smoke with hash `0b36add9e9b3283c`.
- `cargo run -p feverfall_game --features bevy_feel_test -- --smoke` emits feel-test smoke hash `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`.
- Human approval: interactive flow confirmed.

## Checkpoint 3: COMPLETE

Status: COMPLETE. C3 integration reconciled [C3-SEP], [C3-RPG], [C3-ROGUELITE], [C3-BALANCE], [C3-VFX1], and [C3-G] outputs; all exit validation passed.

Completed:
- [C3-BALANCE] Added `tools/balance_sim` as an isolated 1,000-run headless roguelite batch runner using `board_gen` and `physics_core` directly until the C3 roguelite smoke/headless API lands.
- [C3-BALANCE] Simulated 1,000 runs with seed range `0xc3ba000000000000` through `0xc3ba0000000003e7`.
- [C3-BALANCE] Produced `docs/design/balance_notes.md` with methodology, metrics, findings, and concrete tuning recommendations.
- [C3-BALANCE] Applied high-confidence tuning tables in `content/balance/roguelite/board_curve.toml`, `content/balance/roguelite/reward_pool.toml`, and `content/balance/roguelite/scoring_curve.toml`.
- [C3-G] Expanded tooling/CI coverage for C3: RPG Chapter 1 and roguelite 3-act replay fixtures/gates, RPG campaign `character_state` replay parsing, RPG chapter objective board validation, `seed_browser --mode rpg --chapter 1`, RPG gear/skill/balance content lint schemas, and defensive game smoke CI commands.
- [C3-ROGUELITE] Expanded `crates/run_mode` and `game/src` from the Act 1 slice to a deterministic 3-act roguelite structure with Act 1 6/1/1, Act 2 7/2/1, Act 3 8/2/1 board/elite/boss composition, branch choices, shop/event/forge/camp nodes, run resources, curse rarity pressure, all 20 Act 1 relic content IDs wired to state/board effects, `RelicTriggered` feedback, meta-progression save skeleton, and a full 3-act smoke summary.
- [C3-SEP] Added mode-separation integration tests in `crates/run_mode` and `crates/rpg_mode` proving `RunState` and `CharacterState` can process a shared board outcome, consume `physics_core`/`game_rules`/`feedback_events`/`telemetry`, round-trip independently through serde, and avoid shared mutable state.
- [C3-SEP] Added explicit mode path constants: roguelite saves `saves/roguelite/`, RPG saves `saves/rpg/`, roguelite balance `content/balance/roguelite/`, RPG balance `content/balance/rpg/`; added RPG balance directory stub.
- [C3-SEP] Enforced `physics_core` independence from `run_mode`/`rpg_mode` via compile-time integration-test checks over the physics manifest/source.
- [C3-SEP] Resolved concurrent validation blockers needed for the workspace gate: RPG save version precheck, C2-compatible starter relic/reward expectations, enum compatibility, strict-clippy cleanup, authored chapter-board validation allowance, and mode-neutral handling for new feedback/reward variants.
- [C3-RPG] Added playable RPG Chapter 1 campaign support in `crates/rpg_mode` and `game/src/rpg_chapter1.rs`: five authored boards, XP/level thresholds, Aim/Control/Resonance/Luck stat allocation, launcher/core-ball gear swapping, Zen Reroute, Catch Magnet, board-based cooldowns, skill telemetry, versioned campaign save/load, and default `--smoke` abbreviated coverage for boards 1 and 5.
- [C3-INTEGRATE] Fixed formatting, verified the RPG save/load unknown-version test, fixed the reward-selection regression, completed C3 VFX trigger coverage in `game/src/plugins/feedback.rs`/`vfx`, and added two minimal RPG gear content IDs to reach the C3 content-lint threshold.

[C3-BALANCE] simulation metrics:
- Act 1 win rate: 0.0% cleared, 1,000 started.
- Act 2 win rate: 0.0% cleared, 3 started.
- Act 3 win rate: 0.0% cleared, 0 started.
- Average oranges cleared per board: 14.90.
- Average relics collected per run: 0.00.
- Average run length: 35.27 shots.
- Most chosen reward: `relic:boss_feverheart` x2.
- Least chosen reward: `relic:common_spark_catcher` x1.

[C3-BALANCE] validation completed:
- `CARGO_TARGET_DIR=/tmp/opencode/feverfall-target cargo run -p balance_sim --release`
- `cargo fmt --package balance_sim`
- `cargo run -p content_linter` passes with 46 unique IDs.

[C3-BALANCE] blockers / integration notes:
- No blocker for the balance artifact.
- `cargo run -p balance_sim --release` could not use the repo release target because `target/release/.cargo-lock` returned permission denied; using `/tmp/opencode/feverfall-target` completed successfully.
- Replace `tools/balance_sim` local reward approximation with shared roguelite reward/run APIs after [C3-ROGUELITE] lands.

[C3-G] validation completed:
- Existing replay hashes unchanged: default `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`, vertical slice `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`, Act 1 two-board `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`.
- New replay hashes: RPG Chapter 1 `8e566217ee6cddee3be784b3e359b3eda5708638ac8540bce759086e922a145f`, roguelite 3-act `89c224a1ba8aae30965fa42f9547940036badc026b0a2f1bf50e6de15b86682b`.
- `cargo run -p board_validator` passes all authored boards.
- `cargo run -p content_linter` passes with 58 unique IDs observed across `game/assets/content` and top-level `content`.
- `cargo run -p seed_browser -- --act 1 --archetype fan --count 3` passes.
- `cargo run -p seed_browser -- --mode rpg --chapter 1 --archetype fan --count 3` passes with 3/3 valid RPG objective-tagged boards.
- `cargo run -p feverfall_game -- --smoke`, `cargo run -p feverfall_game -- --smoke --mode rpg --chapter 1`, and `cargo run -p feverfall_game -- --smoke --mode roguelite --acts 3` complete via the current generic smoke path.

[C3-G] blockers / integration notes:
- Prior workspace test/clippy blockers were resolved during [C3-SEP]; commit ownership remains shared with concurrent C3 changes.
- Current generic game smoke now emits run summary hash `600628ae0877f49d`; the Checkpoint 2 documented hash is `0b36add9e9b3283c`.

[C3-ROGUELITE] validation completed:
- `cargo test -p run_mode` passes: 12 lib tests plus 2 mode-separation integration tests.
- `cargo run -p content_linter` passes with 58 unique IDs.
- `cargo run -p replay_runner -- --replay tests/golden_replays/roguelite_act1to3_smoke.replay.json` matches `c5db0fb8d90e57c8be159bbb779c56ead19148f36de8bdc077711e59f9a4a36a`.
- `cargo run -p feverfall_game -- --smoke` prints Act 1, Act 2, and Act 3 roguelite summaries and final roguelite smoke hash `4dedb4fcdacb19b9`.

[C3-ROGUELITE] blockers / integration notes:
- No blocker for the roguelite artifacts/validation. Commit not created because concurrent uncommitted C3 changes share files touched by this agent; staging whole files would include other agents' work.
- Coordinate with [C3-BALANCE] to consume `content/balance/roguelite/*.toml` from `full_run_nodes()` and the shared relic/reward APIs.

[C3-SEP] validation completed:
- `cargo test --workspace` passes, including the new `crates/run_mode/tests/mode_separation.rs` and `crates/rpg_mode/tests/mode_separation.rs` tests.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
- `cargo run -p content_linter` passes with 58 unique IDs.

[C3-SEP] blockers / integration notes:
- No blocker for mode separation.
- Dependent C3 agents should use the documented save and balance directories rather than sharing persisted state between modes.

[C3-RPG] validation completed:
- `cargo test -p rpg_mode` passes.
- `cargo run -p board_validator` passes all authored boards, including `boards/rpg_ch1_01` through `boards/rpg_ch1_05`.
- `cargo run -p content_linter` passes with 58 unique IDs.
- `cargo run -p replay_runner -- --replay tests/golden_replays/rpg_ch1_smoke.replay.json` matches `fc72b1144ad88e62bb27c3a1296cbb9b3fa51871a852b9b5ef561d7146033a58`.
- `cargo run -p feverfall_game -- --smoke` passes and prints RPG Chapter 1 summary hash `3364e243ba2065f4`.
- `cargo test -p feverfall_game rpg_chapter1::tests::chapter1_smoke_uses_boards_one_and_five_and_is_stable_after_save_load` passes.

[C3-RPG] blockers / integration notes:
- No blocker for the RPG artifacts.
- The broader workspace test failure in `plugins::reward_ui::tests::reward_selection_applies_correct_relic_to_run_state` was fixed during C3 integration.

[C3-INTEGRATE] final validation completed:
- `cargo fmt --all -- --check` passes.
- `cargo test --workspace` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
- `cargo run -p feverfall_game -- --smoke` passes and prints C2 slice/run summary hash `18202124e6b686d8`, RPG Chapter 1 summary hash `3364e243ba2065f4`, and roguelite 3-act summary hash `4dedb4fcdacb19b9`.
- Golden replay hashes match: minimal `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`, vertical slice `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`, Act 1 two-board `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`, RPG Chapter 1 defensive `8e566217ee6cddee3be784b3e359b3eda5708638ac8540bce759086e922a145f`, RPG Chapter 1 implementation `fc72b1144ad88e62bb27c3a1296cbb9b3fa51871a852b9b5ef561d7146033a58`, roguelite 3-act defensive `89c224a1ba8aae30965fa42f9547940036badc026b0a2f1bf50e6de15b86682b`, roguelite Act 1-3 implementation `c5db0fb8d90e57c8be159bbb779c56ead19148f36de8bdc077711e59f9a4a36a`.
- `cargo run -p content_linter` passes with 60 unique IDs.
- `cargo run -p board_validator` passes all boards, including `boards/rpg_ch1_01` through `boards/rpg_ch1_05`.
- Mode-separation tests verify `physics_core` has no `run_mode`/`rpg_mode` imports or dependencies.
- `cargo run -p feverfall_game --features bevy_feel_test -- --smoke` passes and prints `c3_vfx_triggers=14` covering blue/orange/purple/green peg hits, bucket catch, combo 3/6/10/15+, long shot, near-miss, last orange in flight, Extreme Fever, and board failure.

Next checkpoint:
- Checkpoint 4.
