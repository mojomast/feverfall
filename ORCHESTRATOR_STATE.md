# Orchestrator State

## Current Checkpoint

Checkpoint 4: IN PROGRESS. [C4-G] final tooling/release gate is implemented and locally validated; [C4-UI] full placeholder screen coverage is complete and validated; Windows artifact production requires rerunning the workflow from a pushed C4 ref.

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
- [G] Added a manually runnable native Windows GitHub Actions feel-test build workflow that uploads the `.exe` and SHA-256 checksum artifacts. Not yet run by this agent.
- [G] Updated feature-built feel-test launch behavior so the native Windows artifact opens the playable scene on double-click/no args, with `--smoke` retained for non-interactive smoke mode.
- [F] Fixed Bevy feel-test shot visibility: Space now renders a cyan deterministic trajectory trail and yellow final ball indicator instead of only instant hit markers.
- [A] Tuning iteration 2 added deterministic left/right board wall collisions and reduced restitution/bounce energy.
- [F] Tuning iteration 2 replaced static full-path display with animated shot playback: a yellow ball moves along the sampled trajectory and the cyan trail reveals progressively.
- Human approved Physics Feel Alpha after testing the tuned native Windows build.
- [C2-A] Core vertical slice added a deterministic smoke session that loads `boards/feel_fan_01`, simulates a scripted shot, promotes physics to game events, and updates run/RPG state.
- [C2-B] Content/progression added Act 1 slice defaults, run nodes, starter resources, rewards, character stats, gear, skill, and tests.
- [C2-FE] Runtime UI/feedback added slice completion summaries, replay/score/progression feedback fields, and small Bevy completion markers while preserving aim/fire animation.
- [C2-G] Tooling added a vertical-slice replay fixture using authored board data and CI/local validation coverage.
- [C2-I] QA/Telemetry added vertical-slice shot/score/progression telemetry events, replay labels, and playtest/determinism doc updates.
- [C2-G2] Tooling upgraded the project toolchain to Rust 1.95.0, moved the optional Bevy feel-test dependency to Bevy 0.19, added a two-board Act 1 replay fixture, and wired CI/local validation to verify its hash.
- [C2-LOOP] Integrated the short Act 1 run loop across node progression, board resolution, reward application, run summary, and deterministic smoke output.
- [C2-REWARD] Added reward-choice modeling/UI summaries and deterministic reward application for the C2 run path.
- [C2-NODEMAP] Added node-map UI summaries for the Act 1 slice path.
- [C2-RUNSUMMARY] Added end-of-run summary UI data and telemetry coverage.
- [C2-CONTENT] Added C2 content schema/data for relics, balls, shops, and Act 1 boss board content.
- [C2-SMOKE-FIX] Updated smoke coverage to include the integrated run summary hash and full C2 automated gates.
- [C2-REWARD-CLIPPY-FIX] Fixed reward/UI strict-clippy issues while preserving deterministic smoke behavior.
- [C2-COMPLETE-STATE] Recorded human approval: interactive flow confirmed. Checkpoint 2 is COMPLETE.
- [C3-BALANCE] Ran 1,000 deterministic roguelite headless balance simulations with seeds `0xc3ba000000000000` through `0xc3ba0000000003e7`, produced `docs/design/balance_notes.md`, added `tools/balance_sim`, and applied high-confidence roguelite tuning tables under `content/balance/roguelite/`.
- [C3-G] Tooling/CI coverage implemented defensively for RPG Chapter 1 and roguelite 3-act smoke: replay fixtures/hash gates, RPG character snapshot replay parsing, RPG objective board validation, RPG seed-browser mode, RPG gear/skill/balance lint schemas, CI commands, and local/shared-contract docs.
- [C3-ROGUELITE] Expanded roguelite run mode to a deterministic 3-act structure with Act 1 6 normal/1 elite/1 boss, Act 2 7/2/1, Act 3 8/2/1, branching choices, shop/event/forge/camp nodes, all 20 Act 1 relic content IDs wired to board/state effects, curse risk/reward pressure, hearts/coins/sparks/keys resources, meta-progression save skeleton, 3-act game smoke output, and `tests/golden_replays/roguelite_act1to3_smoke.replay.json`.
- [C3-RPG] Added playable RPG Chapter 1 campaign support with five authored boards, XP/level/stat progression, launcher/core-ball gear swapping, Zen Reroute, Catch Magnet, board-based cooldowns, skill telemetry, versioned campaign save/load, default smoke coverage for boards 1 and 5, and `tests/golden_replays/rpg_ch1_smoke.replay.json`.
- [C3-SEP] Enforced roguelite/RPG mode separation with integration tests in `crates/run_mode` and `crates/rpg_mode`, documented separate save/balance paths, added RPG balance directory stub, and verified `physics_core` has no `run_mode`/`rpg_mode` dependency or import.
- [C3-INTEGRATE] Reconciled all C3 outputs, fixed formatting/test regressions, completed C3 VFX trigger coverage, raised content lint to 60 unique IDs, and validated the full C3 exit gate.
- [C4-ACT4] Added optional Act 4 Final Seed mastery to roguelite run mode: 3-key unlock, 4 seeded high-risk boards plus final boss, combined scripted boss mechanic, `Full Fever Cleared` meta record, better Act 4 meta unlock offers, and full Act 1-4 smoke summary.
- [C4-CONTENT] Expanded the full content pack to 60 relics, 20 ball variants, 40 RPG gear items, 36 RPG skills, and 80 authored boards; added structured boss mechanics for all boss-tagged boards and documented counts in `docs/design/content_manifest.md`.
- [C4-RPG-CH2TO5] Added RPG Chapters 2-5 campaign catalog support in `crates/rpg_mode`: Ch2 has 12 obstacle/active-skill/score-objective boards, Ch3 has 15 gear-synergy boards, Ch4 has 15 multi-objective boards, and Ch5 has 4 normalized-stat mastery boards with leaderboard hashes. Campaign completion now requires all 5 chapters and unlocks the `campaign/mastery_mode_unlocked` flag on `CharacterState`.
- [C4-VFX2] Completed full reactive feedback trigger map: explicit launch/long-shot/lucky-bounce feedback kinds, five audio buses, combo pitch cap/chord clustering, combo rail state, relic category flash colors, board archetype ambience, and accessibility reductions.
- [C4-UI] Added production-placeholder UI screen models for main menu, settings, roguelite map/shop/forge/event/relic bar, RPG chapter/gear/skill/campaign screens, keyboard focus contracts, 1280x720/1920x1080 layout smoke coverage, and F3 debug overlay fields.

## Active Workstreams

- Physics Core & Feel: Checkpoint 1 feel approved; simulator, first-bounce, no-tunneling, wall bounds, and bucket diagnostics passing.
- Checkpoint 2 Core Loop: deterministic vertical-slice smoke session passing through physics, game rules, node progression, reward selection, run state, RPG state, HUD, feedback, telemetry, run summary, and replay/run-summary hashes.
- Content / Progression: Act 1 slice defaults, node path, reward offers, relic/ball/shop content, and boss board validation are available in shared contracts and content data.
- UI / Feedback / Telemetry: node-map, reward, run-summary, slice summaries, and telemetry mappings pass automated validation; human interactive-flow confirmation is recorded.
- Checkpoint gate: Checkpoint 2 COMPLETE. Human approval: interactive flow confirmed.
- Checkpoint 3: COMPLETE. [C3-BALANCE], [C3-ROGUELITE], [C3-RPG], [C3-SEP], [C3-G], and [C3-INTEGRATE] complete with required validations passing.
- [C4-CONTENT]: COMPLETE. Content linter, board validator, and touched crate tests pass with full C4 content quantity targets met.
- [C4-RPG-CH2TO5]: COMPLETE. Default smoke prints RPG Ch1 board 1, Ch3 board 1, and Ch5 board 1 campaign summaries with hashes before the preserved Chapter 1 save/load smoke summary.
- [C4-VFX2]: COMPLETE. Bevy feel-test clippy and smoke pass with `c4_vfx_triggers=21` covering the full reactive feedback trigger map.
- [C4-UI]: COMPLETE. Game tests, Bevy feel-test strict clippy, and Bevy feel-test smoke pass with `screens=13`, `keyboard=true`, `layout=true`, and `f3_fields=5` in registration smoke.
- Tooling gate: CI/local validation includes default, vertical-slice, Act 1 two-board, RPG Chapter 1, and roguelite 3-act replay hash gates; content lint; board validation; roguelite/RPG seed-browser smokes; default/RPG/roguelite game smoke commands; and Bevy feel-test smoke/clippy gates.

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
- [F] UI / HUD Agent: visible ball/trajectory fix for playable feel-test scene.
- [A] Physics Core & Feel Agent: tuning iteration 2 for wall bounds and reduced bounce.
- [F] UI / HUD Agent: animated trajectory playback fix.
- [C2-A] Core Vertical Slice Agent: deterministic smoke gameplay loop.
- [C2-B] Content Progression Agent: Act 1 slice run/character/reward defaults.
- [C2-FE] Runtime UI Feedback Agent: slice completion HUD/feedback and Bevy markers.
- [C2-G] Tooling Validation Agent: vertical-slice replay fixture and CI/local docs.
- [C2-I] QA Telemetry Agent: vertical-slice telemetry events and QA docs.
- [C2-G2] Tooling & CI Update Agent: Rust 1.95.0 / Bevy 0.19 update and two-board replay CI coverage.
- [C2-LOOP] Core Loop Agent: integrated short Act 1 run flow.
- [C2-REWARD] Reward Agent: reward-choice flow and application.
- [C2-NODEMAP] Node Map Agent: node-map UI summary.
- [C2-RUNSUMMARY] Run Summary Agent: end-of-run summary and telemetry shape.
- [C2-CONTENT] Content Agent: relic, ball, shop, and boss-board content schemas/data.
- [C2-SMOKE-FIX] Smoke Fix Agent: integrated smoke validation/hash stabilization.
- [C2-REWARD-CLIPPY-FIX] Reward Clippy Fix Agent: strict-clippy cleanup.
- [C3-BALANCE] Balance Pass Agent: roguelite batch simulation, tuning notes, and balance tables.
- [C3-G] Tooling Update Agent: C3 CI/tooling coverage; implementation complete, commit blocked by failing workspace validation outside intended files.
- [C3-ROGUELITE] Full Roguelite Act 1-3 Agent: 3-act run structure, relic wiring, curse/resources/meta skeleton, smoke summary, and golden replay.
- [C3-RPG] RPG Chapter 1 Campaign Agent: authored boards, progression, gear, active skills, save/load, smoke summary, and golden replay.
- [C3-SEP] Mode Separation & Contracts Agent: mode independence tests, save/balance path contracts, physics dependency guard, and workspace validation cleanup.
- [C3-INTEGRATE] Checkpoint 3 Integration & Validation Agent: final formatting/test/clippy/smoke/replay/content/board/Bevy validation, C3 VFX trigger coverage, and checkpoint state updates.
- [C4-ACT4] Roguelite Act 4 Agent: optional Final Seed mastery path, seeded Act 4 board specs, combined final boss mechanic, meta-progression mastery record, smoke/tests/docs validation.
- [C4-G] Final Tooling & Release Agent: implemented `--smoke-full`, updated CI/Windows workflow, added README and release checklist, and locally validated the full release gate.
- [C4-CONTENT] Full Content Pack Agent: full devplan content quantities, structured boss mechanic schema, content manifest, and content/board validation updates.
- [C4-RPG-CH2TO5] RPG Chapters 2-5 Agent: campaign catalog, mastery unlock contract, campaign smoke expansion, tests, shared-contract docs, and checkpoint status updates.
- [C4-VFX2] Full Juice Polish Agent: reactive feedback trigger map, VFX/audio cue markers, combo rail, relic flash colors, board ambience, tests, docs, and validation updates.
- [C4-UI] Full UI Polish Agent: production-placeholder screen models, keyboard focus, layout smoke, F3 debug overlay, validation, and documentation updates.

## Files Changed

- `Cargo.toml`
- `rust-toolchain.toml`
- `.gitignore`
- `.github/workflows/ci.yml`
- `.github/workflows/windows-feel-test.yml`
- `crates/content_schema/*`
- `crates/physics_core/*`
- `crates/game_rules/*`
- `crates/board_gen/*`
- `crates/run_mode/*`
- `crates/rpg_mode/*`
- `crates/rpg_mode/src/lib.rs`
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
- `game/src/vertical_slice.rs`
- `tests/golden_replays/minimal_test.replay.json`
- `tests/golden_replays/vertical_slice_feel_fan.replay.json`
- `tests/golden_replays/act1_twobboard_run.replay.json`
- `tests/golden_replays/rpg_chapter1_smoke.replay.json`
- `tests/golden_replays/rpg_ch1_smoke.replay.json`
- `tests/golden_replays/roguelite_3act_smoke.replay.json`
- `tests/golden_replays/roguelite_act1to3_smoke.replay.json`
- `game/assets/content/balls/*`
- `game/assets/content/relics/*`
- `game/assets/content/shops/*`
- `game/assets/content/boards/act1_boss_01/*`
- `game/assets/content/boards/rpg_ch1_*.json`
- `game/assets/content/rpg_gear/*`
- `game/assets/content/rpg_skills/*`
- `game/assets/content/rpg_gear/resonance_charm.json`
- `game/assets/content/rpg_gear/luck_trinket.json`
- `game/src/rpg_chapter1.rs`
- `game/src/main.rs`
- `game/src/plugins/node_map_ui.rs`
- `game/src/plugins/reward_ui.rs`
- `game/src/plugins/run_summary_ui.rs`
- `crates/telemetry/*`
- `docs/playtesting/feel_survey.md`
- `docs/qa/bug_triage_template.md`
- `docs/qa/determinism_checklist.md`
- `docs/qa/replay_tagging.md`
- `ORCHESTRATOR_STATE.md`
- `tools/balance_sim/*`
- `content/balance/roguelite/board_curve.toml`
- `content/balance/roguelite/reward_pool.toml`
- `content/balance/roguelite/scoring_curve.toml`
- `content/balance/rpg/.gitkeep`
- `docs/design/balance_notes.md`
- `docs/design/content_manifest.md`
- `game/assets/content/relics/act2/*`
- `game/assets/content/relics/act3/*`
- `game/assets/content/balls/c4/*`
- `game/assets/content/rpg_gear/c4/*`
- `game/assets/content/rpg_skills/c4/*`
- `game/assets/content/boards/c4_act1/*`
- `game/assets/content/boards/c4_act2/*`
- `game/assets/content/boards/c4_act3/*`
- `game/assets/content/boards/c4_bosses/*`
- `game/assets/content/boards/c4_rpg_ch1/*`
- `game/assets/content/boards/c4_rpg_ch2/*`
- `game/assets/content/boards/c4_rpg_ch3/*`
- `game/assets/content/boards/c4_rpg_ch4/*`
- `game/assets/content/boards/c4_rpg_ch5/*`

## Validation Commands Run

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p replay_runner`
- `cargo run -p replay_runner -- --replay tests/golden_replays/vertical_slice_feel_fan.replay.json`
- `cargo run -p replay_runner -- --replay tests/golden_replays/act1_twobboard_run.replay.json`
- `cargo run -p replay_runner -- --replay tests/golden_replays/rpg_chapter1_smoke.replay.json`
- `cargo run -p replay_runner -- --replay tests/golden_replays/roguelite_3act_smoke.replay.json`
- `cargo run -p board_validator`
- `cargo run -p content_linter`
- `cargo test -p rpg_mode`
- `cargo run -p replay_runner -- --replay tests/golden_replays/rpg_ch1_smoke.replay.json`
- `cargo test -p feverfall_game rpg_chapter1::tests::chapter1_smoke_uses_boards_one_and_five_and_is_stable_after_save_load`
- `cargo test -p run_mode`
- `cargo run -p replay_runner -- --replay tests/golden_replays/roguelite_act1to3_smoke.replay.json`
- `cargo run -p seed_browser -- --act 1 --archetype fan --count 3`
- `cargo run -p seed_browser -- --mode rpg --chapter 1 --archetype fan --count 3`
- `cargo run -p feverfall_game -- --smoke`
- `cargo run -p feverfall_game -- --smoke --mode rpg --chapter 1`
- `cargo run -p feverfall_game -- --smoke --mode roguelite --acts 3`
- `cargo check -p feverfall_game --features bevy_feel_test`
- `cargo clippy -p feverfall_game --features bevy_feel_test --all-targets -- -D warnings`
- `cargo run -p feverfall_game --features bevy_feel_test -- --smoke`
- Docker Windows cross-build: `cargo build -p feverfall_game --features bevy_feel_test --release --target x86_64-pc-windows-gnu`
- Native Windows feel-test workflow added but not run locally: `Windows Feel-Test Build`.
- `CARGO_TARGET_DIR=/tmp/opencode/feverfall-target cargo run -p balance_sim --release`
- `cargo fmt --package balance_sim`
- `cargo run -p content_linter`
- [C3-SEP] `cargo test --workspace`
- [C3-SEP] `cargo clippy --workspace --all-targets -- -D warnings`
- [C3-SEP] `cargo run -p content_linter`
- [C3-INTEGRATE] `cargo fmt --all -- --check`
- [C3-INTEGRATE] `cargo test --workspace`
- [C3-INTEGRATE] `cargo clippy --workspace --all-targets -- -D warnings`
- [C3-INTEGRATE] `cargo run -p feverfall_game -- --smoke`
- [C3-INTEGRATE] `cargo run -p replay_runner`
- [C3-INTEGRATE] `cargo run -p replay_runner -- --replay tests/golden_replays/vertical_slice_feel_fan.replay.json`
- [C3-INTEGRATE] `cargo run -p replay_runner -- --replay tests/golden_replays/act1_twobboard_run.replay.json`
- [C3-INTEGRATE] `cargo run -p replay_runner -- --replay tests/golden_replays/rpg_chapter1_smoke.replay.json`
- [C3-INTEGRATE] `cargo run -p replay_runner -- --replay tests/golden_replays/rpg_ch1_smoke.replay.json`
- [C3-INTEGRATE] `cargo run -p replay_runner -- --replay tests/golden_replays/roguelite_3act_smoke.replay.json`
- [C3-INTEGRATE] `cargo run -p replay_runner -- --replay tests/golden_replays/roguelite_act1to3_smoke.replay.json`
- [C3-INTEGRATE] `cargo run -p content_linter`
- [C3-INTEGRATE] `cargo run -p board_validator`
- [C3-INTEGRATE] `cargo run -p feverfall_game --features bevy_feel_test -- --smoke`
- [C4-G] `cargo fmt --all -- --check`
- [C4-G] `cargo test --workspace`
- [C4-G] `cargo run -p feverfall_game -- --smoke-full`
- [C4-CONTENT] `cargo fmt --package content_schema --package board_gen --package content_linter`
- [C4-CONTENT] `cargo test -p content_schema -p board_gen -p content_linter`
- [C4-CONTENT] `cargo run -p content_linter`
- [C4-CONTENT] `cargo run -p board_validator`
- [C4-CONTENT] `cargo run -p replay_runner`
- [C4-CONTENT] `cargo run -p replay_runner -- --replay tests/golden_replays/rpg_ch1_smoke.replay.json`
- [C4-CONTENT] `cargo run -p replay_runner -- --replay tests/golden_replays/roguelite_act1to3_smoke.replay.json`
- [C4-RPG-CH2TO5] `cargo test -p rpg_mode`
- [C4-RPG-CH2TO5] `cargo run -p board_validator`
- [C4-RPG-CH2TO5] `cargo run -p content_linter`
- [C4-RPG-CH2TO5] `cargo run -p feverfall_game -- --smoke`
- [C4-RPG-CH2TO5] `cargo run -p replay_runner -- --replay tests/golden_replays/rpg_ch1_smoke.replay.json`
- [C4-RPG-CH2TO5] `cargo fmt --all -- --check`
- [C4-RPG-CH2TO5] `cargo test -p feverfall_game rpg_chapter1`
- [C4-VFX2] `cargo fmt --all`
- [C4-VFX2] `cargo clippy -p feverfall_game --features bevy_feel_test -- -D warnings`
- [C4-VFX2] `cargo run -p feverfall_game --features bevy_feel_test -- --smoke`
- [C4-VFX2] `cargo test -p feverfall_game`
- [C4-ACT4] `cargo fmt --all`
- [C4-ACT4] `cargo test -p run_mode`
- [C4-ACT4] `cargo test -p feverfall_game roguelite_act4_smoke_preserves_three_act_base_and_records_mastery`
- [C4-ACT4] `cargo run -p content_linter`
- [C4-ACT4] `cargo run -p board_validator`
- [C4-ACT4] `cargo run -p replay_runner -- --replay tests/golden_replays/roguelite_act1to3_smoke.replay.json`
- [C4-ACT4] `cargo run -p feverfall_game -- --smoke`
- [C4-ACT4] `cargo run -p feverfall_game -- --smoke-full`

## Passing Validation

- Formatting, strict clippy, workspace tests, replay runner, vertical-slice replay runner, Act 1 two-board replay runner, board validator, content linter, default game smoke, Bevy feel-test check/clippy, and Bevy feel-test smoke all pass after Checkpoint 2 integration. Human approval: interactive flow confirmed.
- `cargo test --workspace` includes 16 `physics_core` tests, including first-bounce prediction matching simulation, no tunneling, no NaN, bucket catch, peg clear timing, left/right board wall confinement, damped wall rebound, trajectory sampling determinism, and 10,000 random-ish stress shots.
- `cargo run -p board_validator` passes authored boards, including `PASS boards/act1_boss_01`.
- `cargo run -p replay_runner` matches the golden replay hash.
- `cargo run -p replay_runner -- --replay tests/golden_replays/vertical_slice_feel_fan.replay.json` matches vertical-slice hash `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`.
- `cargo run -p replay_runner -- --replay tests/golden_replays/act1_twobboard_run.replay.json` matches Act 1 two-board hash `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`.
- `cargo run -p content_linter` passes with 44 unique IDs.
- `cargo run -p feverfall_game -- --smoke` prints plugin registration, Checkpoint 2 vertical-slice summary, deterministic feel-test smoke summary, node/reward/run-summary output, and smoke run summary hash `0b36add9e9b3283c`.
- `cargo run -p feverfall_game --features bevy_feel_test -- --smoke` passes with feel-test smoke hash `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`.
- Automated bucket diagnostics after tuning iteration 1: all 10 authored boards meet the 2+ catchable trajectory threshold.
- Default game smoke prints a deterministic feel-test summary and exits.
- Feature-gated playable Bevy feel-test compiles and passes clippy with `bevy_feel_test` enabled.
- Bevy feel-test shot visibility now includes a trajectory trail and final ball marker driven by deterministic physics trajectory sampling.
- Bevy feel-test shot playback now animates the ball over time instead of drawing only the final static path.
- Windows binary built successfully at `target/x86_64-pc-windows-gnu/release/feverfall_game.exe`.
- Windows binary SHA-256: `dac381bb4cbd8c764a779cf9a9bac80cb2f26f505ac4f26e8428701f1ef5b652`.
- Native GitHub Actions Windows artifact workflow is available but has not been run by this agent.
- [C3-BALANCE] Headless balance simulation passed: 1,000 runs, seed range `0xc3ba000000000000`-`0xc3ba0000000003e7`; metrics were Act 1 win rate 0.0%, Act 2 win rate 0.0% from 3 starts, Act 3 0 starts, average oranges cleared per board 14.90, average relics collected 0.00, average run length 35.27 shots.
- [C3-BALANCE] `cargo run -p content_linter` passes with 46 unique IDs after balance-table changes.
- [C3-G] Existing golden replay hashes are unchanged: default `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`, vertical slice `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`, Act 1 two-board `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`.
- [C3-G] New replay hashes match: RPG Chapter 1 `8e566217ee6cddee3be784b3e359b3eda5708638ac8540bce759086e922a145f`, roguelite 3-act `89c224a1ba8aae30965fa42f9547940036badc026b0a2f1bf50e6de15b86682b`.
- [C3-G] `cargo run -p board_validator` passes all authored boards, including `PASS boards/act1_boss_01`.
- [C3-G] `cargo run -p content_linter` passes with 58 unique IDs observed across `game/assets/content` and top-level `content`.
- [C3-G] `cargo run -p seed_browser -- --act 1 --archetype fan --count 3` and `cargo run -p seed_browser -- --mode rpg --chapter 1 --archetype fan --count 3` pass; RPG mode generated 3/3 valid objective-tagged boards.
- [C3-G] `cargo run -p feverfall_game -- --smoke`, `--smoke --mode rpg --chapter 1`, and `--smoke --mode roguelite --acts 3` all complete via the current generic smoke path.
- [C3-ROGUELITE] `cargo test -p run_mode` passes: 12 lib tests plus 2 mode-separation integration tests.
- [C3-ROGUELITE] `cargo run -p content_linter` passes with 58 unique IDs.
- [C3-ROGUELITE] `cargo run -p replay_runner -- --replay tests/golden_replays/roguelite_act1to3_smoke.replay.json` matches `c5db0fb8d90e57c8be159bbb779c56ead19148f36de8bdc077711e59f9a4a36a`.
- [C3-ROGUELITE] `cargo run -p feverfall_game -- --smoke` prints all three roguelite act summaries and final roguelite smoke hash `4dedb4fcdacb19b9`.
- [C3-ROGUELITE] Artifact SHA-256: `crates/run_mode/src/lib.rs` `585e0a1a6a9f67b890972ecd53ea34f5254700461ab0e34205afbef822c4a822`; `game/src/vertical_slice.rs` `1f5f857d98db278e29b83428b3381d3005a7cb935b931cfe9f79307580861721`; `tests/golden_replays/roguelite_act1to3_smoke.replay.json` `ad0cbc7666a74e6d36b37fe0345f94ac131587fae390c961949beb7f059566d8`.
- [C3-RPG] `cargo test -p rpg_mode` passes with Chapter 1 XP/leveling, stat allocation, gear swap, skill cooldown, save/load, and unknown-version coverage.
- [C3-RPG] `cargo run -p board_validator` passes all authored boards, including `PASS boards/rpg_ch1_01` through `PASS boards/rpg_ch1_05`.
- [C3-RPG] `cargo run -p content_linter` passes with 58 unique IDs.
- [C3-RPG] `cargo run -p replay_runner -- --replay tests/golden_replays/rpg_ch1_smoke.replay.json` matches `fc72b1144ad88e62bb27c3a1296cbb9b3fa51871a852b9b5ef561d7146033a58`.
- [C3-RPG] `cargo run -p feverfall_game -- --smoke` passes and prints RPG Chapter 1 smoke hash `3364e243ba2065f4`.
- [C3-RPG] Focused game test `cargo test -p feverfall_game rpg_chapter1::tests::chapter1_smoke_uses_boards_one_and_five_and_is_stable_after_save_load` passes.
- [C3-SEP] `cargo test --workspace` passes, including mode-separation integration tests in `crates/run_mode` and `crates/rpg_mode`.
- [C3-SEP] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [C3-SEP] `cargo run -p content_linter` passes with 58 unique IDs.
- [C3-INTEGRATE] Final C3 gate passes: formatting, workspace tests, strict clippy, default game smoke, all golden replay gates, content linter, board validator, and feature-built Bevy feel-test smoke.
- [C3-INTEGRATE] `cargo run -p content_linter` passes with 60 unique IDs after adding minimal RPG Chapter 1 gear content (`gear/rpg_ch1/resonance_charm`, `gear/rpg_ch1/luck_trinket`).
- [C3-INTEGRATE] `cargo run -p feverfall_game -- --smoke` prints C2 run summary hash `18202124e6b686d8`, RPG Chapter 1 hash `3364e243ba2065f4`, and roguelite 3-act summary hash `4dedb4fcdacb19b9`.
- [C3-INTEGRATE] `cargo run -p feverfall_game --features bevy_feel_test -- --smoke` passes and prints `c3_vfx_triggers=14` for blue/orange/purple/green peg hits, bucket catch, combo 3/6/10/15+, Long Shot, near-miss, last orange in flight, Extreme Fever, and board failure. `reduce_flash` suppresses scale pulse/bloom layers and `reduce_shake` suppresses camera shake cues.
- [C4-G] `cargo test --workspace` passes after reconciling concurrent Act 4/RPG/UI/VFX additions.
- [C4-G] `cargo run -p feverfall_game -- --smoke-full` passes. Hashes: C2 `18202124e6b686d8`, RPG Chapter 1 `3364e243ba2065f4`, RPG campaign `04029810211125c5`, roguelite Act 1-3 `e72374145338c3b3`, roguelite Acts 1-4 `152fc850303d8356`, feel-test `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`.
- [C4-G] `--smoke-full` verified all seven golden replay fixtures: minimal `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`, vertical slice `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`, Act 1 two-board `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`, RPG defensive `8e566217ee6cddee3be784b3e359b3eda5708638ac8540bce759086e922a145f`, RPG implementation `fc72b1144ad88e62bb27c3a1296cbb9b3fa51871a852b9b5ef561d7146033a58`, roguelite defensive `89c224a1ba8aae30965fa42f9547940036badc026b0a2f1bf50e6de15b86682b`, and roguelite implementation `c5db0fb8d90e57c8be159bbb779c56ead19148f36de8bdc077711e59f9a4a36a`.
- [C4-G] `--smoke-full` reported `content lint passed: 242 unique id(s)` and board validation PASS for all authored boards.
- [C4-CONTENT] `cargo run -p content_linter` passes with 242 unique IDs.
- [C4-CONTENT] `cargo run -p board_validator` passes all 80 authored boards.
- [C4-CONTENT] `cargo test -p content_schema -p board_gen -p content_linter` passes, including board generation/authored-board validation tests.
- [C4-CONTENT] Golden replay checks preserved hashes: default `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`, RPG Chapter 1 implementation `fc72b1144ad88e62bb27c3a1296cbb9b3fa51871a852b9b5ef561d7146033a58`, roguelite Act 1-3 implementation `c5db0fb8d90e57c8be159bbb779c56ead19148f36de8bdc077711e59f9a4a36a`.
- [C4-CONTENT] Content counts achieved: relics 60 total with Ball/Peg/Basket/Board/EconomyCombo 12 each; ball variants 20; RPG gear 40 with Launcher/CoreBall/BasketRig/Charm 10 each; RPG skills 36 with Trickshot/Basket/Alchemy/Tactician 9 each; authored boards 80; boss-tagged boards 13 using 12 structured mechanic kinds.
- [C4-RPG-CH2TO5] `cargo test -p rpg_mode` passes: 8 lib tests plus 2 mode-separation tests, including Chapter 2-5 catalog counts, Chapter 5 mastery hashes, campaign completion, and mastery unlock coverage.
- [C4-RPG-CH2TO5] `cargo run -p board_validator` passes all authored boards observed in the concurrent C4 content pack, including `boards/c4_rpg_ch1_*`, `boards/c4_rpg_ch2_*`, `boards/c4_rpg_ch3_*`, `boards/c4_rpg_ch4_*`, `boards/c4_rpg_ch5_*`, and existing `boards/rpg_ch1_*`.
- [C4-RPG-CH2TO5] `cargo run -p content_linter` passes with 242 unique IDs after concurrent C4 content landed.
- [C4-RPG-CH2TO5] `cargo run -p feverfall_game -- --smoke` passes and prints RPG campaign sample hashes: Ch1 board 1 `c18385eaa33af638`, Ch3 board 1 `01efbd0f270af2e8`, Ch5 board 1 `ef2ae2140c5abcdf`, RPG campaign summary `04029810211125c5`, and preserved Chapter 1 summary `3364e243ba2065f4`.
- [C4-RPG-CH2TO5] `cargo run -p replay_runner -- --replay tests/golden_replays/rpg_ch1_smoke.replay.json` preserves Chapter 1 replay hash `fc72b1144ad88e62bb27c3a1296cbb9b3fa51871a852b9b5ef561d7146033a58`.
- [C4-RPG-CH2TO5] `cargo fmt --all -- --check` passes, and `cargo test -p feverfall_game rpg_chapter1` passes the campaign smoke and preserved Chapter 1 smoke tests.
- [C4-VFX2] `cargo clippy -p feverfall_game --features bevy_feel_test -- -D warnings` passes.
- [C4-VFX2] `cargo run -p feverfall_game --features bevy_feel_test -- --smoke` passes and prints `c4_vfx_triggers=21` covering ball launch, blue/orange/purple/green peg hits, bucket catch, combo 3/6/10/15+, Long Shot, Lucky Bounce, near-miss, final orange in flight, Extreme Fever, five relic-category flashes, and board failure.
- [C4-VFX2] `cargo test -p feverfall_game` passes with 47 tests, including audio pitch/chord-cluster, combo rail reset, relic category color, board ambience, and accessibility coverage.
- [C4-ACT4] `cargo test -p run_mode` passes with Act 4 unlock, seeded-board, combined boss mechanic, and `Full Fever Cleared` meta-progression tests.
- [C4-ACT4] `cargo test -p feverfall_game roguelite_act4_smoke_preserves_three_act_base_and_records_mastery` passes and verifies the base smoke remains 3 acts while the full smoke reaches Act 4.
- [C4-ACT4] `cargo run -p content_linter` passes with 242 unique IDs; `cargo run -p board_validator` passes all authored boards observed in the concurrent C4 content pack.
- [C4-ACT4] `cargo run -p replay_runner -- --replay tests/golden_replays/roguelite_act1to3_smoke.replay.json` preserves `c5db0fb8d90e57c8be159bbb779c56ead19148f36de8bdc077711e59f9a4a36a`.
- [C4-ACT4] `cargo run -p feverfall_game -- --smoke` passes and prints roguelite Act 1-3 summary hash `e72374145338c3b3`, Act 4 derived seed `14462389677421375956`, and full Act 1-4 summary hash `152fc850303d8356`.
- [C4-ACT4] `cargo run -p feverfall_game -- --smoke-full` passes all included content, board, and replay gates with `smoke-full summary: PASS checks=12 replays=7`.

## Failing Validation

- None. Checkpoint 3 exit validation passed.
- None for [C4-CONTENT] validation.
- None for [C4-RPG-CH2TO5] validation. Commit is blocked by unrelated concurrent C4 worktree changes sharing tracked files and adding content directories; staging whole files would include other agents' work.
- None for [C4-ACT4] validation.

## Environment Notes

- The workspace now pins Rust 1.95.0 via `rust-toolchain.toml`; this unblocks Bevy 0.19 for the feature-gated playable scene.
- Playable feel-test command: `cargo run -p feverfall_game --features bevy_feel_test -- --feel-test`, or no args for feature-built binaries such as the Windows artifact. Use `--smoke` to force the non-interactive smoke path. Pressing Space draws a cyan shot trail and yellow final ball marker.
- [C3-BALANCE] `cargo run -p balance_sim --release` could not use repo `target/release` because `.cargo-lock` returned permission denied; the full simulation succeeded with `CARGO_TARGET_DIR=/tmp/opencode/feverfall-target`.
- [C4-RPG-CH2TO5] Concurrent C4 content/tool/runtime changes were present during validation. This agent avoided adding duplicate board JSON IDs; Chapter 2-5 board IDs live in the `rpg_mode` catalog as `boards/rpg_ch2_*`, `boards/rpg_ch3_*`, `boards/rpg_ch4_*`, and `boards/rpg_ch5_mastery_*` while the concurrent content pack uses `boards/c4_rpg_*`.
- [C4-CONTENT] New authored content uses `boards/c4_*`, `balls/c4/*`, `gear/c4/*`, and `skills/c4/*` IDs to avoid collisions with [C4-RPG-CH2TO5] and future [C4-ACT4] additions.
- [C4-ACT4] `MetaProgressionSave` now serializes `mastery_records`; no migration/backward-compatibility shim was added because there is no shipped persisted roguelite meta save requirement yet.
- [C4-ACT4] Act 4 final boss mechanic contract is `boss_mechanics/act4/final_seed_row_tempo`, combining `ScriptedObstacleRow` and `BucketTempoShift` for [C4-CONTENT] compatibility.

## Blockers

- None for [C3-BALANCE].
- [C3-G] Prior test/clippy blockers were resolved during [C3-SEP]. Also observed `cargo run -p feverfall_game -- --smoke` now emits run summary hash `600628ae0877f49d` rather than documented Checkpoint 2 hash `0b36add9e9b3283c`, likely from concurrent C3 runtime changes.
- None for [C3-ROGUELITE] artifacts/validation. Commit not created because concurrent uncommitted C3 changes share files touched by this agent; staging whole files would include other agents' work.
- None for [C3-RPG].
- None for [C3-SEP] mode separation. Commit ownership is shared with concurrent C3 changes across several files.
- None for [C3-INTEGRATE].
- [C4-G] GitHub workflow dispatch was available and queued run `28720908116`, but it used the remote `main` workflow before these local C4 changes were pushed. Rerun `.github/workflows/windows-feel-test.yml` from a ref containing this change to produce the Checkpoint 4 Windows artifact with `--smoke-full` verified.
- None for [C4-CONTENT] implementation or validation. Commit may need coordination because concurrent C4 changes touched shared tracked files and observed the new content directories.
- None for [C4-RPG-CH2TO5] implementation or validation. Commit deferred because the worktree contains unrelated concurrent C4 changes in shared files and untracked content directories.
- None for [C4-ACT4] implementation or validation. Commit decision pending git inspection because concurrent C4 agents have changed shared tracked files and content directories.

## Decisions Needed From Human

- None for Checkpoint 2. Human approval recorded: interactive flow confirmed.

## Last Replay Hash

- `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`

## Last Vertical Slice Replay Hash

- `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`

## Last Act 1 Two-Board Replay Hash

- `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`

## Last RPG Chapter 1 Smoke Replay Hash

- `8e566217ee6cddee3be784b3e359b3eda5708638ac8540bce759086e922a145f`
- Implementation fixture `tests/golden_replays/rpg_ch1_smoke.replay.json`: `fc72b1144ad88e62bb27c3a1296cbb9b3fa51871a852b9b5ef561d7146033a58`

## Last RPG Chapter 1 Smoke Summary Hash

- `3364e243ba2065f4`

## Last RPG Campaign Smoke Hashes

- Ch1 board 1: `c18385eaa33af638`
- Ch3 board 1: `01efbd0f270af2e8`
- Ch5 board 1: `ef2ae2140c5abcdf`
- Campaign summary: `04029810211125c5`

## Last Roguelite 3-Act Smoke Replay Hash

- `89c224a1ba8aae30965fa42f9547940036badc026b0a2f1bf50e6de15b86682b`

## Last Roguelite Act 1-3 Smoke Replay Hash

- `c5db0fb8d90e57c8be159bbb779c56ead19148f36de8bdc077711e59f9a4a36a`

## Last Smoke Run Summary Hash

- `18202124e6b686d8`

## Last Roguelite Act 1-3 Smoke Summary Hash

- `e72374145338c3b3`

## Last Roguelite Act 4 Derived Seed

- `14462389677421375956`

## Last Feel-Test Smoke Hash

- `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`

## Last Full Smoke Hashes

- C2 run summary: `18202124e6b686d8`
- RPG Chapter 1 summary: `3364e243ba2065f4`
- RPG campaign summary: `04029810211125c5`
- Roguelite Act 1-3 summary: `e72374145338c3b3`
- Roguelite Acts 1-4 full-run summary: `152fc850303d8356`
- Feel-test replay: `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`

## Next Integration Target

- Continue Checkpoint 4 integration; rerun Windows workflow from a pushed C4 ref.

## Next Parallel Dispatch

- Integrate remaining C4 agent outputs against `--smoke-full`; rerun `cargo run -p feverfall_game -- --smoke-full` after content/mode changes and update golden/docs only when hashes intentionally change.
