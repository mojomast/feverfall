# Artifact 1 — Research Findings Summary with Sources

## 1. Original Peggle physics, mechanics, and feel

**Key finding:** Solid documentation exists for Peggle’s behavioral feel and rules, but no official public source was found that discloses exact numeric gravity, restitution, or ball-speed constants. The original design research therefore supports a tunable physics spec rather than copying fixed hidden values.

PopCap’s own “Making of Peggle” account says Peggle began as a pachinko-inspired prototype with bare-bones pegs, balls, gravity, and realistic bouncing, but early versions were too chaotic and luck-based; the design challenge became balancing luck with skill. The team moved away from rapid-fire pachinko/pinball clutter toward a one-ball-at-a-time structure, disappearing hit pegs, target orange pegs, green power pegs, and finally the Extreme Fever celebration. Source: https://peggle.fandom.com/wiki/Peggle_Deluxe/Making_of_Peggle

The Peggle 2 manual, which describes the same core rules lineage, documents the canonical board structure: around 100 pegs, 25 orange pegs, 10 balls, ball launched from a top cannon, pegs lighting up on hit, lit pegs disappearing after the shot, a bottom free-ball bucket, orange-peg multiplier escalation, purple 50x peg, and green magic-power pegs. Source: https://dlassets-ssl.xboxlive.com/public/content/efd011ea-47b6-4b61-9e86-08063a3e1cfa/GameManual/be144115-0a2f-448a-abe8-855437167d8a/en-US/index.html

Peggle mechanics references also document free balls at 25,000 / 75,000 / 125,000 shot-score thresholds, the no-peg-shot coin flip, the free-ball bucket, and style shots such as Lucky Bounce and Free Ball Skillz. Source: https://peggle.fandom.com/wiki/Game_Mechanics

The US Peggle patent confirms the intended design axis: Peggle “balances skill and luck,” has a physics engine, and contemplates variations such as peg material/bounciness, gravity regions, board movement after launch, power-up storage, extra balls, jackpots, and different ball variants. It does **not** disclose exact gravity, restitution, or speed constants. Source: https://patents.google.com/patent/US8678904B1/en

For “lucky bounce” and DDA: no reliable official source was found proving hidden dynamic difficulty adjustment in original Peggle. Documented player-facing assistance includes the bucket, score-based free balls, a no-peg free-ball coin flip, magic powers such as Super Guide and Zen Ball, and community-documented “Microzen” behavior for low-ball first-playthrough states. Because that evidence is mostly mechanics-wiki-level rather than primary-source code, this project should use **explicit, inspectable assist systems** rather than hidden DDA. Source: https://peggle.fandom.com/wiki/Game_Mechanics

Peggle’s feel differs from generic pinball because it gives the player a single deliberate launch, then lets deterministic but hard-to-fully-predict geometry create drama. GameSpot’s Peggle 2 review describes the physics as natural enough that randomness does not feel unfair, while Wired reports that advanced players read Peggle as Euclidean causality even after many collisions, whereas casual players may perceive the same system as chaos. Source: https://www.gamespot.com/reviews/peggle-2-review/1900-6415588/

**Design basis:** Do not clone a black-box numeric physics profile. Build a deterministic, tunable ball-and-peg solver that produces predictable first collisions, rich late-shot chaos, no unfair tunneling, visible skill via bucket timing, and explicit assist mechanics.

---

## 2. Peggle-likes, clones, and competitors

| Reference | Public engine / physics info found | What it gets right | What falls short / lesson for this project |
|---|---:|---|---|
| **Peggle 2** | Public manual describes behavior; exact engine constants not disclosed. | Preserves 10-ball / 25-orange / bucket / magic-power structure and adds heavy audiovisual payoff; GameSpot calls out delicate peg tones, drum rolls, camera zoom near final orange, and music/color explosion. | Powers should change board interpretation without undermining the core launch-and-bounce skill. Peggle 2’s polish bar is the feel benchmark. |
| **Peglin** | Developer interview: Peglin is described as “Pachinko Roguelike” / “Pinball Roguelike,” a roguelike deckbuilder where collected pinballs have stats, effects, and physics properties; Peglin was originally influenced by Peggle and pinball/pachinko, and the team discussed switching future projects from Unity to GameMaker. | Best direct competitor: turns balls into “deck” pieces, relics alter rules, turn-based combat gives physics outcomes tactical meaning. Steam positions it as a Pachinko Roguelike with orbs, relics, and no-two-runs structure. | Peglin’s combat can pull attention toward damage math. Our project should keep **board clearing, bucket catches, combo routing, and peg ecology** central so it feels more like Peggle’s physical skill than only RPG damage conversion. |
| **Roundguard** | Official site: bouncy Peggle-inspired dungeon crawler roguelike with pinball physics, loot, and a randomized castle. Game Developer describes the player character as the launched ball, with enemies attacking while the player bounces semi-helplessly. | Strong lesson for RPG mode: physics uncertainty can coexist with classes, loot, enemies, and dungeon structure. | The “character is the ball” framing depends on character art; the early constraint forbids character art, so RPG systems must be expressed through pegs, launcher, ball, basket, UI, and primitive effects first. |
| **Ballionaire** | Steam / Raw Fury: physics-driven roguelike, pachinko board construction, kinetic chaos, combos, rule-bending boons, “autobonker” genre. | Excellent reference for player-built board synergies and escalating combo spectacle. | Full board-building can become an idle “numbers go up” game. The roguelite should allow limited board manipulation but preserve aimed shots and orange-clear objectives. |
| **Pegs X Stickers** | Steam/itch pitch: pachinko-roguelite combining Peggle/Balatro inspirations, board building, peg purchases, sticker synergies, physics scoring, rounds, and shop upgrades. | Shows market interest in Balatro-like modifiers mapped to peg-scoring systems; planned content emphasizes peg types, stickers, and board variants. | The lesson is not “copy Balatro”; it is to let modifiers visibly change the scoring grammar of the board. |
| **Luck be a Landlord** | Steam: roguelike deckbuilder using a slot machine to earn rent, explicitly no real-world currency gambling or microtransactions. | Strong reference for gambling-adjacent audiovisual language without real-money gambling. | Must be ethically careful: use slot-machine-like feedback for skill/reward clarity, not monetization pressure. |
| **Bounzy** | Official site: drag-and-release spells, bouncing attacks, monsters, wall defense, lab upgrades. | Mobile-accessible line-drawing/bounce skill with persistent upgrade hooks. | Its brick-breaker/defense loop is less about peg-clearing drama; useful mainly for simple RPG upgrade readability. |
| **Open-source Peggle clones** | Examples include Swift/SwiftUI clone with custom physics engine, browser/CS assignments with tick-based physics-engine separation, and clones exposing gravity, restitution, air drag, timestep, launch speed, ball radius, bucket speed, and guide-bounce tunables. | Confirms that the genre benefits from explicit tunables and isolated physics modules. | Clones often reproduce rules but lack Peggle-grade feel and payoff. The feedback layer must be treated as core, not polish. |

Primary/source links used in this section:
- Peggle 2 manual: https://dlassets-ssl.xboxlive.com/public/content/efd011ea-47b6-4b61-9e86-08063a3e1cfa/GameManual/be144115-0a2f-448a-abe8-855437167d8a/en-US/index.html
- GameSpot Peggle 2 review: https://www.gamespot.com/reviews/peggle-2-review/1900-6415588/
- Peglin developer interview: https://stage.gamemaker.io/fr/blog/peglin-developers-interview
- Peglin Steam: https://store.steampowered.com/app/1296610/Peglin/
- Roundguard official site: https://www.wonderbellygames.com/
- Ballionaire Steam: https://store.steampowered.com/app/2667120/Ballionaire/
- Pegs X Stickers Steam: https://store.steampowered.com/app/4403010/Pegs_X_Stickers/
- Luck be a Landlord Steam community: https://steamcommunity.com/app/1404850/
- Bounzy official site: https://www.bounzyga.me/
- Example open-source clone: https://github.com/ngmarcus/peggle-clone

---

## 3. 2D physics engine research for 2026

Box2D is a mature 2D rigid-body library with C17 portability, continuous collision detection, contact events, shape casts, filtering, a robust solver, and modern data-oriented/multithreaded features. Source: https://box2d.org/documentation/

Box2D’s author also emphasizes that deterministic simulation is brittle but valuable for repeatability, debugging, and CI replay tests; same-computer determinism is easier than cross-platform determinism. Source: https://box2d.org/posts/2024/08/determinism/

Fixed-timestep simulation is strongly supported by classic and modern engine guidance: Gaffer’s “Fix Your Timestep” recommends an accumulator with fixed `dt` and render interpolation, while Godot documentation explains that physics ticks are fixed separately from render frames and interpolation smooths the render result. Sources: https://www.gafferongames.com/post/fix_your_timestep/ and https://docs.godotengine.org/

Rust ecosystem options are viable in 2026. Avian is an ECS-driven Bevy physics engine with dynamic/kinematic/static bodies, CCD, material properties, collision events, raycasts/shapecasts, interpolation, and debug rendering. Source: https://github.com/avianphysics/avian

Rapier also has official Bevy integration and supports efficient 2D/3D physics simulation, including restitution and debug rendering. Source: https://docs.rs/bevy_rapier2d/latest/bevy_rapier2d/

Chipmunk2D remains a portable, MIT-licensed C physics library with bindings, but its ecosystem momentum is less aligned with a Rust/AI-agent codebase. Source: https://chipmunk-physics.net/release/Chipmunk-7.x/Chipmunk-7.0.1-Docs/

**Recommendation:** Use a custom deterministic 2D ball/peg solver for the core gameplay, wrapped in a pure Rust crate, with Bevy as the presentation/game framework. Use Avian or Rapier only for debug prototypes, non-core sensors, or tooling experiments. This is justified because the core collision set is simple—circles, line segments, bucket colliders, static obstacles—and determinism/replayability matter more than general rigid-body complexity.

---

## 4. Roguelite mechanics that transfer well

Peglin is the closest genre reference: collected orbs replace cards, relics reshape the run, and turn-based encounters make physics shots tactically meaningful. The developer interview explicitly says Peglin’s run variety comes from randomness in deckbuilding plus unpredictability in physics, and Steam describes the core as collecting special orbs, popping pegs to deal damage, and acquiring relics that change the game. Sources: https://stage.gamemaker.io/fr/blog/peglin-developers-interview and https://store.steampowered.com/app/1296610/Peglin/

Slay the Spire’s transferable lesson is **map/node structure + relic-driven run identity + metrics-driven balance**. Its public materials emphasize unique deck crafting, bizarre creatures, relics of immense power, and a GDC talk focused on metrics-driven design and balance. Sources: https://store.steampowered.com/app/646570/Slay_the_Spire/ and https://www.gdcvault.com/play/1025731/-Slay-the-Spire-Metrics

Vampire Survivors’ transferable lesson is **low-input, high-escalation snowballing**: its official page describes minimalistic roguelite survival where players gather gold in each run to buy upgrades for the next survivor; Steam notes that choices allow the player to quickly snowball against huge monster counts. Source: https://poncle.games/vampire-survivors.html

Balatro’s transferable lesson is **modifier taxonomy**: Jokers, Tarot cards, Planet cards, Spectral cards, Vouchers, deck modifiers, seeded runs, challenges, and difficulty stakes create combinatorial build identity from simple core rules. Source: https://store.steampowered.com/app/2379780/Balatro/

**Mechanics that transfer best to ball physics:** relics that alter peg behavior, ball properties, bucket behavior, scoring grammar, and board geometry; risk nodes that add hazards or extra orange pegs for better rewards; shops with peg/ball/relic offers; meta-unlocks that expand options more than raw permanent power.

---

## 5. RPG progression systems for arcade/puzzle cores

Puzzle Quest is the strongest historical example of RPG progression bolted onto a puzzle core. Its Steam page describes match-3 combat backed by an epic story, spells built from crystal matches, a heroic character built by winning battles, 150+ quests, cities/castles/companions, money, helms, weapons, armor, XP tiles, and skull tiles for damage. Source: https://store.steampowered.com/app/12500/PuzzleQuest_Challenge_of_the_Warlords/

StrategyWiki summarizes it as an RPG that uses puzzle combat for encounters. Source: https://strategywiki.org/wiki/Puzzle_Quest%3A_Challenge_of_the_Warlords

Puzzle & Dragons demonstrates cooldown-based active skills and passive leader skills layered over puzzle turns: active skills count down only when the player makes matches, then reset after use; leader skills passively boost stats or reduce damage and can stack. Source: https://pad.fandom.com/wiki/Monster_Skills

Gems of War shows how long-term account/world progression can be organized as kingdoms, resource tracks, prestige, tasks, and bonuses; its 9.2 update explicitly streamlined Kingdom progression into Kingdom XP / Crystals / Prestige / Bonuses. Source: https://gemsofwar.com/9-2-crystal-kingdom-a-deep-dive/

**Pitfalls to avoid:** do not let RPG stats erase the physics skill; do not make persistent power the only reason to replay boards; avoid invisible math that prevents players from understanding why a shot mattered. RPG mode should add objectives, skill expression, cooldown timing, gear identity, and campaign structure while preserving the same deterministic physics core.

---

## 6. Juice, slot-machine feedback, pachinko-style engagement

Slot-machine psychology research is directly relevant but must be translated ethically. Clark et al. found that near-misses can increase desire to continue gambling and recruit win-related circuitry, especially when the player feels personal control. Source: https://www.sciencedirect.com/science/article/pii/S0896627309000373

Dixon et al. show that “losses disguised as wins” occur when losing outcomes are celebrated with reinforcing sights and sounds; their study found that sound can cause players to miscategorize LDWs as wins, while negative/clearer feedback can unmask the loss. Source: https://lumsa.it/sites/default/files/pdf/DIXON_2015.pdf

More recent reviews complicate the near-miss story: near-misses can increase arousal/frustration and prolong behavior in some contexts, but reinforcing effects are mixed and stronger in problem-gambling populations or particular experimental designs. Source: https://www.frontiersin.org/journals/psychiatry/articles/10.3389/fpsyt.2024.1322631/full

Game-feel sources support layered feedback. Steve Swink’s *Game Feel* frames feel as a language of virtual sensation, and his polish chapter defines polish as artificial cues layered on top of simulation. Source: https://www.taylorfrancis.com/books/mono/10.1201/9781482267334/game-feel-steve-swink

“Juice it or Lose It” style references show that color, tweening, scaling, sound, particles, and screen shake can make a simple Breakout-like game feel dramatically more responsive without changing the core mechanics. Source: https://cobble.games/wise-inspiring-smart/game-design/juice-it-or-lose-it

**Ethical translation:** use slot-machine escalation for real mechanical wins only. Do not celebrate a bad shot as if it were a win. Near misses can get readable “almost” feedback, but never stronger feedback than true catches, orange clears, skill shots, or board clears.

---

## 7. 2026 tooling / engine choice for small-team and AI-agent development

Bevy 0.19 was released in June 2026 and is a Rust, free/open-source, ECS-based engine; Bevy’s docs emphasize parallel/cache-friendly ECS, systems/components, 2D rendering, and fast compile improvements. Source: https://bevy.org/news/bevy-0-19/

Bevy’s CLI and templates support web/Wasm and multi-platform build workflows. Source: https://thebevyflock.github.io/bevy_cli/cli/web.html

Godot 4.7 is also a strong small-team option: free/open-source, cross-platform, dedicated 2D engine, GDScript/C#/C++/GDExtension, and broad platform export support; godot-ci Docker images support automated export/deploy workflows. Source: https://godotengine.org/

**Recommendation:** For this specific project, choose **Rust + Bevy 0.19 + custom deterministic physics crate**. Godot is faster for visual iteration, but the team’s comfort with Rust and the need for deterministic seed/replay tests, AI-agent-friendly code review, pure-code contracts, modular crates, and strong CI favors Bevy/Rust.
