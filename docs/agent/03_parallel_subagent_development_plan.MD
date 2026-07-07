# Artifact 3 — Parallel Subagent Development Plan

## Integration principle

Physics feel is the dependency root. Other workstreams may start immediately using mocked events and schemas, but no mode balance or content quantity should be considered valid until the Physics Feel Alpha checkpoint passes.

---

## Workstream A — Physics Core & Feel

**Subagent role:** Senior gameplay physics engineer.

**Inputs needed**
- Core physics spec.
- Board coordinate system.
- Peg/basket event schema.
- Target feel benchmarks.

**Outputs**
- `crates/physics_core`
- deterministic fixed-step simulator,
- collision primitives,
- sweep/CCD tests,
- replay hash support,
- debug visualization hooks,
- physics tuning file.

**Interfaces**

```rust
struct ShotInput { aim_angle: f64, launch_speed: f64, ball_id: BallId }
struct SimConfig { gravity, restitution_table, damping, speed_cap }
enum PhysicsEvent {
  BallHitPeg { ball, peg, position, normal, speed },
  BallHitObstacle { ball, obstacle, position, normal },
  BallEnteredBucket { ball, tick },
  BallExitedBoard { ball, tick },
  ShotEnded { summary },
}
```

**Dependency order**
- Starts immediately.
- Must deliver stable API before VFX, board validation, roguelite balance, and RPG skill effects lock.

**Validation**
- Unit tests for circle/capsule/segment collision.
- Property tests for no NaNs, no tunneling under speed cap.
- Golden replay hash tests.
- 10 prototype boards manually tuned for feel.
- Blind playtest comparing “too floaty / too pinball / just right.”

---

## Workstream B — Board Schema & Procedural Generation

**Subagent role:** Procedural content systems designer / engineer.

**Inputs needed**
- Board coordinate system.
- Peg type definitions.
- Physics simulator API.
- Difficulty budgets.

**Outputs**
- `crates/board_gen`
- board schema,
- template library,
- seeded generator,
- board validator,
- seed browser,
- 30 prototype boards.

**Interfaces**

```rust
struct BoardDefinition {
  id: String,
  size: Vec2,
  pegs: Vec<PegDef>,
  obstacles: Vec<ObstacleDef>,
  basket: BasketDef,
  tags: Vec<Tag>,
}

struct GenerationParams {
  act, difficulty, archetype, seed, peg_budget, hazard_budget
}
```

**Dependency order**
- Starts immediately with placeholder physics mocks.
- Validator requires Workstream A simulator.

**Validation**
- Schema linting.
- Seed determinism tests.
- Monte Carlo aim sampling.
- Solvability thresholds.
- Dead-zone heatmaps.
- Manual “interesting first shot” review.

---

## Workstream C — Roguelite Run Systems

**Subagent role:** Roguelite systems designer / economy engineer.

**Inputs needed**
- Relic taxonomy.
- Board difficulty tags.
- Run resource definitions.
- Reward schema.

**Outputs**
- `crates/run_mode`
- run map generator,
- node system,
- relic system,
- shop/forge/event rules,
- reward generation,
- meta-unlock save data,
- 50 MVP relics.

**Interfaces**

```rust
struct RunState { act, node, hearts, coins, relics, balls, curse, rng_state }
trait RelicModifier {
  fn modify_board(&self, board: &mut BoardDefinition);
  fn on_event(&self, event: &GameEvent, state: &mut RunState);
}
struct RewardOffer { choices: Vec<Reward>, rarity, source }
```

**Dependency order**
- Starts immediately using mocked board completion.
- Balance waits for Workstream A/B.

**Validation**
- Run-state serialization tests.
- Reward distribution tests.
- Relic trigger tests.
- Simulated 10,000-run economy sanity checks.
- Human vertical-slice run test.

---

## Workstream D — RPG Progression & Campaign

**Subagent role:** RPG systems designer / progression engineer.

**Inputs needed**
- Stat/gear/skill specification.
- Campaign chapter structure.
- Save schema.
- Modifier interface from game rules.

**Outputs**
- `crates/rpg_mode`
- character state,
- XP/leveling,
- gear inventory,
- skill trees,
- campaign map,
- board objective system,
- 20 prototype RPG boards.

**Interfaces**

```rust
struct CharacterState { level, xp, stats, gear, skills, campaign_flags }
struct GearModifier { slot, effects, rarity, level_req }
struct SkillUse { skill_id, timing_window, target }
```

**Dependency order**
- Starts immediately with abstract board outcomes.
- Active physics skills depend on Workstream A event hooks.

**Validation**
- Save/load migration tests.
- Skill-tree unlock tests.
- Gear modifier tests.
- Normalized-stat mastery tests.
- Campaign progression smoke test.

---

## Workstream E — Feedback, VFX, and Audio Event Layer

**Subagent role:** Game-feel / juice engineer.

**Inputs needed**
- Feedback trigger map.
- Physics/gameplay event schema.
- Primitive asset constraints.
- Audio bus rules.

**Outputs**
- Bevy VFX plugin,
- particle systems,
- score floaters,
- camera impulse system,
- combo rail,
- fever sequence timeline,
- procedural placeholder SFX integration.

**Interfaces**

```rust
struct FeedbackEvent {
  kind: FeedbackKind,
  intensity: f32,
  position: Vec2,
  combo: u32,
  value: i64,
}
```

**Dependency order**
- Starts immediately using mocked event playback.
- Final timing requires Workstream A event cadence.

**Validation**
- Event playback test scene.
- Collision-to-feedback latency measurement.
- Audio voice-limit tests.
- Accessibility toggles: reduce shake, reduce flash, mute high-frequency layers.
- Playtest: event readability and excitement.

---

## Workstream F — UI / HUD / Menus

**Subagent role:** UI engineer / UX designer.

**Inputs needed**
- Mode state schemas.
- Feedback event priorities.
- Run/RPG state models.
- Minimal art rules.

**Outputs**
- aim HUD,
- score HUD,
- fever meter,
- ball inventory,
- relic bar,
- run map screen,
- RPG inventory/skills screen,
- debug overlays.

**Interfaces**

```rust
struct HudState {
  balls_remaining,
  score,
  multiplier,
  combo,
  objective_state,
  active_power,
}
```

**Dependency order**
- Starts immediately with mock data.
- Integrates with Workstreams C/D after state APIs stabilize.

**Validation**
- Controller/mouse/keyboard navigation tests.
- UI screenshot regression tests.
- Readability at 720p / 1080p / Steam Deck scale.
- 5-second comprehension tests for core HUD.

---

## Workstream G — Build, Tooling, CI

**Subagent role:** DevOps / tools engineer.

**Inputs needed**
- Repo structure.
- Target platforms.
- Replay requirements.
- Content schema.

**Outputs**
- Cargo workspace,
- GitHub Actions CI,
- formatting/linting,
- desktop build,
- web build,
- replay runner,
- content linter,
- seed browser CLI,
- nightly performance benchmark command.

**Interfaces**
- `cargo test`
- `cargo run -p replay_runner`
- `cargo run -p board_validator`
- `cargo run -p content_linter`
- build artifacts for desktop and web.

**Dependency order**
- Starts immediately.
- Blocks all merge gates once baseline CI exists.

**Validation**
- CI must run fmt, clippy, tests, golden replays, content lint, and build smoke tests.
- Failing replay hash blocks merge unless ruleset version changed intentionally.

---

## Workstream H — Content & Balance Data

**Subagent role:** Technical designer / balance analyst.

**Inputs needed**
- Content schemas.
- Initial tunables.
- Relic/gear/skill taxonomies.
- Board generator outputs.

**Outputs**
- MVP content pack:
  - 60 roguelite relics,
  - 20 ball variants,
  - 40 RPG gear items,
  - 36 skills,
  - 80 board templates,
  - 12 boss mechanics.
- Balance spreadsheets or data tables.
- Tuning notes.

**Interfaces**
- JSON/RON/TOML content files.
- Content linter compliance.
- No hard-coded gameplay constants.

**Dependency order**
- Starts once schemas exist.
- Final values depend on physics feel checkpoint.

**Validation**
- Content schema tests.
- Duplicate/invalid ID linting.
- Simulated reward-pool distribution.
- Manual build review.

---

## Workstream I — QA, Telemetry, and Playtest Analytics

**Subagent role:** QA automation / telemetry analyst.

**Inputs needed**
- Success criteria.
- Event schemas.
- Build pipeline.
- Replay system.

**Outputs**
- telemetry event schema,
- local playtest logger,
- replay tagging,
- metrics dashboard exports,
- bug triage templates,
- feel-test survey.

**Interfaces**

```rust
enum TelemetryEvent {
  ShotStarted,
  PegHit,
  BucketCatch,
  BoardWon,
  BoardLost,
  RelicChosen,
  SkillUsed,
  ReplayHash,
}
```

**Dependency order**
- Starts after initial event schemas.
- Deep analytics depend on playable vertical slice.

**Validation**
- Telemetry does not affect determinism.
- Replay logs anonymized.
- Metrics match gameplay state.
- Playtest forms produce actionable categories.

---

## Merge and integration strategy

### Checkpoint 0 — Contracts first

Merge shared schemas before features:
- `PhysicsEvent`
- `GameEvent`
- `BoardDefinition`
- `RunState`
- `CharacterState`
- `FeedbackEvent`
- content ID conventions.

No workstream may invent private duplicate schemas.

### Checkpoint 1 — Physics Feel Alpha

Required before mode balance:
- deterministic shot simulator,
- 10 hand-authored test boards,
- bucket catch working,
- score/free-ball events working,
- peg clear timing working,
- event playback VFX scene,
- replay hash tests passing.

Exit criteria:
- first-bounce prediction exact,
- no tunneling in stress suite,
- manual test group rates physics at least “promising,”
- no unresolved “ball feels wrong” blocker.

### Checkpoint 2 — Vertical Slice

One playable flow:
- start game,
- play one generated board,
- earn one relic,
- play one board with relic,
- use one RPG skill in a separate test board,
- trigger Extreme Fever,
- save replay.

### Checkpoint 3 — Mode Separation

- Roguelite and RPG use same physics crate.
- Each mode has separate save data and balance tables.
- Shared feedback layer consumes identical event types.

### Checkpoint 4 — Content Expansion

- Add content only after replay determinism, content lint, and board validation are stable.
- No content PR merges without automated schema validation and at least one replay/board smoke test.
