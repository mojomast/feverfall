# Shared Contracts

## Type Ownership

- `content_schema`: content IDs, `BoardDefinition`, board primitives, content manifests.
- `physics_core`: `ShotInput`, `SimConfig`, `PhysicsEvent`, deterministic replay event hashing.
- `game_rules`: `GameEvent`, scoring/resource event vocabulary, `ReplayMetadata`.
- `board_gen`: generation parameters and board validation reports.
- `run_mode`: `RunState`, run nodes, rewards, relic modifier trait.
- `rpg_mode`: `CharacterState`, stats, gear slots, skills.
- `feedback_events`: `FeedbackEvent`, accessibility feedback flags, gameplay-to-feedback mapping.

## Content ID Convention

IDs are ASCII, lowercase, namespace-friendly strings. They must start with a lowercase letter or digit. Allowed non-alphanumeric separators are `_`, `-`, `.`, `/`, and `:`.

Examples: `boards/minimal_test`, `relics/act1:first-bounce_bonus`, `skills/zen_reroute`.

## Duplicate Schema Rule

Feature crates must depend on these owners instead of creating private lookalike event, state, board, or ID schemas. Temporary mocks should wrap or construct these contract types.
