# Checkpoint 6 Resumption Prompt — FeverFall Release Readiness

## Project Summary

FeverFall is a Peggle spiritual successor with roguelite meta-progression and RPG mode, built in Rust 1.95.0 / Bevy 0.19 as a Cargo workspace with 8 crates (`physics_core`, `board_gen`, `game_rules`, `run_mode`, `rpg_mode`, `feedback_events`, `content_schema`, `telemetry`) plus the main `feverfall_game` binary. All game logic is deterministic and replay-verified.

## Current State

C5 complete, alpha candidate. 246 content IDs, 80 boards, 13 rendered screens, 5 audio buses, versioned save (v0→v1), fuzz harnesses, balance TOML tables, GitHub release workflows for Windows + macOS.

Known C5-to-C6 gaps include unverified remote release dry-run artifacts, no long fuzz campaigns, no 10,000+ balance validation campaign, open human playtest telemetry benchmarks, unimplemented code signing/notarization, unimplemented WASM build, local Ch5 leaderboard persistence stub/hash-only behavior, missing local `actionlint`, and macOS CI artifacts not yet verified from a pushed ref.

## What C6 Is Solving

Checkpoint 6 solves the gap between alpha candidate and public alpha release: WASM web build, signing/notarization guidance, long balance campaign validation, local leaderboard persistence, extended fuzz campaign, CI hardening, and human playtest infrastructure.

## Full Parallel Dispatch Order

Round 1 (Group A — all fire simultaneously):
C6-BALANCE-LONG, C6-FUZZ-CAMPAIGN, C6-LEADERBOARD, C6-CI-HARDEN, C6-PLAYTEST

Round 2 (Group B — fire after Group A complete):
C6-WASM, C6-SIGN

Round 3 (Group C — fires after Group B complete):
C6-INTEGRATE

## Full Subagent Definitions

[C6-WASM] Web Build & itch.io Deploy

Owns: `game/Cargo.toml` (wasm feature additions only), `.github/workflows/wasm-deploy.yml` (new file), `game/web/` (new dir: `index.html`, favicon, canvas CSS), `README.md` web play section, `docs/release.md` WASM section, `CHANGELOG.md`.

Does NOT touch: any crate under `crates/`, any existing workflow files, `game/src/`.

Entry criteria: Group A complete; R-WASM and RP-WASM handoffs available.

Exit criteria: `cargo build -p feverfall_game --target wasm32-unknown-unknown --features wasm` succeeds; `trunk build` produces a valid `dist/` directory; GitHub Actions job uploads ZIP artifact; smoke test confirms game canvas loads in a headless browser or documented equivalent compile/load gate; `cargo test --workspace` still passes; docs updated.

Tests to write: CI WASM build smoke (compile-only gate acceptable if browser smoke is not stable locally).

Docs to update: `README.md` (web play section), `docs/release.md` (WASM section), `CHANGELOG.md`.

Implementation notes: use WebGL2 as default web renderer; avoid `x11`, `wayland`, and `multi_threaded` in wasm features; enable Bevy web/audio/WAV features only as needed; use Trunk with relative asset paths and `index.html` at ZIP root for itch.io.

[C6-SIGN] Alpha Code Signing

Owns: release workflow signing step additions only, `docs/release.md` signing section, `.github/SECURITY.md` (new), `CHANGELOG.md`.

Does NOT touch: any source files, `Cargo.toml`, game assets.

Entry criteria: Group A complete; R-CI and RP-SIGN handoffs available.

Exit criteria: Windows artifact warning/checksum guidance documented; macOS `.app` notarization path or deferred decision documented with rationale; required Apple/Windows secrets listed; `docs/release.md` updated; `.github/SECURITY.md` created with vulnerability reporting process.

Tests to write: none (documentation + workflow only).

Docs to update: `docs/release.md`, `.github/SECURITY.md`, `CHANGELOG.md`.

Implementation notes: do not publish self-signed Windows artifacts as trusted; use unsigned Windows alpha plus checksums unless real signing credentials exist; macOS notarization needs Apple Developer Program, Developer ID cert, temporary keychain, `codesign`, `notarytool`, and stapling.

[C6-BALANCE-LONG] Long Balance Validation Campaign

Owns: `tools/balance_sim/src/` (cohort count, reporting, and parallelism only), `docs/design/balance_notes.md`, `content/balance/roguelite/`, `content/balance/rpg/`, `CHANGELOG.md`.

Does NOT touch: any `crates/` source, `game/src/`, board JSON files.

Entry criteria: Group A start; R-STATE, R-FUZZ, and RP-BALANCE-LONG handoffs available.

Exit criteria: 10,000-run roguelite simulation completes and reports Act 1 win rate >= 25% and Act 3 win rate >= 5%; RPG cohort sim completes 1,000 runs per cohort and reports chapter clear rates; `balance_notes.md` updated with new results; `cargo test --workspace` still passes; if win rates are below target, TOML tables are updated and sim is rerun until targets are met or the miss is escalated as a blocker.

Tests to write: balance_sim smoke test asserting nonzero win rates above a minimum threshold.

Docs to update: `docs/design/balance_notes.md`, `CHANGELOG.md`.

Implementation notes: statistical minimum for ±2% at 95% CI near 30% win rate is 2,017 runs; 10,000 runs gives roughly ±0.9%. Current sim appears single-threaded, so measure runtime before adding nightly/deep CI.

[C6-LEADERBOARD] Local Ch5 Leaderboard Persistence

Owns: `crates/rpg_mode/src/leaderboard.rs` (new file), minimal leaderboard wiring needed to append Ch5 mastery scores, `docs/technical/shared_contracts.md`, `CHANGELOG.md`.

Does NOT touch: `crates/run_mode/`, `crates/physics_core/`, broad game screen rewrites, `game/assets/` leaderboard files.

Entry criteria: Group A start; R-LEADERBOARD and RP-LEADERBOARD handoffs available.

Exit criteria: Ch5 mastery completion writes a JSONL entry to `{data_dir}/feverfall/leaderboard.jsonl` or the repo's accepted local save convention if data-dir support is unavailable; `cargo test -p rpg_mode` passes including leaderboard write/read round-trip test; smoke still passes; docs updated.

Tests to write: `leaderboard_write_read_roundtrip`, `leaderboard_appends_without_overwrite`, `leaderboard_handles_missing_file_gracefully`.

Docs to update: `docs/technical/shared_contracts.md`, `CHANGELOG.md`.

Implementation notes: prefer existing `serde_json` and `std::fs::OpenOptions`; use append-only JSONL records keyed by board ID and `leaderboard_hash`; keep remote/online leaderboard out of alpha scope.

[C6-FUZZ-CAMPAIGN] Extended Fuzz Campaign & Corpus

Owns: `fuzz/corpus/` (new seed inputs), `fuzz/artifacts/` (crash reproductions if any), `docs/qa/fuzz_campaign_results.md` (new file), `docs/qa/determinism_checklist.md`, `CHANGELOG.md`.

Does NOT touch: `fuzz/fuzz_targets/` source except read-only; any `crates/` source unless a crash requires a separate named fix.

Entry criteria: Group A start; R-FUZZ handoff available.

Exit criteria: each fuzz target has been run for >= 60 seconds locally with no unresolved crashes; corpus seed files committed; `docs/qa/fuzz_campaign_results.md` written with target names, run durations, crash count, and corpus sizes; if any crash is found, a regression test is added to `physics_core` before this subagent exits.

Tests to write: any regression tests for crashes found; none expected if zero crashes.

Docs to update: `docs/qa/fuzz_campaign_results.md`, `docs/qa/determinism_checklist.md`, `CHANGELOG.md`.

Implementation notes: current targets are `first_bounce_consistency`, `trajectory_sampling`, `multi_shot_replay`, and `authored_board_corpus`; estimated campaign time is about four minutes plus build time.

[C6-CI-HARDEN] CI Hardening & Dependabot

Owns: `.github/workflows/ci.yml` (actionlint job addition and `permissions: contents: read` only), `.github/workflows/windows-feel-test.yml` (`permissions: contents: read` only), `.github/dependabot.yml` (new file), `CONTRIBUTING.md`, `CHANGELOG.md`.

Does NOT touch: `game/`, `crates/`, `tools/`, `content/`, non-workflow files except docs listed above.

Entry criteria: Group A start; R-CI and RP-ACTIONLINT handoffs available.

Exit criteria: `actionlint` runs as a CI step on push/PR and passes on existing workflow files; Dependabot adds weekly Cargo ecosystem checks; least-privilege workflow permissions are explicit; `cargo test --workspace` still passes; docs updated.

Tests to write: none (CI config only).

Docs to update: `CONTRIBUTING.md` (CI/dependency update notes), `CHANGELOG.md`.

Implementation notes: use `rhysd/actionlint@v1` unless policy requires SHA pinning; `GITHUB_TOKEN` is sufficient for same-repo releases when publish job has `contents: write`.

[C6-PLAYTEST] Playtest Infrastructure & Session Script

Owns: `docs/playtesting/` (all files), `.github/ISSUE_TEMPLATE/playtest_feedback.md` (new), optional `tools/playtest_survey/` only if explicitly chosen, `CHANGELOG.md`.

Does NOT touch: any game source, `crates/`, `game/src/`.

Entry criteria: Group A start; RP-PLAYTEST handoff available.

Exit criteria: `docs/playtesting/session_script.md` written with 15-minute structured tester session; `docs/playtesting/feel_survey.md` updated or replaced; playtest feedback GitHub issue template created; if a `tools/playtest_survey` CLI is created, it compiles and `cargo test --workspace` passes; docs updated.

Tests to write: if `playtest_survey` CLI is created, add a basic CLI smoke test.

Docs to update: `docs/playtesting/session_script.md`, `docs/playtesting/feel_survey.md`, `CHANGELOG.md`.

Implementation notes: for under 20 testers, prefer Google Form plus manual local JSONL/replay attachment; do not build telemetry upload infrastructure for alpha unless explicitly requested.

[C6-INTEGRATE] Checkpoint 6 Integration & Validation

Owns: `ORCHESTRATOR_STATE.md`, `docs/devplan/checkpoint_status.md`, `docs/qa/pre_release_report.md`.

Does NOT touch: any file owned by a concurrent Group A/B subagent.

Entry criteria: ALL Group A and Group B subagents complete.

Exit criteria:

`cargo fmt --all -- --check` passes.

`cargo clippy --workspace --all-targets -- -D warnings` passes.

`cargo test --workspace` passes.

`cargo run -p feverfall_game -- --smoke-full` passes.

`cargo run -p content_linter` passes with >= 246 unique IDs.

`cargo run -p board_validator` passes all 80 boards.

All golden replays pass.

`ORCHESTRATOR_STATE.md` updated with C6 status `CHECKPOINT 6 COMPLETE / PUBLIC ALPHA READY`.

`docs/qa/pre_release_report.md` updated: all C6 items marked closed or explicitly deferred to post-launch.

Tests to write: none (validation only).

Docs to update: `ORCHESTRATOR_STATE.md`, `docs/devplan/checkpoint_status.md`, `docs/qa/pre_release_report.md`.

## Full Exit Criteria

All validation commands below pass locally:

`cargo fmt --all -- --check`

`cargo clippy --workspace --all-targets -- -D warnings`

`cargo test --workspace`

`cargo run -p feverfall_game -- --smoke-full`

`cargo run -p content_linter` with >= 246 unique IDs

`cargo run -p board_validator` with all 80 authored boards passing

`for replay in tests/golden_replays/*.json; do cargo run -p replay_runner -- --replay "$replay" || exit 1; done`

WASM build compiles and produces a `dist/` artifact.

Signing guidance is documented, even if full notarization/signing is deferred.

Roguelite Act 1 win rate is >= 25% in a 10,000-run headless sim.

Roguelite Act 3 win rate is >= 5% in the same validation campaign.

RPG cohorts complete 1,000 runs per cohort and publish clear-rate metrics.

All fuzz targets run >= 60 seconds with zero unresolved crashes.

Local Ch5 leaderboard writes and reads correctly.

`actionlint` passes in CI.

Playtest session script is written and issue template is created.

`docs/qa/pre_release_report.md` has no unmarked open C6 items.

`ORCHESTRATOR_STATE.md` status: `CHECKPOINT 6 COMPLETE / PUBLIC ALPHA READY`.

## Full Local Validation Gate

`cargo fmt --all -- --check`

`cargo clippy --workspace --all-targets -- -D warnings`

`cargo test --workspace`

`cargo run -p feverfall_game -- --smoke-full`

`cargo run -p content_linter` (currently 246 unique IDs — must not regress)

`cargo run -p board_validator` (80 authored boards — must not regress)

`for replay in tests/golden_replays/*.json; do cargo run -p replay_runner -- --replay "$replay" || exit 1; done`

## Resumption Instruction

When a user says 'resume C6' or 'use the C6 prompt', read `ORCHESTRATOR_STATE.md` first. If `## Checkpoint 6` section is absent or incomplete, begin dispatching from Round 1. If Round 1 is marked complete, begin Round 2. If Round 2 is marked complete, dispatch C6-INTEGRATE. Never re-dispatch a subagent whose exit criteria are already marked COMPLETE in `ORCHESTRATOR_STATE.md`.
