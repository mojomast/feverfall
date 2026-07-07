# Checkpoint 5 Devplan — FeverFall

## Overview

Checkpoint 5 turns FeverFall from a feature-complete deterministic prototype into a playable alpha candidate with real presentation, versioned persistence, measurable balance, and release-grade project hygiene. Checkpoint 4 is complete: core crates, C4 content, RPG Chapters 2-5, reactive feedback contracts, placeholder screen models, smoke gates, content validation, board validation, replay validation, strict clippy, and workspace tests are all green. C5 addresses the explicitly open gaps: real Bevy rendering/audio, save migration safety, fuzz/property coverage, cross-platform release automation, balance cohort metrics, and public-facing repository documentation.

The push is deliberately parallelized by file ownership. Independent systems work can proceed first, rendering/audio work is staged to avoid `game/Cargo.toml` and runtime plugin conflicts, and final integration runs last to reconcile cargo features, docs links, release gates, and full validation.

## Parallel Workstream Groups

### Group A — Can run fully in parallel (no shared file ownership)

- [✅] [C5-SAVE-MIGRATION] Save System Version Shim
  - Files owned: `crates/run_mode/src/lib.rs`, `crates/run_mode/Cargo.toml`, `docs/technical/save_migration.md`
  - Entry criteria: C5 devplan accepted
  - Exit criteria: `MetaProgressionSave` has versioned load/save shim, v0 migration, unknown-version rejection, tests, and docs

- [✅] [C5-FUZZ] Physics Fuzz Harness
  - Files owned: `fuzz/`, `crates/physics_core/Cargo.toml`, `crates/physics_core/src/` test modules, `docs/technical/fuzzing.md`
  - Entry criteria: C5 devplan accepted
  - Exit criteria: isolated `cargo-fuzz` workspace, bounded physics harnesses, proptest invariants, docs, normal workspace tests unaffected

- [✅] [C5-RPG-BALANCE] RPG Balance Simulation
  - Files owned: `content/balance/rpg/`, `tools/balance_sim/src/rpg.rs`, `tools/balance_sim/src/metrics.rs`, `docs/design/rpg_balance_notes.md`
  - Entry criteria: C5 devplan accepted
  - Exit criteria: RPG balance tables exist, RPG cohort metrics emit deterministic JSON, tests/docs updated

- [✅] [C5-ROGUELITE-REBALANCE] Roguelite Win Rate Tuning
  - Files owned: `content/balance/roguelite/`, `tools/balance_sim/src/roguelite.rs`, `docs/design/balance_notes.md`
  - Entry criteria: C5 devplan accepted
  - Exit criteria: sim consumes roguelite TOML and runtime APIs, separates random/heuristic cohorts, target ranges documented, tests/docs updated

- [✅] [C5-CI] CI Expansion
  - Files owned: `.github/workflows/`, `.github/PULL_REQUEST_TEMPLATE.md`, `.github/ISSUE_TEMPLATE/`, `docs/release.md`, `cliff.toml`
  - Entry criteria: C5 devplan accepted
  - Exit criteria: macOS release builds, tagged release publishing, coverage job, changelog workflow docs, and CI docs updated

- [✅] [C5-REPO-HEALTH] Repository Health & Documentation
  - Files owned: `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `docs/agent/`, relocated agent scaffolding docs
  - Entry criteria: C5 devplan accepted
  - Exit criteria: public repo entry path is clear, contributing/release/changelog docs exist, agent scaffolding is relocated, docs-only validation complete

### Group B — Depends on Group A outputs only

- [✅] [C5-RENDER] Real Bevy Rendering Pass
  - Files owned: `game/src/plugins/screens/`, `game/assets/sprites/`, `game/assets/fonts/`, render-only edits in `game/Cargo.toml`, `docs/technical/rendering.md`
  - Entry criteria: Group A complete; repo-health docs paths settled; no concurrent audio edits to `game/Cargo.toml`
  - Exit criteria: all 13 screen models have real Bevy ECS systems, visible window opens, sprites/fonts load, smoke/clippy/tests pass

- [✅] [C5-AUDIO] Real Audio Integration
  - Files owned: `game/src/plugins/audio/`, `game/assets/audio/`, `game/assets/audio/ATTRIBUTION.md`, audio-only edits in `game/Cargo.toml`, `docs/technical/audio_contracts.md`
  - Entry criteria: Group A complete; C5-RENDER has landed or explicitly released `game/Cargo.toml`
  - Exit criteria: 5 audio buses use real runtime adapter, CC0 assets or square-wave beep fallback registered, accessibility behavior preserved, tests/docs pass

### Group C — Integration & Validation (sequential, after B)

- [✅] [C5-INTEGRATE] Checkpoint 5 Integration & Validation
  - Files owned: cross-cutting reconciliation only in `Cargo.toml`, `game/Cargo.toml`, `tools/balance_sim/src/main.rs`, `ORCHESTRATOR_STATE.md`, `docs/qa/pre_release_report.md`
  - Entry criteria: Groups A and B complete
  - Exit criteria: all C5 gates pass, ownership conflicts resolved, state/docs updated, deferred C6 gaps recorded

## Subagent Definitions

### [C5-RENDER] Real Bevy Rendering Pass

Owns: `game/src/plugins/screens/`, `game/assets/sprites/`, `game/assets/fonts/`, render-only edits in `game/Cargo.toml`, `docs/technical/rendering.md`

Does not touch: any crate under `crates/`, `physics_core`, `run_mode`, `rpg_mode`, `game/src/plugins/audio/`, `game/assets/audio/`, CI workflows

Entry criteria: Group A complete; C5-AUDIO is not editing `game/Cargo.toml`

Exit criteria: All 13 screen models have real Bevy ECS systems; `cargo run -p feverfall_game --features bevy_feel_test` opens a visible window; SVG-derived sprites load; smoke and clippy pass

Asset strategy: add `bevy_ui`, `bevy_state`, explicit `bevy_asset`/`bevy_image` as needed; bake simple geometric SVGs to PNGs with a tool using `resvg`/`usvg`/`tiny-skia`; store source SVGs outside runtime assets and generated PNGs under `game/assets/sprites/`; use Kenney CC0 sprites/UI, OpenGameArt only when individual asset pages are CC0, and Google Fonts OFL with license files preserved

Screens to render: main menu, settings, board/HUD, reward choice, node map, shop, forge, event, relic bar, run summary/failure, RPG chapter select, RPG gear, RPG skill tree, campaign progress

Tests to write: `screen_state_spawns_one_root`, `screen_exit_despawns_screen_entities`, `reward_screen_spawns_three_cards`, `node_map_screen_spawns_reachable_nodes`, `rpg_skill_tree_spawns_skill_nodes`, `render_asset_manifest_paths_exist`, `font_license_files_exist`

Docs to update: `docs/technical/rendering.md`, `README.md` run instructions if commands change

### [C5-AUDIO] Real Audio Integration

Owns: `game/src/plugins/audio/`, `game/assets/audio/`, `game/assets/audio/ATTRIBUTION.md`, audio-only edits in `game/Cargo.toml`, `docs/technical/audio_contracts.md`

Does not touch: `game/src/plugins/screens/`, `game/assets/sprites/`, `game/assets/fonts/`, physics crates, balance tables, CI workflows

Entry criteria: Group A complete; C5-RENDER has landed or released `game/Cargo.toml`

Exit criteria: all 5 audio bus contracts have runtime playback mapping; every `FeedbackKind` maps to a cue or intentional silence gate; CC0 provenance is recorded; high-frequency accessibility mute and ethical intensity caps remain tested

Recommended crate: `bevy_kira_audio` compatible with Bevy 0.19, without enabling conflicting `bevy_audio`/`vorbis` features. If dependency risk is unacceptable, built-in `bevy_audio` may be used for one-shots, but bus support must be modeled explicitly.

Asset strategy: prefer Kenney CC0 packs for UI, impact, casino/plink, and digital cues; use OpenGameArt only for individually verified CC0 assets; use Freesound only with explicit Creative Commons 0 pages; fallback synthesis is limited to short square-wave beeps

Audio bus implementation: map `CollisionPlinks`, `ComboPercussion`, `RewardStingers`, `AmbientBoardHum`, and `UiConfirmations` to dedicated Kira channels with volume/rate limiting; route `FeedbackEvent` stream into audio without mutating deterministic gameplay state

Tests to write: `all_feedback_kinds_have_audio_mapping`, `all_audio_buses_have_runtime_channel`, `loss_and_near_miss_do_not_use_victory_layers`, `high_frequency_mute_omits_plinks`, `extreme_fever_silence_gate_precedes_sting`, `audio_asset_manifest_paths_exist`

Docs to update: `docs/technical/audio_contracts.md`, `game/assets/audio/ATTRIBUTION.md`

### [C5-SAVE-MIGRATION] Save System Version Shim

Owns: `crates/run_mode/src/lib.rs`, `crates/run_mode/Cargo.toml`, `docs/technical/save_migration.md`

Does not touch: `crates/rpg_mode/`, game UI/audio files, balance sim, CI workflows

Entry criteria: Group A start

Exit criteria: `MetaProgressionSave` has current version constant, versioned wire structs, legacy v0 migration for missing `version`, current v1 save output, unknown-version rejection, tests, docs

Recommended pattern: manual serde shim matching the existing RPG `CampaignSave` style; keep public runtime type clean; serialize/deserialize through private v0/v1 wire structs; treat missing `version` as legacy v0; reject versions greater than current; use `#[serde(default)]` only for known legacy-missing fields

Tests to write: `legacy_meta_save_without_version_loads`, `legacy_meta_save_missing_starter_balls_defaults_basic_orb`, `current_meta_save_writes_version_one`, `current_meta_save_round_trips`, `unknown_meta_save_version_errors`, `missing_optional_meta_arrays_default_empty`

Docs to update: `docs/technical/save_migration.md`

### [C5-FUZZ] Physics Fuzz Harness

Owns: `fuzz/`, `crates/physics_core/Cargo.toml`, `crates/physics_core/src/` test modules, `docs/technical/fuzzing.md`

Does not touch: gameplay mode crates, game UI/audio, balance tables, release workflows except docs links

Entry criteria: Group A start

Exit criteria: `cargo-fuzz` workspace is isolated from root workspace tests; `proptest` invariants run under `cargo test`; fuzz harnesses build on nightly; docs explain run/minimize/reproduce workflow

Recommended tools: `cargo-fuzz`, `libfuzzer-sys`, `arbitrary` with derive, `proptest`; keep fuzz workspace outside root `[workspace].members` to avoid disrupting normal `cargo test --workspace`

Harness design: bounded valid-domain physics inputs, generated boards with finite dimensions and capped pegs/obstacles, authored board corpus target, first-bounce consistency target, trajectory sampling target, multi-shot replay sequence target

Tests to write: `proptest_simulation_is_deterministic`, `proptest_shot_terminates_with_final_event`, `proptest_no_duplicate_pegs_hit`, `proptest_trajectory_samples_are_finite`, `proptest_first_bounce_matches_first_collision`

Docs to update: `docs/technical/fuzzing.md`

### [C5-RPG-BALANCE] RPG Balance Simulation

Owns: `content/balance/rpg/`, `tools/balance_sim/src/rpg.rs`, `tools/balance_sim/src/metrics.rs`, `docs/design/rpg_balance_notes.md`

Does not touch: `content/balance/roguelite/`, `tools/balance_sim/src/roguelite.rs`, rendering/audio files, save migration code

Entry criteria: Group A start

Exit criteria: RPG balance TOML tables exist; RPG sim reports deterministic JSON metrics for on-curve, underleveled, overleveled, no-skill, skill-aware, gear archetype, and normalized mastery cohorts; tests/docs pass

Metrics: board clear rate, objective-tier rate, retries to clear, XP/level curve, gear acquisition timing, skill unlock/use rate, stat dominance, chapter completion, mastery normalized clear, content coverage

Tests to write: `rpg_balance_tables_parse`, `rpg_sim_seed_is_deterministic`, `rpg_metrics_schema_is_stable`, `rpg_sim_covers_all_c4_gear_and_skills`, `rpg_stat_dominance_metric_flags_outliers`

Docs to update: `docs/design/rpg_balance_notes.md`, `docs/qa/pre_release_report.md` via integration

### [C5-ROGUELITE-REBALANCE] Roguelite Win Rate Tuning

Owns: `content/balance/roguelite/`, `tools/balance_sim/src/roguelite.rs`, `docs/design/balance_notes.md`

Does not touch: `content/balance/rpg/`, `tools/balance_sim/src/rpg.rs`, rendering/audio files, save migration code

Entry criteria: Group A start

Exit criteria: sim consumes roguelite TOML instead of hardcoded values; runtime run/reward APIs are used where practical; random, center-bias, orange-targeting, bucket-aware cohorts are separated; target ranges are documented

Target ranges: random bot 0-2% full-run wins; basic heuristic no-meta 10-20% full-run wins; basic heuristic Act 1 clear floor 25-35%; new-human target 45-65% Act 1 completion measured by playtest, not random sim; experienced base difficulty 40-60% full-run wins

Tests to write: `roguelite_balance_tables_parse`, `roguelite_sim_seed_is_deterministic`, `roguelite_metrics_separate_act_survival_and_perfect_clear`, `roguelite_sim_uses_starting_balls_from_toml`, `roguelite_smoke_sim_emits_json`

Docs to update: `docs/design/balance_notes.md`, `docs/qa/pre_release_report.md` via integration

### [C5-CI] CI Expansion

Owns: `.github/workflows/`, `.github/PULL_REQUEST_TEMPLATE.md`, `.github/ISSUE_TEMPLATE/`, `docs/release.md`, `cliff.toml`

Does not touch: game/runtime source, crates, balance tables, assets

Entry criteria: Group A start

Exit criteria: Ubuntu CI remains stable; macOS arm64/x86_64 release builds exist; tagged release workflow attaches Windows/macOS artifacts and checksums; coverage workflow emits `lcov.info`; changelog/release docs exist

Recommended workflows: keep PR CI Ubuntu-focused; run macOS/Windows release builds on tags and manual dispatch; use `macos-15` and `macos-15-intel`; use final publish job with `contents: write`; use `cargo-llvm-cov` for baseline coverage; use `git-cliff` for release notes once commit hygiene is adequate

Tests to write: workflow syntax validation via `actionlint` if available, documented dry-run/manual dispatch instructions, CI command smoke in docs

Docs to update: `docs/release.md`, README links through repo-health or integration, PR/issue templates

### [C5-REPO-HEALTH] Repository Health & Documentation

Owns: `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `docs/agent/` relocated scaffolding

Does not touch: any source files

Entry criteria: Group A start

Exit criteria: README explains project status, play path, dev quick start, workspace layout, release process, known gaps, and license status; contributing guide and changelog exist; root agent scaffolding is relocated to `docs/agent/`; docs links updated

Tests to write: none (documentation only)

Docs to update: `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `docs/agent/README.md`, relocated agent docs

### [C5-INTEGRATE] Checkpoint 5 Integration & Validation

Owns: cross-cutting reconciliation in `Cargo.toml`, `game/Cargo.toml`, `tools/balance_sim/src/main.rs`, `ORCHESTRATOR_STATE.md`, `docs/qa/pre_release_report.md`

Does not touch: feature implementation files except to resolve explicit integration conflicts from completed agents

Entry criteria: Groups A and B complete

Exit criteria: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo run -p feverfall_game -- --smoke-full`, content linter, board validator, replay runner, balance sim smoke, and release workflow dry-run docs are green or explicitly documented if platform-only

Tests to write: integration smoke coverage only if gaps are found; otherwise run existing gates and require failed-gate fixes from owning agents

Docs to update: `ORCHESTRATOR_STATE.md`, `docs/qa/pre_release_report.md`, release/readme links if needed

## File Ownership Matrix

| Subagent ID | Files/Directories Owned | Explicitly Forbidden Paths |
|---|---|---|
| C5-RENDER | `game/src/plugins/screens/`, `game/assets/sprites/`, `game/assets/fonts/`, render-only `game/Cargo.toml`, `docs/technical/rendering.md` | `crates/`, `game/src/plugins/audio/`, `game/assets/audio/`, `.github/workflows/` |
| C5-AUDIO | `game/src/plugins/audio/`, `game/assets/audio/`, audio-only `game/Cargo.toml`, `docs/technical/audio_contracts.md` | `game/src/plugins/screens/`, `game/assets/sprites/`, `game/assets/fonts/`, `crates/physics_core/`, balance files |
| C5-SAVE-MIGRATION | `crates/run_mode/src/lib.rs`, `crates/run_mode/Cargo.toml`, `docs/technical/save_migration.md` | `crates/rpg_mode/`, `game/`, `tools/balance_sim/` |
| C5-FUZZ | `fuzz/`, `crates/physics_core/Cargo.toml`, `crates/physics_core/src/` tests, `docs/technical/fuzzing.md` | mode crates, game assets/plugins, CI workflows |
| C5-RPG-BALANCE | `content/balance/rpg/`, `tools/balance_sim/src/rpg.rs`, `tools/balance_sim/src/metrics.rs`, `docs/design/rpg_balance_notes.md` | `content/balance/roguelite/`, `tools/balance_sim/src/roguelite.rs`, game UI/audio |
| C5-ROGUELITE-REBALANCE | `content/balance/roguelite/`, `tools/balance_sim/src/roguelite.rs`, `docs/design/balance_notes.md` | `content/balance/rpg/`, `tools/balance_sim/src/rpg.rs`, game UI/audio |
| C5-CI | `.github/workflows/`, `.github/PULL_REQUEST_TEMPLATE.md`, `.github/ISSUE_TEMPLATE/`, `docs/release.md`, `cliff.toml` | source crates, game assets/plugins, balance tables |
| C5-REPO-HEALTH | `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `docs/agent/` | any source files, workflows, assets |
| C5-INTEGRATE | `Cargo.toml`, `game/Cargo.toml`, `tools/balance_sim/src/main.rs`, `ORCHESTRATOR_STATE.md`, `docs/qa/pre_release_report.md` | feature implementation files unless resolving explicit integration conflict |

## Parallel Dispatch Order

1. Round 1: dispatch `C5-SAVE-MIGRATION`, `C5-FUZZ`, `C5-RPG-BALANCE`, `C5-ROGUELITE-REBALANCE`, `C5-CI`, and `C5-REPO-HEALTH` simultaneously.
2. Round 2a: dispatch `C5-RENDER` after Round 1 completes.
3. Round 2b: dispatch `C5-AUDIO` after Round 1 completes and after `C5-RENDER` releases `game/Cargo.toml`; it may run concurrently with late rendering asset polish only if file ownership remains disjoint.
4. Round 3: dispatch `C5-INTEGRATE` after all Round 1 and Round 2 agents are complete.

## Exit Criteria for Checkpoint 5

- All 13 production-placeholder screen models are replaced by real Bevy ECS rendering systems.
- `cargo run -p feverfall_game --features bevy_feel_test` opens a visible playable window.
- SVG-derived sprites/icons and OFL fonts load from tracked assets with license/provenance docs.
- Five audio bus contracts play through runtime channels and preserve accessibility/ethical-intensity rules.
- Every imported asset is confirmed CC0, MIT, or OFL; no unclear-license assets are present.
- `MetaProgressionSave` has versioned save/load behavior, legacy v0 migration, current v1 output, and unknown-version rejection.
- Physics fuzz harnesses build outside normal workspace tests and property tests run under `cargo test`.
- Roguelite sim consumes current balance TOML and reports random/heuristic cohort metrics separately.
- RPG balance tables and RPG cohort simulation metrics exist.
- CI has macOS release build coverage, GitHub release artifact publishing, baseline coverage reporting, and release/changelog documentation.
- Repository has updated README, CONTRIBUTING, CHANGELOG, and relocated agent scaffolding under `docs/agent/`.
- Required validation passes: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo run -p feverfall_game -- --smoke-full`, content linter, board validator, replay runner, and balance sim smoke.
- `ORCHESTRATOR_STATE.md` and `docs/qa/pre_release_report.md` reflect C5 completion status and remaining gaps.

## Known Gaps Deferred to C6

- Human feel/comprehension benchmark execution and analysis.
- Web build or wasm release automation.
- Code signing/notarization for Windows/macOS binaries.
- Full visual art direction beyond minimum alpha-quality sprites/icons/fonts.
- Advanced audio mixing, spatialization, music composition, and non-placeholder procedural synthesis.
- Long-running fuzz campaigns as mandatory PR gates.
- Coverage threshold enforcement; C5 only establishes a baseline.
- Large-batch balance cohorts in required CI; C5 keeps heavyweight runs manual or scheduled.
