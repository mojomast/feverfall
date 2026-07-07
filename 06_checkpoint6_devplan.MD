Checkpoint 6 Devplan — FeverFall Release Readiness

Overview

Checkpoint 6 closes the release-readiness gap between the current C5 alpha candidate and a public alpha. C5 local gates are green with 246 content IDs, 80 authored boards, deterministic replay baselines, rendered screens, modeled audio, save migration, fuzz harnesses, balance tables, and release workflows already present. C6 focuses on the remaining release blockers and high-risk unknowns: web/WASM distribution, alpha signing guidance, long validation campaigns, local Ch5 leaderboard persistence, CI hardening, extended fuzzing, and human playtest infrastructure.

C6 is organized to maximize parallel execution while preserving file ownership boundaries. Group A handles independent validation, docs, local persistence, CI hardening, and playtest infrastructure. Group B uses Group A/recon outputs for web deployment and signing guidance. Group C runs last and updates authoritative project state only after all implementation work and the full local gate pass.

Parallel Workstream Groups

Group A — Fully parallel (no shared file ownership)

| Agent ID | Title | Files Owned | Entry Criteria | Exit Criteria |
|---|---|---|---|---|
| C6-BALANCE-LONG | Long Balance Validation Campaign | `tools/balance_sim/src/`, `docs/design/balance_notes.md`, `content/balance/roguelite/`, `content/balance/rpg/` | C6 devplan accepted; R-STATE, R-FUZZ, RP-BALANCE-LONG handoffs available | 10,000-run roguelite validation complete; Act 1 win rate >= 25%; Act 3 win rate >= 5%; RPG 1,000-run-per-cohort validation complete; docs and tests updated |
| C6-FUZZ-CAMPAIGN | Extended Fuzz Campaign & Corpus | `fuzz/corpus/`, `fuzz/artifacts/`, `docs/qa/fuzz_campaign_results.md` | C6 devplan accepted; R-FUZZ handoff available | Every fuzz target run >= 60 seconds; zero unresolved crashes; corpus seeds committed; QA results documented |
| C6-LEADERBOARD | Local Ch5 Leaderboard Persistence | `crates/rpg_mode/src/leaderboard.rs`, Ch5 leaderboard wiring only, `docs/technical/shared_contracts.md` | C6 devplan accepted; R-LEADERBOARD and RP-LEADERBOARD handoffs available | Ch5 mastery completion appends JSONL to local data path; read/write tests pass; docs updated |
| C6-CI-HARDEN | CI Hardening & Dependabot | `.github/workflows/ci.yml`, `.github/dependabot.yml`, workflow permissions comments only, `CONTRIBUTING.md` | C6 devplan accepted; R-CI and RP-ACTIONLINT handoffs available | `actionlint` CI job added; weekly Cargo Dependabot configured; least-privilege permissions documented/applied; docs updated |
| C6-PLAYTEST | Playtest Infrastructure & Session Script | `docs/playtesting/`, `.github/ISSUE_TEMPLATE/playtest_feedback.md`, optional `tools/playtest_survey/` | C6 devplan accepted; RP-PLAYTEST handoff available | 15-minute session script written; survey expanded; playtest issue template created; optional CLI compiles if added |

Group B — Depends on Group A outputs only

| Agent ID | Title | Files Owned | Entry Criteria | Exit Criteria |
|---|---|---|---|---|
| C6-WASM | Web Build & itch.io Deploy | `game/Cargo.toml` wasm feature additions only, `.github/workflows/wasm-deploy.yml`, `game/web/`, README/docs release updates | Group A complete; R-WASM and RP-WASM handoffs available | WASM compile succeeds; Trunk produces `dist/`; HTML5 ZIP artifact uploads; headless browser/canvas smoke documented; docs updated |
| C6-SIGN | Alpha Code Signing | `.github/workflows/release.yml` signing additions only if present/created decision approved, docs signing section, `.github/SECURITY.md` | Group A complete; R-CI and RP-SIGN handoffs available | Alpha signing/notarization decision documented; macOS notarization path or deferral explicit; Windows tester warning/checksum guidance documented; SECURITY.md created |

Group C — Integration & Validation (sequential, runs last)

| Agent ID | Title | Files Owned | Entry Criteria | Exit Criteria |
|---|---|---|---|---|
| C6-INTEGRATE | Checkpoint 6 Integration & Validation | `ORCHESTRATOR_STATE.md`, `docs/devplan/checkpoint_status.md`, `docs/qa/pre_release_report.md` | All Group A and Group B subagents complete | Full local validation gate passes; C6 marked complete/public alpha ready; pre-release report has no unmarked open C6 items |

Subagent Definitions

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

File Ownership Matrix

| Agent ID | Files/Directories Owned | Explicitly Forbidden Paths |
|---|---|---|
| C6-BALANCE-LONG | `tools/balance_sim/src/`, `docs/design/balance_notes.md`, `content/balance/roguelite/`, `content/balance/rpg/`, `CHANGELOG.md` | `crates/`, `game/src/`, board JSON files |
| C6-FUZZ-CAMPAIGN | `fuzz/corpus/`, `fuzz/artifacts/`, `docs/qa/fuzz_campaign_results.md`, `docs/qa/determinism_checklist.md`, `CHANGELOG.md` | `fuzz/fuzz_targets/` source, `crates/` except separate crash fix |
| C6-LEADERBOARD | `crates/rpg_mode/src/leaderboard.rs`, minimal Ch5 leaderboard wiring, `docs/technical/shared_contracts.md`, `CHANGELOG.md` | `crates/run_mode/`, `crates/physics_core/`, broad `game/src/plugins/screens/`, `game/assets/` data files |
| C6-CI-HARDEN | `.github/workflows/ci.yml`, `.github/workflows/windows-feel-test.yml`, `.github/dependabot.yml`, `CONTRIBUTING.md`, `CHANGELOG.md` | `game/`, `crates/`, `tools/`, `content/`, unrelated workflow logic |
| C6-PLAYTEST | `docs/playtesting/`, `.github/ISSUE_TEMPLATE/playtest_feedback.md`, optional `tools/playtest_survey/`, `CHANGELOG.md` | game source, `crates/`, `game/src/` |
| C6-WASM | `game/Cargo.toml`, `.github/workflows/wasm-deploy.yml`, `game/web/`, `README.md`, `docs/release.md`, `CHANGELOG.md` | `crates/`, existing workflow files, `game/src/` |
| C6-SIGN | release workflow signing additions only, `docs/release.md`, `.github/SECURITY.md`, `CHANGELOG.md` | source files, `Cargo.toml`, game assets |
| C6-INTEGRATE | `ORCHESTRATOR_STATE.md`, `docs/devplan/checkpoint_status.md`, `docs/qa/pre_release_report.md` | files owned by active Group A/B subagents |

Parallel Dispatch Order

Round 1 (Group A — all fire simultaneously):
C6-BALANCE-LONG, C6-FUZZ-CAMPAIGN, C6-LEADERBOARD, C6-CI-HARDEN, C6-PLAYTEST

Round 2 (Group B — fire after Group A complete):
C6-WASM, C6-SIGN

Round 3 (Group C — fires after Group B complete):
C6-INTEGRATE

Exit Criteria for Checkpoint 6

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

Deferred to Post-Launch (C7 / Post-Alpha)

Full Apple Developer notarization if no paid Apple Developer account is available.

Windows EV code signing certificate.

Remote/online leaderboard server.

Long-running nightly CI balance campaigns if GitHub Actions minutes budget is constrained.

Web audio on Safari if compatibility problems remain.

Mobile iOS/Android targets.

Steam/GOG distribution packaging.

Production telemetry upload endpoint and account/auth infrastructure.
