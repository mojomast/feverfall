# Pre-Release QA Report

Date: 2026-07-04

Agent: `[C4-QA] Full QA Pass Agent`, reconciled by `[C4-INTEGRATE]`.

Status: Checkpoint 4 feature-complete automation passes. Human feel/comprehension benchmarks and Windows artifact verification remain release-process gaps, not feature-complete blockers.

## Validation Commands

| Command | Result | Exact output / hash evidence |
|---|---|---|
| `cargo fmt --all -- --check` | Pass | No output. |
| `cargo test --workspace` | Pass | Workspace completed with passing crate tests. Notable counts: `board_gen` 6 passed, `feverfall_game` 47 passed, `physics_core` 16 passed, `rpg_mode` 8 passed plus 2 mode-separation tests, `run_mode` 16 passed plus 2 mode-separation tests, `telemetry` 7 passed. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass | Strict clippy completed without warnings. |
| `cargo test -p telemetry` | Pass | 7 passed including `telemetry_derivation_and_logging_do_not_mutate_physics_state`. |
| `cargo run -p content_linter` | Pass | `content lint passed: 242 unique id(s)`. |
| `cargo run -p board_validator` | Pass | 80 authored boards emitted `PASS`; no failures. |
| `cargo run -p replay_runner` | Pass | Default minimal replay hash `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`. |
| `for replay in tests/golden_replays/*.json; do cargo run -p replay_runner -- --replay "$replay" || exit 1; done` | Pass | All seven golden replay hashes matched; see `docs/qa/replay_audit.md`. |
| `cargo run -p feverfall_game -- --smoke-full` | Pass | `smoke-full summary: PASS checks=12 replays=7`; hashes match the Smoke Hashes section. |
| `cargo run -p feverfall_game -- --smoke` | Pass | C2 hash `18202124e6b686d8`; feel hash `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`; RPG campaign hash `04029810211125c5`; roguelite Act 1-3 `e72374145338c3b3`; full roguelite `152fc850303d8356`. |
| `cargo run -p feverfall_game --features bevy_feel_test -- --smoke` | Pass | Same deterministic smoke output under feature build; C4 VFX trigger count `21`. |

## Replay Hashes

- Minimal: `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`
- Vertical slice: `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`
- Act 1 two-board: `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`
- RPG Chapter 1 defensive: `8e566217ee6cddee3be784b3e359b3eda5708638ac8540bce759086e922a145f`
- RPG Chapter 1 implementation: `fc72b1144ad88e62bb27c3a1296cbb9b3fa51871a852b9b5ef561d7146033a58`
- Roguelite 3-act defensive: `89c224a1ba8aae30965fa42f9547940036badc026b0a2f1bf50e6de15b86682b`
- Roguelite Act 1-3 implementation: `c5db0fb8d90e57c8be159bbb779c56ead19148f36de8bdc077711e59f9a4a36a`

## Smoke Hashes

- C2 run summary: `18202124e6b686d8`
- Feel-test replay: `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`
- RPG Ch1 sample: `c18385eaa33af638`
- RPG Ch3 sample: `01efbd0f270af2e8`
- RPG Ch5 mastery sample: `ef2ae2140c5abcdf`
- RPG campaign summary: `04029810211125c5`
- Preserved RPG Ch1 save/load smoke: `3364e243ba2065f4`
- Roguelite Act 1-3 smoke: `e72374145338c3b3`
- Roguelite full-run smoke: `152fc850303d8356`

## Telemetry Audit

Result: pass.

Added test: `crates/telemetry/src/lib.rs::tests::telemetry_derivation_and_logging_do_not_mutate_physics_state`.

Evidence:

- The test snapshots `BoardDefinition`, `ShotInput`, physics events, and replay hash.
- It logs `ShotStarted`, converted physics telemetry, and `ShotResolved` telemetry.
- It re-simulates the same shot and asserts the board/input/events/hash are unchanged.
- `cargo test -p telemetry` passes with 7 tests.

## Success Criteria / Feel Benchmarks

| Spec criterion | Target | Current result |
|---|---|---|
| Replay determinism | 100% same hash | Met for all seven golden replay fixtures. |
| Tunneling failures | 0 in 10,000 max-speed tests | Met by passing `physics_core` stress/no-tunneling tests. |
| Stuck-ball events | <0.1%; auto-resolve within 2.0s | Proxy met by passing stress tests; no smoke stuck events observed. |
| First-bounce prediction accuracy | Exact within solver tolerance | Met by tests and `first_bounce=true` smoke output. |
| Sim performance | 120 Hz under 0.25 ms normal board | Not measured in this QA pass; requires timing benchmark. |
| Board generator rejection | Rejects unplayable seeds | Met by `board_gen` tests and `board_validator` pass over current authored content. |
| Collision-to-feedback latency | <50 ms | Proxy met: feedback is generated synchronously from shot events; one 120 Hz tick is 8.33 ms. No wall-clock timing logs exist. |
| New-player comprehension | 80% explain orange/bucket/aim | Not met by automation; human survey required. |
| Physics felt unfair survey | <15% agree | Not met by automation; no unfair proxy tags observed. |
| One more shot desire | 70%+ agree | Not met by automation; human survey required. |
| Bucket catch satisfaction | 75%+ satisfying | Proxy inconclusive; current smoke had `bucket=0`. |
| Board clear payoff | 80% exciting/not annoying | Proxy pass via `extreme_fever` and accessibility caps; human survey required for annoyance. |
| Feedback clarity | 90% distinguish win/near miss/loss | Proxy pass via distinct trigger list and audio/VFX tests; human survey required. |
| Act 1 completion | 45-65% new players after tutorial | Not measured; current smoke is scripted, not a player cohort. |
| Full-run win rate | 10-20% before meta unlocks | Not measured in this QA pass. |
| Distinct build identity | Players name relic/ball combo | Not measured; human survey required. |
| Bad-seed reports | <1% runs | Proxy pass for authored validation; player reporting not available. |
| Campaign board replay | 30%+ replay for mastery medals | Not measured; telemetry cohort required. |
| Skill usage | Active skills used in 70%+ eligible boards | Proxy pass in smoke for two skill events; cohort target not measured. |
| Gear readability | 80% explain item changed | Not measured; human survey required. |
| Stat dominance | No stat line improves win >20% alone | Not measured; balance simulation required. |

## Known Gaps

- [P1] Human feel survey remains required for comprehension, unfairness, one-more-shot desire, bucket satisfaction, payoff annoyance, and feedback clarity percentages.
- [P1] Sim performance benchmark is not automated; add a timing harness for 120 Hz normal-board frame cost.
- [P1] Roguelite/RPG cohort benchmarks are not measured by smoke tests; add batch simulations or telemetry analysis for win rate, replay rate, skill usage, gear readability, and stat dominance.
- [P2] Full C4 content landed concurrently during QA; reports are re-runnable and reflect the current 242-ID/80-board content pack.
- [P2] Checkpoint 4 Windows artifact workflow must be rerun from a pushed C4 ref; local Linux validation cannot verify the nonlocal artifact.

## Remaining TODO

- [P1] Run a real 10-15 minute human feel-test session using `docs/playtesting/feel_survey.md` and append results beside `docs/qa/feel_test_results.md`.
- [P1] Add performance timing output for normal-board simulation and feedback latency.
- [P1] Re-run this QA suite after any post-C4 gameplay/content changes or golden replay additions.
- [P2] Decide whether `saves/` generated by smoke should be ignored or cleaned from the working tree before release commits.
- [P2] Trigger `.github/workflows/windows-feel-test.yml` from a pushed C4 ref and record artifact/checksum details in release notes.

## Commit Status

Final reconciliation is owned by `[C4-INTEGRATE]` after all C4 concurrent docs/state/contract changes were preserved and validated together.
