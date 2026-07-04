# Shared Contracts

## Type Ownership

- `content_schema`: content IDs, `BoardDefinition`, board primitives, content manifests.
- `physics_core`: `ShotInput`, `SimConfig`, `PhysicsEvent`, `ShotResult`, first-bounce prediction, deterministic SHA-256 replay hashing.
- `game_rules`: `GameEvent`, scoring/resource event vocabulary, `ReplayMetadata`.
- `board_gen`: generation parameters and board validation reports.
- `run_mode`: `RunState`, run nodes, rewards, relic modifier trait.
- `rpg_mode`: `CharacterState`, stats, gear slots, skills.
- `feedback_events`: `FeedbackEvent`, accessibility feedback flags, gameplay-to-feedback mapping.
- `telemetry`: telemetry envelopes, local JSONL logger, replay tags, QA analytics event vocabulary.

## Content ID Convention

IDs are ASCII, lowercase, namespace-friendly strings. They must start with a lowercase letter or digit. Allowed non-alphanumeric separators are `_`, `-`, `.`, `/`, and `:`.

Examples: `boards/minimal_test`, `relics/act1:first-bounce_bonus`, `skills/zen_reroute`.

## Duplicate Schema Rule

Feature crates must depend on these owners instead of creating private lookalike event, state, board, or ID schemas. Temporary mocks should wrap or construct these contract types.

## Physics Contracts

- `physics_core::simulate_shot(seed, board, input)` runs the deterministic fixed-step simulator and returns `ShotResult`.
- `physics_core::predict_first_bounce(board, input)` returns the first collision event using the same CCD path as shot simulation.
- `physics_core::sample_shot_trajectory(seed, board, input)` returns deterministic trajectory samples for visual playback using the same simulator integration path.
- `ShotSummary::replay_hash` is a lowercase SHA-256 hex string derived from seed, shot input, config, per-tick state, and events.
- `PhysicsEvent::ShotEnded` carries the canonical shot summary, including hit pegs, catch/exit flags, tick count, and replay hash.
- The simulator applies implicit left/right board wall collisions so balls cannot escape horizontally under the speed cap.

## Board Validation Contracts

- Authored boards live under `game/assets/content/boards/*.json` using `content_schema::BoardDefinition`.
- `board_gen::load_authored_boards` loads authored board data.
- `board_gen::validate_board` enforces orange count, board bounds, bucket bounds, first-collision orange reachability, catch opportunity, and dead-zone thresholds.
- Checkpoint 1 dead-zone rejection threshold is `> 15%` playable aim samples.

## Telemetry Contracts

- `telemetry::JsonlTelemetryLogger` writes one `TelemetryEnvelope` per JSONL line to a caller-provided writer.
- Telemetry observes cloned IDs, positions, ticks, and hashes. It must not mutate board, replay, or physics simulation state.
- `telemetry::ReplayTag` annotates deterministic replay hashes for QA triage and feel analysis.
