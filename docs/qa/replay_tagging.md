# Replay Tagging

Replay tags are lightweight QA annotations attached to deterministic replay hashes. They are for triage and playtest analysis only; they must not feed back into physics simulation.

## JSON Shape

```json
{
  "replay_hash": "64-char-sha256-hex",
  "board": "boards/minimal_test",
  "seed": 123,
  "labels": ["DeterminismBaseline", "FirstBounceReadable"],
  "notes": "First bounce was clear; bucket miss felt fair."
}
```

## Checkpoint 1 Labels

- `DeterminismBaseline`
- `FeelTooFloaty`
- `FeelTooPinball`
- `BucketCatchSatisfying`
- `PhysicsFeltUnfair`
- `FirstBounceReadable`
- `Custom(String)`

The Rust source of truth for this format is `telemetry::ReplayTag` and `telemetry::ReplayLabel`.
