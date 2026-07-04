# Shared Contracts

## Type Ownership

- `content_schema`: content IDs, `BoardDefinition`, board primitives, relic, ball, shop item, and content manifest schemas.
- `physics_core`: `ShotInput`, `SimConfig`, `PhysicsEvent`, `ShotResult`, first-bounce prediction, deterministic SHA-256 replay hashing.
- `game_rules`: `GameEvent`, scoring/resource event vocabulary, `ReplayMetadata`.
- `board_gen`: generation parameters and board validation reports.
- `run_mode`: `RunState`, run nodes, reward offers/choices, Act 1 slice helpers, relic modifier trait.
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
- Boss boards may use boss-tagged validation allowances and must still pass `board_validator`; the current C2 content set includes `PASS boards/act1_boss_01`.

## Content Schema Contracts

- `content_schema::RelicDefinition` is the shared TOML/serde contract for relic content. It owns `RelicId`, display name, `RelicCategory`, `Rarity`, act, description, and effect reference IDs.
- `content_schema::BallVariantDefinition` is the shared TOML/serde contract for ball content. It owns `BallId`, family ID, rarity, description, positive `BallStats`, and effect reference IDs.
- `content_schema::ShopItemDefinition` is the shared TOML/serde contract for shop content. It owns `ShopItemId`, `ShopItemType`, act, price, stock weight, description, and the granted content ID.
- `tools/content_linter` is the schema/id gate for board JSON plus relic, ball, and shop TOML content. The current C2 content linter pass reports 44 unique IDs.

## Run Mode Contracts

- `RunState::act1_slice(seed)` creates the canonical C2 Act 1 starting run state.
- `run_mode::act1_slice_nodes()` owns the canonical short Act 1 node path used by C2 smoke/UI summaries.
- `run_mode::act1_slice_reward_offers()` owns deterministic reward choices for the short C2 path.
- `RunState::advance_to_node(node)` and `RunState::apply_reward(reward)` are the shared helpers for node progression and reward application. UI and game runtime code should call these instead of duplicating state mutations.

## Telemetry Contracts

- `telemetry::JsonlTelemetryLogger` writes one `TelemetryEnvelope` per JSONL line to a caller-provided writer.
- Telemetry observes cloned IDs, positions, ticks, and hashes. It must not mutate board, replay, or physics simulation state.
- `telemetry::ReplayTag` annotates deterministic replay hashes for QA triage and feel analysis.
- `telemetry::shot_summary_to_telemetry` records the vertical-slice shot result from `physics_core::ShotSummary`, including replay hash, peg count, catch flag, exit flag, and tick count.
- `telemetry::game_event_to_telemetry` records score and board progression outcomes from `game_rules::GameEvent`; callers should not create private score/progression telemetry schemas.
- `telemetry::TelemetryEvent::RunEnded` records final score, boards cleared, oranges cleared, bucket catches, relics collected, XP gained, character level, run duration in shots, and the run replay/summary hash for integrated C2 smoke and playtest sessions.

## Replay Runner Contracts

- `replay_runner` reads `tests/golden_replays/minimal_test.replay.json` by default, or a fixture path via `--replay FILE`.
- Single-board fixtures may embed `board` or reference `board_path`.
- Multi-board fixtures use ordered `boards` entries, each with a `board_path` and `shots`; the runner simulates shots in order, carries remaining pegs within each board, increments seeds by global shot count, and hashes the combined physics events.
- C2 golden hashes are default replay `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`, vertical-slice replay `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`, and Act 1 two-board replay `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`.
