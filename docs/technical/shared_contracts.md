# Shared Contracts

## Type Ownership

- `content_schema`: content IDs, `BoardDefinition`, board primitives/objectives, boss mechanics, relic, ball, shop item, RPG gear, RPG skill, balance table, and content manifest schemas.
- `physics_core`: `ShotInput`, `SimConfig`, `PhysicsEvent`, `ShotResult`, first-bounce prediction, deterministic SHA-256 replay hashing.
- `game_rules`: `GameEvent`, scoring/resource event vocabulary, `ReplayMetadata`.
- `board_gen`: generation parameters and board validation reports.
- `run_mode`: `RunState`, run nodes, reward offers/choices, Act 1 slice helpers, relic modifier trait.
- `rpg_mode`: `CharacterState`, stats, Chapter 1 XP/leveling, gear inventory/equip state, active skills, full RPG campaign chapter catalog, mastery-mode unlock state, and campaign save/load.
- `feedback_events`: `FeedbackEvent`, accessibility feedback flags, gameplay-to-feedback mapping. Runtime feedback/VFX coverage uses existing feedback kinds for C3 peg, bucket, combo, long-shot, near-miss, final-orange, fever, and failure triggers.
- `telemetry`: telemetry envelopes, local JSONL logger, replay tags, QA analytics event vocabulary.

## Content ID Convention

IDs are ASCII, lowercase, namespace-friendly strings. They must start with a lowercase letter or digit. Allowed non-alphanumeric separators are `_`, `-`, `.`, `/`, and `:`.

Examples: `boards/minimal_test`, `relics/act1:first-bounce_bonus`, `skills/zen_reroute`.

## Duplicate Schema Rule

Feature crates must depend on these owners instead of creating private lookalike event, state, board, or ID schemas. Temporary mocks should wrap or construct these contract types.

## Mode Separation Contracts

- Roguelite owns `RunState`, roguelite relics, roguelite rewards, and roguelite balance data. It must not require `rpg_mode::CharacterState`, RPG gear, RPG skills, or campaign chapter data.
- RPG owns `CharacterState`, RPG gear, RPG skills, and campaign chapter data. It must not require `run_mode::RunState`, roguelite relics, roguelite rewards, or roguelite balance data.
- Both modes may consume `physics_core`, `game_rules`, `feedback_events`, and `telemetry` through their public mode-neutral contracts.
- `physics_core` must not depend on or import `run_mode` or `rpg_mode`; `crates/run_mode/tests/mode_separation.rs` and `crates/rpg_mode/tests/mode_separation.rs` enforce this with compile-time `include_str!` checks over the physics manifest/source.
- Roguelite saves live under `saves/roguelite/`; RPG saves live under `saves/rpg/`.
- Roguelite balance tables live under `content/balance/roguelite/`; RPG balance tables live under `content/balance/rpg/`.

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
- RPG chapter boards are identified by `rpg/chapter*` tags and must include at least one `BoardObjective` with a positive target. Objective IDs must be board-local unique content IDs.
- Checkpoint 1 dead-zone rejection threshold is `> 15%` playable aim samples for general boards. Authored aim-assist tutorial boards may use up to `20%`, and authored chapter boards tagged `rpg_ch*` may use up to `30%` while remaining explicit authored content.
- Boss boards may use boss-tagged validation allowances and must still pass `board_validator`; boss-tagged boards must declare a structured `boss_mechanic` with positive `intensity` and `cadence_shots`. C4 defines 12 mechanic kinds: `ShieldPegs`, `HealingPegs`, `RotatingBlockers`, `TimedHazardRows`, `SplitterStorm`, `GravityPulse`, `BasketLock`, `GhostVeil`, `BombCountdown`, `MirrorWalls`, `CurseTax`, and `FeverDrain`.

## Content Schema Contracts

- `content_schema::RelicDefinition` is the shared TOML/serde contract for relic content. It owns `RelicId`, display name, `RelicCategory`, `Rarity`, act, description, and effect reference IDs.
- `content_schema::BallVariantDefinition` is the shared TOML/serde contract for ball content. It owns `BallId`, family ID, rarity, description, positive `BallStats`, and effect reference IDs.
- `content_schema::ShopItemDefinition` is the shared TOML/serde contract for shop content. It owns `ShopItemId`, `ShopItemType`, act, price, stock weight, description, and the granted content ID.
- `content_schema::GearDefinition` is the shared TOML/serde contract for RPG gear content. It owns `GearId`, gear slot, rarity, level requirement, display text, and effect reference IDs.
- `content_schema::RpgSkillDefinition` is the shared TOML/serde contract for RPG skill content. It owns `SkillId`, skill tree, rank/unlock fields, timing, display text, and effect reference IDs.
- `content_schema::BalanceTableDefinition` is the shared TOML/serde contract for balance tables. It owns a table ID, version, and finite numeric keyed entries.
- `tools/content_linter` is the schema/id gate for board JSON plus relic, ball, shop, RPG gear, RPG skill, and balance table content under `game/assets/content` and top-level `content`. The current C4 content linter pass reports 242 unique IDs.
- C4 full content targets are recorded in `docs/design/content_manifest.md`: 60 relics with 12 per category, 20 ball variants, 40 RPG gear items with 10 per target slot, 36 RPG skills with 9 per tree, and 80 authored board templates.

## RPG Campaign Contracts

- Chapter 1 authored boards are `boards/rpg_ch1_01` through `boards/rpg_ch1_05`.
- `rpg_mode::CharacterState::chapter1()` creates the canonical Chapter 1 campaign start with Aim, Control, Resonance, Luck, launcher/core-ball inventory, and Zen Reroute/Catch Magnet active skills.
- XP awards are `100 + 20 * objective_tiers_met`; level thresholds are 200, 500, and 900 XP for levels 2, 3, and 4.
- Level-up grants stat points allocated to `ChapterStat::{Aim, Control, Resonance, Luck}`. Older stat fields remain serialized for existing C2 compatibility.
- Gear swapping is represented by `CharacterState::equip_gear` and `unequip_gear` for Launcher/CoreBall before campaign boards.
- Active skills use `CharacterState::use_skill`, are once per board, and tick cooldowns by board completion through `finish_board_cooldowns`.
- Campaign save/load uses versioned JSON at `saves/rpg/campaign.json`; unknown versions return `CampaignSaveError::UnknownVersion` instead of panicking.
- Default game smoke runs RPG campaign samples for Ch1 board 1, Ch3 board 1, and Ch5 board 1, then runs the preserved abbreviated Chapter 1 path over boards 1 and 5 and emits `TelemetryEvent::SkillUsed` for Zen Reroute and Catch Magnet.
- `rpg_mode::campaign_chapters()` owns the C4 RPG campaign catalog: Chapter 1 has 5 boards, Chapter 2 has 12 boards, Chapter 3 has 15 boards, Chapter 4 has 15 boards, and Chapter 5 has 4 Endgame mastery boards.
- `CharacterState::mark_chapter_cleared(chapter)` records `campaign/chapter{n}_cleared`; `CharacterState::has_cleared_all_chapters()` requires chapters 1 through 5.
- Clearing all 5 chapters unlocks `rpg_mode::MASTERY_MODE_FLAG`, serialized as `campaign/mastery_mode_unlocked` in `CharacterState::campaign_flags`.
- `CharacterState::normalized_for_mastery()` removes equipped gear and gear inventory for normalized-stat mastery while preserving base stats and unlocked skills.
- Chapter 5 mastery boards set `normalized_mastery=true` and expose a stable per-board `leaderboard_hash` derived from board ID, chapter, objectives, introduced tags, and normalized-mastery state.

## Run Mode Contracts

- `RunState::act1_slice(seed)` creates the canonical C2 Act 1 starting run state.
- `run_mode::ROGUELITE_SAVE_DIR` is `saves/roguelite/`; `run_mode::ROGUELITE_BALANCE_DIR` is `content/balance/roguelite/`.
- `run_mode::act1_slice_nodes()` owns the canonical short Act 1 node path used by C2 smoke/UI summaries.
- `run_mode::act1_slice_reward_offers()` owns deterministic reward choices for the short C2 path.
- `RunState::advance_to_node(node)` and `RunState::apply_reward(reward)` are the shared helpers for node progression and reward application. UI and game runtime code should call these instead of duplicating state mutations.
- `run_mode::full_run_act_plan()` and `run_mode::full_run_nodes()` own the C3 three-act roguelite structure: Act 1 is 6 normal boards, 1 elite, and 1 boss; Act 2 is 7 normal boards, 2 elites, and 1 boss; Act 3 is 8 normal boards, 2 elites, and 1 boss. Each act includes at least two deterministic path choices plus shop/event/forge/camp utility nodes.
- `run_mode::act4_unlocked(state)` gates optional Act 4 on `ACT4_REQUIRED_KEYS == 3`; the legacy C3 `full_run_act_plan()` and `full_run_nodes()` remain three-act only.
- `run_mode::act4_seed(run_seed)`, `act4_final_seed_board_specs(run_seed)`, and `full_run_nodes_with_act4(run_seed)` own the optional Act 4 Final Seed contract: 4 high-risk generated boards plus 1 final boss, all seeded from the run seed through the derived Act 4 seed. Act 4 board specs expose `CurseFrequency` and `meta_reward_rarity`, currently `High`/`Extreme` curse frequency and boss-tier meta rewards.
- `run_mode::ScriptedBossMechanic` and `BossMechanicKind` describe boss mechanics in content-compatible ID form. The Act 4 final boss uses `boss_mechanics/act4/final_seed_row_tempo`, combining `ScriptedObstacleRow` and `BucketTempoShift`.
- `run_mode::RelicModifier` now includes `modify_board(board, state)` and `on_event(event, state)` hooks. `run_mode::ContentRelicModifier`, `apply_relic_board_modifiers`, and `trigger_relics_on_event` wire the 20 Act 1 relic content IDs to concrete board/resource mutations and emit `FeedbackKind::RelicTriggered` feedback.
- `RunState::accept_curse()` increments curse risk on the current run and raises the resulting reward rarity track. `run_mode::MetaProgressionSave` is the skeleton save contract for `saves/roguelite/meta.json`, tracking total runs, total oranges cleared, relics seen, three-option meta-unlock offers, and `mastery_records`. `MetaProgressionSave::record_full_fever_cleared()` records the exact Act 4 win string `Full Fever Cleared`, and `act4_mastery_unlock_offer()` returns the higher-tier Act 4 meta unlock offer.

## UI Contracts

- `game::plugins::ui::PlaceholderScreenSuite` owns C4 production-placeholder screen models for main menu, settings, roguelite node/shop/forge/event/relic-bar screens, and RPG chapter/gear/skill/campaign screens.
- Placeholder UI must remain keyboard-navigable through `FocusList` and `UiNavInput`; renderer-specific code should consume these models instead of inventing separate focus state.
- `UiViewport::HD` and `UiViewport::FHD` are the smoke targets for 1280x720 and 1920x1080 layout checks until renderer automation lands.
- `game::plugins::debug::F3DebugOverlay` owns the debug overlay contract toggled by F3: physics tick rate, last replay hash, current board ID, active relic count, and telemetry event count.

## RPG Mode Contracts

- `CharacterState::act1_slice()` creates the canonical C2 RPG companion state without depending on roguelite run state.
- `rpg_mode::RPG_SAVE_DIR` is `saves/rpg/`; `rpg_mode::RPG_BALANCE_DIR` is `content/balance/rpg/`.
- `rpg_mode::CHAPTER1_SAVE_PATH` is `saves/rpg/campaign.json`; `load_campaign` checks the save version before deserializing the full character payload so future-version files fail as `UnknownVersion` even when their body shape has changed.
- RPG gear and skill cooldown state belongs to `CharacterState`/`SkillState`; roguelite systems must not mutate it directly.
- RPG Chapter 2-5 board IDs reserved by `rpg_mode` are `boards/rpg_ch2_*`, `boards/rpg_ch3_*`, `boards/rpg_ch4_*`, and `boards/rpg_ch5_mastery_*`; authored content packs may use distinct IDs such as `boards/c4_rpg_*` to avoid duplicate content IDs.

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
- RPG campaign fixtures may include an `rpg_mode::CharacterState` `character_state` snapshot alongside `board_path`; the runner parses and reports the snapshot count but keeps replay hashing scoped to deterministic physics events.
- Multi-board fixtures use ordered `boards` entries, each with a `board_path` and `shots`; the runner simulates shots in order, carries remaining pegs within each board, increments seeds by global shot count, and hashes the combined physics events.
- C2 golden hashes are default replay `f9de2e888670d1d7da3e7e65db54c53e4217f059d375e9f17b7f36dfb9e49031`, vertical-slice replay `39a27a4d0e60d29262c33894837dd1434814aa9252e23309fe87c55f7d5ac383`, and Act 1 two-board replay `1d1a7485925e15c4a1a917ebcda582188df1748b1030ce9669887df224408455`.
- C3 defensive smoke replay hashes are RPG Chapter 1 `8e566217ee6cddee3be784b3e359b3eda5708638ac8540bce759086e922a145f` and roguelite 3-act `89c224a1ba8aae30965fa42f9547940036badc026b0a2f1bf50e6de15b86682b`.
- C3 RPG implementation smoke fixture `tests/golden_replays/rpg_ch1_smoke.replay.json` covers Chapter 1 boards 1 and 5 with hash `fc72b1144ad88e62bb27c3a1296cbb9b3fa51871a852b9b5ef561d7146033a58`.
- `cargo run -p feverfall_game -- --smoke-full` is the C4 release smoke contract. Exit code 0 means the C2 vertical slice, RPG Chapter 1 plus campaign Chapter 1/3/5 smoke, roguelite Acts 1-4 smoke, feel-test smoke, full content lint, board validation, and every golden replay fixture have all passed.
