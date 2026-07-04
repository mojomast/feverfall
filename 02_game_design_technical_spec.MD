# Artifact 2 — Game Design & Technical Specification Document

## Working title

**Feverfall**  
Peggle-style ball-drop arcade physics with two major modes: roguelite runs and persistent RPG campaign progression.

## Design pillars

1. **Readable chaos:** the first collision should feel planned; later collisions should feel surprising but not unfair. This follows Peggle’s documented balance of skill/luck and the review/player-analysis finding that expert players perceive causality inside apparent chaos. Source: https://patents.google.com/patent/US8678904B1/en
2. **Physics first, RPG second:** progression modifies the board/ball/basket grammar but never replaces aiming, bounce prediction, and bucket timing. Puzzle Quest and Peglin prove RPG/roguelite systems can sit on a puzzle/physics core, but the differentiator is preserving physical shot mastery. Sources: https://store.steampowered.com/app/12500/PuzzleQuest_Challenge_of_the_Warlords/ and https://store.steampowered.com/app/1296610/Peglin/
3. **Ethical dopamine:** strong audiovisual escalation is reserved for true mechanical value: peg hits, orange clears, combo thresholds, free balls, skill shots, relic triggers, and board clears. This avoids the LDW problem identified in slot-machine research. Source: https://lumsa.it/sites/default/files/pdf/DIXON_2015.pdf
4. **Minimal assets, maximum feel:** early development uses circles, capsules, rectangles, gradients, particles, text, and procedural audio only.

---

## Core physics specification

### Coordinate system

Use a normalized world-space board:

| Parameter | Initial value | Tuning range |
|---|---:|---:|
| Board width | `20.0 wu` | locked |
| Board height | `35.56 wu` | locked for 16:9 |
| Cannon position | `(10.0, 1.5)` | per board |
| Bottom kill plane | `y = 36.5` | per board |
| Physics timestep | `1 / 120 s` | `1/60` to `1/240` |
| Render interpolation | enabled | required |

Fixed timestep and interpolation are required because physics should not vary with render framerate. Source: https://www.gafferongames.com/post/fix_your_timestep/

### Ball

| Parameter | Initial value | Tuning range |
|---|---:|---:|
| Ball radius | `0.22 wu` | `0.18–0.30` |
| Ball mass | `1.0` | mostly cosmetic |
| Launch speed | `17.5 wu/s` | `14–22` |
| Gravity | `(0, 22.0 wu/s²)` | `16–30` |
| Air damping | `0.9985` per tick | `0.995–1.0` |
| Tangential damping on collision | `0.995` | `0.985–1.0` |
| Min active speed | `0.25 wu/s` | `0.1–0.5` |
| Max speed cap | `38 wu/s` | `28–50` |

Ball rotation/spin is disabled for MVP. The genre’s early feel target is Peggle-like ballistic readability, not pinball flipper simulation.

### Pegs

Peg types are primitive shapes only:

| Peg type | Shape | Collision | Clear behavior |
|---|---|---|---|
| Blue | circle or capsule | solid until shot end | score only |
| Orange | circle or capsule | solid until shot end | win-condition peg |
| Purple | circle/capsule | solid until shot end | 50x value, relocates each shot |
| Green | circle/capsule | solid until shot end | activates power / relic event |
| Stone | circle/capsule/rect | solid permanent | obstacle |
| Ghost | circle/capsule | sensor only | hit event, no bounce |
| Bomb | circle | solid | explosion event after shot |
| Splitter | capsule/triangle proxy | solid | spawns child ball on contact |

Peggle and Peggle 2 document blue/orange/purple/green peg semantics and lit pegs disappearing after each shot. Source: https://dlassets-ssl.xboxlive.com/public/content/efd011ea-47b6-4b61-9e86-08063a3e1cfa/GameManual/be144115-0a2f-448a-abe8-855437167d8a/en-US/index.html

### Collision rules

Use continuous collision detection for moving ball vs static circles/capsules/segments/bucket colliders:

1. Integrate velocity with gravity and damping for fixed `dt`.
2. Sweep ball along segment from previous position to proposed position.
3. Find earliest time of impact among colliders.
4. Move to contact point minus slop.
5. Reflect velocity using:

```text
v' = v - (1 + restitution) * dot(v, n) * n
```

6. Apply tangential damping.
7. Emit collision event.
8. Continue remaining timestep, max 4 collision iterations per tick.
9. If unresolved after 4 collisions, move to safe position and damp speed by 15%.

| Material | Restitution initial | Tuning range |
|---|---:|---:|
| Peg | `0.94` | `0.88–1.02` |
| Wall | `0.91` | `0.82–1.0` |
| Bucket rim | `1.02` | `0.95–1.10` |
| Stone obstacle | `0.78` | `0.6–0.95` |
| Rubber obstacle | `1.08` | `1.0–1.2` |

The patent documents peg material/bounciness as a contemplated axis, while open-source clones expose gravity/restitution/air drag/timestep/launch-speed tunables; therefore all physical constants are first-class data. Source: https://patents.google.com/patent/US8678904B1/en

### Peg clear timing

On first contact during a shot:

1. Peg enters `Lit` state immediately.
2. It continues colliding for the remainder of the active shot.
3. At shot end, lit clearable pegs dissolve and are removed.
4. Special relics may override this: immediate vanish, delayed explosion, ghost conversion, duplicate peg, or “echo hit.”

This matches Peggle 2’s documented “lit pegs disappear after the shot” behavior. Source: https://dlassets-ssl.xboxlive.com/public/content/efd011ea-47b6-4b61-9e86-08063a3e1cfa/GameManual/be144115-0a2f-448a-abe8-855437167d8a/en-US/index.html

### Basket / free-ball bucket

| Parameter | Initial value | Tuning range |
|---|---:|---:|
| Basket width | `3.0 wu` | `2.2–4.4` |
| Basket height | `0.55 wu` | `0.3–0.8` |
| Basket y | `34.4` | per board |
| Basket horizontal speed | `6.0 wu/s` | `3.5–9.0` |
| Basket motion | ping-pong linear | sine / scripted |
| Catch sensor margin | `0.18 wu` | `0.05–0.3` |

A ball caught by the basket grants one shot back, emits `BallCaught`, ends that ball, and triggers catch feedback. Bucket catches are core because Peggle’s free-ball bucket is both a resource mechanic and a skill expression layer. Source: https://dlassets-ssl.xboxlive.com/public/content/efd011ea-47b6-4b61-9e86-08063a3e1cfa/GameManual/be144115-0a2f-448a-abe8-855437167d8a/en-US/index.html

### Assist rules

Assist systems are explicit and mode-visible:

| Assist | Default | Rule |
|---|---:|---|
| Aim line | Always first bounce | Shows ray until first collision. |
| Prediction dots | Upgrade-gated | Shows simulated path for `0.8–2.5 s`. |
| Micro-adjust | Always | Keyboard/controller slow aim. |
| Mercy free ball | Optional accessibility | If no peg hit, weighted coin-flip chance; surfaced in UI. |
| Zen reroute | Power / RPG skill | Simulates candidate offsets and offers one improved path. |

This avoids unproven hidden DDA while honoring documented Peggle assist-style mechanics such as Super Guide/Zen Ball/no-peg free ball. Source: https://peggle.fandom.com/wiki/Game_Mechanics

---

## Scoring and combo rules

Base values:

| Event | Points |
|---|---:|
| Blue peg | 100 |
| Orange peg | 1,000 |
| Purple peg | 5,000 |
| Green peg | 500 |
| Bucket catch | free shot + 2,500 |
| Long Shot | 25,000 |
| Lucky Bounce | 25,000 |
| Free Ball Skill | 5,000 |
| Board clear | Fever buckets + remaining-ball bonus |

Shot score:

```text
shot_score =
  (sum peg values + style bonuses not marked excluded)
  * current_fever_multiplier
  * combo_multiplier
```

Fever multiplier steps:

| Orange pegs remaining | Multiplier |
|---:|---:|
| 25–16 | x1 |
| 15–10 | x2 |
| 9–5 | x3 |
| 4–2 | x5 |
| 1–0 | x10 |

Peggle uses escalating score multipliers as orange pegs are cleared and score-based free balls, so this keeps the known reward grammar while leaving exact thresholds tunable. Source: https://dlassets-ssl.xboxlive.com/public/content/efd011ea-47b6-4b61-9e86-08063a3e1cfa/GameManual/be144115-0a2f-448a-abe8-855437167d8a/en-US/index.html

---

## Roguelite Mode specification

### Run structure

A run contains 3 acts plus optional Act 4:

| Act | Boards | Boss | Theme |
|---|---:|---|---|
| Act 1 | 6 normal + 1 elite + boss | Tutorial boss | Core pegs |
| Act 2 | 7 normal + 2 elites + boss | Hazard boss | Moving obstacles |
| Act 3 | 8 normal + 2 elites + boss | Combo boss | Relic pressure |
| Act 4 optional | 4 high-risk boards + final boss | Final | Seed mastery |

Peglin’s Act 4 / final boss expansion in 2026 shows that act-based growth remains relevant for this direct competitor/reference. Source: https://www.pcgamer.com/games/roguelike/nearly-2-years-after-hitting-1-0-goblin-pachinko-roguelike-peglin-adds-a-fourth-act-and-a-final-boss-its-been-a-long-time-coming/

### Node map

Nodes are generated like a Slay-the-Spire-style path map:

| Node | Function |
|---|---|
| Board | Standard clear objective |
| Elite board | Hard modifiers, rare relic |
| Shop | Buy relics, balls, rerolls, peg edits |
| Event | Trade resource for risk/reward |
| Forge | Upgrade one ball or relic |
| Camp | Heal run resource or preview next boards |
| Boss | Scripted board mechanic |

Slay the Spire’s lesson is not cards specifically; it is pathing, relic identity, escalating difficulty, and metrics-driven balance. Source: https://www.gdcvault.com/play/1025731/-Slay-the-Spire-Metrics

### Run resources

| Resource | Role |
|---|---|
| Balls | Per-board shot count |
| Hearts | Run health; failed boards cost hearts |
| Coins | Shop/forge currency |
| Sparks | Temporary combo energy, spent on active relics |
| Keys | Optional boss/Act 4 access |
| Curse | Adds risk modifiers for better rewards |

### Upgrade and relic taxonomy

Relics modify one of five systems:

1. **Ball relics:** split, heavy, rubber, ghost, pierce, magnet, fireball, freeze ball.
2. **Peg relics:** orange pegs duplicate value, blue pegs chain lightning, green pegs recharge, purple pegs persist.
3. **Basket relics:** wider bucket, moving magnet, catch streak bonuses, bucket ricochet.
4. **Board relics:** remove stone pegs, rotate obstacle rows, spawn side bumpers, reveal predicted landing zone.
5. **Economy/combo relics:** interest, multiplier carryover, shop discounts, combo thresholds, risk payouts.

Peglin and Balatro show why modifiers should create run identity, not just add flat numbers; Ballionaire/Pegs X Stickers show peg-board synergies as a strong current market pattern. Sources: https://stage.gamemaker.io/fr/blog/peglin-developers-interview and https://store.steampowered.com/app/2379780/Balatro/

### Procedural board generation

Boards are generated from templates plus constraints:

1. Select board archetype: `fan`, `wave`, `clusters`, `lanes`, `spiral`, `rings`, `fortress`, `boss`.
2. Place required orange pegs using spacing and accessibility rules.
3. Place blue filler via Poisson-disk sampling or authored splines.
4. Place special pegs according to act budget.
5. Place obstacles.
6. Simulate 512–2,048 sampled aim angles.
7. Reject seed if:
   - no orange reachable in first collision set,
   - skilled bot clear probability below target,
   - bucket catch opportunities below minimum,
   - excessive dead zones,
   - ball stalls longer than allowed.

### Difficulty escalation

| Lever | Act 1 | Act 2 | Act 3 | Act 4 |
|---|---:|---:|---:|---:|
| Orange pegs | 20–25 | 25–30 | 28–35 | 35+ |
| Starting balls | 10 | 9–10 | 8–9 | 7–8 |
| Hazards | rare | moderate | high | high |
| Moving pegs | rare | moderate | high | scripted |
| Required score | low | medium | high | high |
| Curse offers | optional | common | common | frequent |

### Meta-progression

Meta-progression expands the possibility space without making early boards trivial:

| Unlock type | Examples |
|---|---|
| New ball families | Splitter, Ghost, Bomb, Heavy, Orbit |
| New relic pools | Basket, elemental, economy, risk |
| New board biomes | Clockwork, Crystal, Void, Garden |
| Starter kits | 1 starter ball + 1 minor relic |
| Difficulty tiers | Fever Heat 1–20 |
| Cosmetics | Ball trails, peg skins, UI frames |

Vampire Survivors supports the idea of gold/meta-upgrades between runs, but this project should bias toward unlock breadth rather than permanent raw power. Source: https://poncle.games/vampire-survivors.html

---

## RPG Mode specification

### RPG premise

RPG Mode uses the same physics core but replaces run reset with authored campaign progression. The player is represented mechanically by their **Launcher**, **Core Ball**, **Basket**, **Charms**, and **Skill Tree**, not by character art.

### Campaign structure

| Chapter | Content |
|---|---|
| Prologue | 5 tutorial boards, no gear complexity |
| Chapter 1 | 12 boards, introduces blue/orange/purple/green economy |
| Chapter 2 | 15 boards, introduces obstacles and active skills |
| Chapter 3 | 15 boards, introduces gear sets and boss boards |
| Chapter 4 | 18 boards, introduces multi-objective boards |
| Endgame | challenge towers, normalized-stat mastery boards, remix bosses |

Puzzle Quest supports the model of an epic campaign and quests layered over puzzle combat; this project maps that model to peg physics. Source: https://store.steampowered.com/app/12500/PuzzleQuest_Challenge_of_the_Warlords/

### Character stats

Stats alter readable affordances more than raw outcomes:

| Stat | Effect | Cap philosophy |
|---|---|---|
| Aim | Longer prediction line, slower aim drift | capped so ricochet mastery still matters |
| Control | Wider perfect-launch timing and basket micro-control | no auto-catch |
| Power | Wider launch-speed band for special balls | no board-breaking speeds |
| Resonance | Combo decay slower, combo multiplier grows faster | capped by event budget |
| Luck | Better reward choices and rerolls | no hidden physics bias |
| Focus | Active skill cooldown reduction | cooldown floor |

Puzzle & Dragons’ active-skill cooldown structure supports a turn/shot-based RPG skill layer; cooldowns should tick on meaningful shots, not passive time. Source: https://pad.fandom.com/wiki/Monster_Skills

### Gear slots

| Slot | Examples |
|---|---|
| Launcher | steady aim, charged shot, split-cannon, bank-shot sight |
| Core Ball | heavy core, glass core, ember core, ghost core |
| Basket | wide basket, magnet basket, ricochet basket |
| Charm | purple tracker, green amplifier, shop discount |
| Relic | campaign-limited rule changer |

Gear should be horizontal first: new verbs, new timing, new synergies. Numeric bonuses must be soft-capped.

### Skill trees

**Trickshot Tree**
- +prediction duration
- bank-shot preview
- one reroute per board
- long-shot multiplier

**Basket Tree**
- catch-streak bonus
- temporary basket widen
- bucket rim bounce boost
- near-catch slow-motion preview

**Alchemy Tree**
- peg transformation
- fire/ice/lightning peg reactions
- bomb peg crafting
- green-peg cooldown restore

**Tactician Tree**
- preview orange clusters
- swap two pegs before launch
- freeze moving obstacle
- convert one blue to purple

### RPG fail/win rules

- Board failure costs stamina/durability but does not reset campaign.
- Boss boards have scripted mechanics: shield pegs, healing pegs, rotating blockers, timed hazard rows.
- Mastery versions normalize gear and stats so leaderboards test physics skill.
- XP is awarded for objectives: clear board, catch streak, long shot, all-clear, low-ball clear, no-assist clear.

### RPG pitfalls and safeguards

| Pitfall | Safeguard |
|---|---|
| Stats trivialize aim | Aim stats extend preview, not accuracy. |
| Gear becomes spreadsheet-only | Every gear item must create visible peg/ball/basket behavior. |
| Campaign boards become solved chores | Add challenge modifiers and mastery medals. |
| Persistent power invalidates roguelite balance | Separate save profiles and balance tables. |
| Hidden luck feels unfair | Luck affects reward offers, never collision outcomes. |

---

## Reactive feedback / juice specification

### Feedback principles

1. **Every mechanical event gets immediate sensory acknowledgment.**
2. **Escalation maps to real value.**
3. **Near misses are readable, not celebratory.**
4. **Slot-style suspense is used for skill outcomes, not monetization.**
5. **Primitive shapes can still produce premium feel through timing, color, scale, particles, camera, and audio.**

This follows game-feel research on polish as artificial cues layered over simulation and slot-machine research showing that celebratory sights/sounds can alter reward perception. Sources: https://www.taylorfrancis.com/chapters/mono/10.1201/9781482267334-17/polish-metrics-steve-swink and https://lumsa.it/sites/default/files/pdf/DIXON_2015.pdf

### Trigger map

| Gameplay event | VFX | SFX | Camera/UI |
|---|---|---|---|
| Ball launch | muzzle flash, aim-line collapse | soft air cannon | cannon recoil |
| Blue peg hit | 80 ms flash, 1.12x scale pulse, 8 particles | pitched plink | +score floater |
| Orange peg hit | larger burst, orange ring, meter spark | brighter plink + thump | fever meter tick, micro shake |
| Purple peg hit | starburst, score beam to HUD | shimmer + bass accent | score multiplier pulse |
| Green peg hit | rune ring, power icon charge | rising arpeggio | power banner |
| Combo 3/6/10/15 | color ramp, particle count increase | pitch ladder + percussion layer | combo rail appears |
| Long Shot | streak trail replay line | whoosh + bell | “LONG SHOT” kinetic text |
| Lucky Bounce | bucket-edge spark + delayed burst | rim ding + catch chord | text appears only after catch |
| Near bucket miss | small grey whoosh, no coin burst | soft downbeat | “near” marker optional, muted |
| Bucket catch | splash ring, magnet snap, free-ball icon | resolving chord | basket glow, +1 ball animation |
| Last orange in flight | subtle zoom, audio low-pass | tension riser | final-orange spotlight |
| Extreme Fever | freeze-frame, silence, explosion, fireworks, bottom buckets | Ode-like original victory sting, not copied melody | staged celebration, skippable after 1.2 s |

Peggle’s Extreme Fever worked because the final peg became a defining audiovisual moment; PopCap refined fireworks, rainbow, and Ode-to-Joy-style triumph after testers reacted strongly. Source: https://peggle.fandom.com/wiki/Peggle_Deluxe/Making_of_Peggle

### Audio layering

Use five buses:

1. **Collision plinks**
2. **Combo percussion**
3. **Reward stingers**
4. **Ambient board hum**
5. **UI confirmations**

Rules:
- Peg-hit pitch rises with combo, capped after 15 hits.
- No more than 12 simultaneous peg-hit samples; additional hits merge into chord clusters.
- True rewards use major/resolved intervals; misses use neutral/short sounds.
- LDW safeguard: failed boards cannot trigger victory-like stingers.

### Visual budget

For early primitive art:

| Element | Implementation |
|---|---|
| Pegs | flat circles/capsules with emissive rim |
| Obstacles | rectangles/capsules with material pattern |
| Basket | capsule cup + rim sparks |
| Particles | CPU/GPU circle sprites |
| Trails | polyline ribbon |
| Text | bitmap/vector font |
| Background | single gradient + subtle grid |
| Fever | procedural confetti, radial lines, screen-space bloom |

---

## Technical architecture

### Engine choice

**Rust + Bevy 0.19 + custom deterministic physics crate.**

Rationale:
- Bevy 0.19 is current as of June 2026, Rust-native, ECS-based, open-source, parallel/cache-friendly, and supports 2D rendering. Source: https://bevy.org/news/bevy-0-19/
- Bevy supports web/Wasm-oriented build tooling and multi-platform templates. Source: https://thebevyflock.github.io/bevy_cli/cli/web.html
- A custom pure Rust physics crate makes seed/replay CI easier than relying on a general engine solver.
- Box2D/Avian/Rapier remain references and optional tooling, but core gameplay must be deterministic and testable. Source: https://box2d.org/posts/2024/08/determinism/

### Project structure

```text
feverfall/
  Cargo.toml
  crates/
    physics_core/        # pure deterministic ball/peg/bucket simulation
    game_rules/          # scoring, objectives, relic effects
    board_gen/           # seeded procedural boards and validators
    run_mode/            # roguelite run state, map, shops, relic rewards
    rpg_mode/            # campaign, stats, gear, skills, saves
    feedback_events/     # typed event definitions and intensity mapping
    content_schema/      # serde schemas, validation, migrations
  game/
    src/
      main.rs
      plugins/
        render/
        input/
        ui/
        audio/
        vfx/
        debug/
    assets/
      audio_placeholder/
      fonts/
      shaders/
      content/
        boards/
        relics/
        balls/
        skills/
        encounters/
  tools/
    replay_runner/
    board_validator/
    content_linter/
    seed_browser/
  tests/
    golden_replays/
    property_tests/
    perf_tests/
  docs/
    design/
    technical/
    devplan/
```

### Determinism and replay

Each replay stores:

```text
ruleset_version
content_manifest_hash
physics_version
seed
mode
board_id
initial_state_hash
input_commands:
  - aim_angle
  - launch_tick
  - selected_ball
  - active_skill_use
  - shop_choice
per_shot_hashes
final_state_hash
```

CI runs golden replays on every physics/content/rules change. Any hash divergence requires either a ruleset version bump or a bug fix. This follows the research finding that determinism is valuable but must be actively tested. Source: https://box2d.org/posts/2024/08/determinism/

### Minimal early-asset plan

No character art, environments, or bespoke illustration in Phase 1 implementation.

Build only:
- pegs,
- rectangular/capsule obstacles,
- ball variants,
- basket,
- cannon,
- UI panels,
- score text,
- particles,
- procedural trails,
- debug overlays.

---

## Success criteria / feel benchmarks

### Physics benchmarks

| Criterion | Target |
|---|---:|
| Replay determinism | 100% same hash on same platform/config |
| Tunneling failures | 0 in 10,000 max-speed collision tests |
| Stuck-ball events | <0.1% of shots; auto-resolve within 2.0 s |
| First-bounce prediction accuracy | exact within solver tolerance |
| Sim performance | 120 Hz sim under 0.25 ms for normal board |
| Board generator rejection | rejects unplayable seeds before player sees them |

### Feel benchmarks

| Criterion | Target |
|---|---:|
| Collision-to-feedback latency | <50 ms |
| New-player comprehension | 80% explain orange/bucket/aim after 5 boards |
| “Physics felt unfair” survey | <15% agree |
| “One more shot” desire | 70%+ agree after 15-minute test |
| Bucket catch satisfaction | 75%+ rate satisfying |
| Board clear payoff | 80%+ rate “exciting,” not annoying |
| Feedback clarity | 90% distinguish win, near miss, loss |

### Roguelite benchmarks

| Criterion | Target |
|---|---:|
| Act 1 completion | 45–65% for new players after tutorial |
| Full-run win rate | 10–20% before meta unlocks |
| Distinct build identity | players name their main relic/ball combo in post-run survey |
| Bad-seed reports | <1% of runs |

### RPG benchmarks

| Criterion | Target |
|---|---:|
| Campaign board replay | 30%+ replay for mastery medals |
| Skill usage | active skills used in 70%+ eligible boards |
| Gear readability | 80% explain what equipped item changed |
| Stat dominance | no stat line improves win rate >20% alone in normalized tests |
