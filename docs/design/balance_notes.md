# Roguelite Balance Notes

## Simulation Methodology

- Agent: `[C3-BALANCE]`.
- Runner: `cargo run -p balance_sim --release` using `board_gen::generate_board` and `physics_core::simulate_shot` directly because the C3 roguelite smoke/headless API has not landed yet.
- Run count: 1,000 roguelite runs.
- Seed range: `0xc3ba000000000000` through `0xc3ba0000000003e7`.
- Run shape: 3 acts, 3 boards per act, random generated board archetypes, random aim angles, random launch-speed jitter, bucket catches granting one extra ball, and random reward choices after cleared boards.
- Reward model: local headless approximation with relic, ball, coin, and heal outcomes. It is intentionally isolated in `tools/balance_sim` so it can be replaced by `[C3-ROGUELITE]` APIs once integrated.

## Key Metrics

| Metric | Result |
|---|---:|
| Runs simulated | 1,000 |
| Act 1 win rate | 0.0% cleared, 1,000 started |
| Act 2 win rate | 0.0% cleared, 3 started |
| Act 3 win rate | 0.0% cleared, 0 started |
| Full-run win rate | 0.0% |
| Average oranges cleared per board | 14.90 |
| Average relics collected per run | 0.00 |
| Average run length | 35.27 shots |
| Boards played | 3,003 |
| Most chosen reward | `relic:boss_feverheart` x2 |
| Least chosen reward | `relic:common_spark_catcher` x1 |

## Findings

- The random-skill baseline is too punishing: average oranges cleared per board is below the current 20-25 Act 1 orange range, so most boards fail before reward economy can activate.
- Reward metrics are sparse because almost no boards are cleared; relic tuning must first guarantee early access after any clear and avoid relying on deep-run sampling.
- Act 2 and Act 3 data are not yet meaningful because Act 1 is a hard progression gate in random simulation.
- Average run length is high despite zero win rate, indicating runs are spending many shots failing boards rather than making fast, informative progress.

## Applied Parameter Changes

| Parameter | Before | After | Applied In | Rationale |
|---|---:|---:|---|---|
| Act 1 orange peg range | 20-25 | 18-21 | `content/balance/roguelite/board_curve.toml` | Aligns first-act clear target with 14.90 average random oranges cleared while still requiring improved aim or free balls. |
| Act 1 starting balls | 10 | 12 | `content/balance/roguelite/board_curve.toml` | Gives the early run enough shots for reward economy to appear in simulations and playtests. |
| Board-clear relic drop rate | ad hoc / not tabled | 62% with guaranteed relic choice until 2 relics | `content/balance/roguelite/reward_pool.toml` | Prevents zero-relic runs after early clears and increases build identity sampling. |
| First/second/third free-ball score thresholds | 25,000 / 75,000 / 125,000 | 18,000 / 55,000 / 95,000 | `content/balance/roguelite/scoring_curve.toml` | Adds recovery without changing physics or hidden RNG. |
| Combo multiplier curve | x1 flat in C2 smoke scoring | x1.25 at 3, x1.75 at 6, x2.5 at 10, x3.5 at 15 | `content/balance/roguelite/scoring_curve.toml` | Rewards multi-peg shots and helps score-based free balls trigger before the board is already lost. |

## Additional Recommendations

- Use the new `board_curve.toml` as the source of truth when `[C3-ROGUELITE]` wires generated-board difficulty into run progression.
- Re-run `balance_sim` after reward and scoring tables are consumed by runtime code; target Act 1 random/bot completion should rise from 0% toward 25-35% before human skill adjustment.
- Keep Act 2 and Act 3 orange counts conservative until Act 1 produces enough surviving samples for meaningful downstream metrics.
- Add telemetry buckets for oranges remaining at failure, free balls earned by source, and relic count at each act transition.

## Validation

- `cargo run -p balance_sim --release` could not use the repo release target because `target/release/.cargo-lock` returned permission denied, so the full simulation was run with `CARGO_TARGET_DIR=/tmp/opencode/feverfall-target`.
- `cargo run -p content_linter` must pass after these content-table changes.
