# RPG Balance Notes

Checkpoint 5 adds authored RPG balance tables under `content/balance/rpg/` and a deterministic RPG simulator module for `balance_sim`.

## Tables

- `cohorts.toml` defines the seven required cohorts: on-curve, underleveled, overleveled, no-skill, skill-aware, gear archetype, and normalized mastery.
- `progression.toml` defines the XP/level curve, chapter board targets, gear acquisition timing, and stat dominance threshold.
- `content_coverage.toml` enumerates C4 RPG gear, skills, chapters, and board counts covered by the sim.

## Metrics

The RPG metrics schema is deterministic JSON with these stable top-level fields:

- `schema_version`
- `seed_start`
- `seed_count`
- `cohorts[]`
- `content_coverage`

Each cohort reports board clear rate, objective-tier rate, retries to clear, XP/level curve, gear acquisition timing, skill unlock/use rate, stat dominance, chapter completion, and mastery normalized clear.

## Interpretation

The C5 simulator is a lightweight deterministic balance model, not a replacement for human playtests. It is intended to flag curve regressions, missing coverage, and stat dominance outliers before integration runs heavier validation.

Stat dominance is flagged when any stat share exceeds `dominance_outlier_ratio` from `progression.toml` (currently `0.45`). Normalized mastery clears intentionally use normalized gear timing and a small difficulty penalty so mastery boards can be compared independently of campaign gear spikes.
