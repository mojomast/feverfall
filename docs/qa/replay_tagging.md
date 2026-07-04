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
