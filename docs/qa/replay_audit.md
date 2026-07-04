# Replay Audit

Date: 2026-07-04

Command: `for replay in tests/golden_replays/*.json; do cargo run -p replay_runner -- --replay "$replay" || exit 1; done`

Result: pass. All golden replay hashes matched.

| Replay fixture | Boards | Shots | Character snapshots | Mode | Hash |
|---|---:|---:|---:|---|---|
| `tests/golden_replays/act1_twobboard_run.replay.json` | 2 | 2 | 0 | `act1_two_board_smoke` | `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455` |
| `tests/golden_replays/minimal_test.replay.json` | 1 | 1 | 0 | `simulate_shot` | `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031` |
| `tests/golden_replays/roguelite_3act_smoke.replay.json` | 3 | 3 | 0 | `roguelite_3_act_smoke` | `89c224a1ba8aae30965fa42f9547940036badc026b0a2f1bf50e6de15b86682b` |
| `tests/golden_replays/roguelite_act1to3_smoke.replay.json` | 3 | 3 | 0 | `roguelite_act1to3_smoke` | `c5db0fb8d90e57c8be159bbb779c56ead19148f36de8bdc077711e59f9a4a36a` |
| `tests/golden_replays/rpg_ch1_smoke.replay.json` | 2 | 2 | 0 | `rpg_chapter1_smoke` | `fc72b1144ad88e62bb27c3a1296cbb9b3fa51871a852b9b5ef561d7146033a58` |
| `tests/golden_replays/rpg_chapter1_smoke.replay.json` | 1 | 1 | 1 | `rpg_campaign_chapter1_smoke` | `8e566217ee6cddee3be784b3e359b3eda5708638ac8540bce759086e922a145f` |
| `tests/golden_replays/vertical_slice_feel_fan.replay.json` | 1 | 1 | 0 | `simulate_shot` | `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383` |

Shared versions emitted by all replay runs:

- Rules: `game_rules/0.1.0-contracts`
- Physics: `physics_core/0.2.0-fixed-step`

Re-run note: This audit intentionally uses shell glob expansion over `tests/golden_replays/*.json` so newly landed replay fixtures are included automatically.
