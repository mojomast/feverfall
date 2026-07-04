# Replay Tagging

Replay tags are lightweight QA annotations attached to deterministic replay hashes. They are for triage and playtest analysis only; they must not feed back into physics simulation.

## JSON Shape

```json
{
  "replay_hash": "64-char-sha256-hex",
  "board": "boards/minimal_test",
  "seed": 123,
  "labels": ["DeterminismBaseline", "FirstBounceReadable", "BucketCatchMissed"],
  "notes": "Shot 2 missed the bucket by a small margin; miss felt fair."
}
```

## Checkpoint 2 Labels

- `DeterminismBaseline`
- `FeelTooFloaty`
- `FeelTooPinball`
- `BucketCatchSatisfying`
- `BucketCatchMissed`
- `PhysicsFeltUnfair`
- `FirstBounceReadable`
- `VerticalSliceFailure`
- `Custom(String)`

The Rust source of truth for this format is `telemetry::ReplayTag` and `telemetry::ReplayLabel`.

## Vertical Slice Notes

- Add `BucketCatchSatisfying` when the catch reads clearly and feels earned.
- Add `BucketCatchMissed` when a near miss needs review, even if it felt fair.
- Add `VerticalSliceFailure` when the short run blocks completion, progression, scoring, replay capture, or feedback review.
- Put shot index, score/progression outcome, and failure/catch details in `notes` instead of adding new ad hoc fields.

## Run-Session Tags

Use `Custom(String)` labels with a stable `run-session:` prefix for multi-board Checkpoint 2 runs. These labels annotate the replay only; they must not change replay input, physics state, reward rolls, node choices, or replay hashes.

Recommended labels:
- `run-session:act1-node-01`
- `run-session:act1-node-02`
- `run-session:two-board-round-trip`
- `run-session:reward-choice-clear`
- `run-session:reward-choice-unclear`
- `run-session:node-map-clear`
- `run-session:node-map-unclear`
- `run-session:run-summary-reviewed`

Run-session `notes` should include:
- `run_id`
- current node ID and next node ID
- board IDs played in order
- selected reward ID or resource reward
- final score, shots, hearts, coins, sparks, XP, and relic count
- replay hashes for both boards or shots under review

Example:

```json
{
  "replay_hash": "64-char-sha256-hex",
  "board": "boards/feel_wave_01",
  "seed": 13961158844848537602,
  "labels": [
    "DeterminismBaseline",
    { "Custom": "run-session:act1-node-02" },
    { "Custom": "run-session:reward-choice-clear" },
    { "Custom": "run-session:run-summary-reviewed" }
  ],
  "notes": "run_id=run/c2a0000000000002 nodes=runs/act1/node_01>runs/act1/node_02 boards=boards/feel_fan_01>boards/feel_wave_01 reward=relics/act1/orange_echo score=7500 shots=6 hearts=3 coins=22 sparks=9 xp=9 relics=2"
}
```
