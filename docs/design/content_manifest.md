# C4 Content Manifest

## Counts

- Roguelite relics: 60 total.
- Relic category distribution: Ball 12, Peg 12, Basket 12, Board 12, EconomyCombo 12.
- Ball variants: 20 total.
- RPG gear: 40 total.
- Gear slot distribution: Launcher 10, CoreBall 10, BasketRig 10, Charm 10.
- RPG skills: 36 total.
- Skill tree distribution: Trickshot 9, Basket 9, Alchemy 9, Tactician 9.
- Authored board templates: 80 total.
- Boss boards with structured mechanics: 13 total boards using 12 mechanic kinds.

## Board Coverage

- Existing feel boards: 10.
- Existing RPG Chapter 1 boards: 5.
- Existing Act 1 boss board: 1.
- C4 roguelite boards: 38 across Acts 1-3.
- C4 boss boards: 11 across Acts 1-3.
- C4 RPG boards: 15 across Chapters 1-5.

## Boss Mechanics

- `ShieldPegs`: `boards/act1_boss_02`.
- `HealingPegs`: `boards/act1_boss_03`.
- `RotatingBlockers`: `boards/act1_boss_04`.
- `TimedHazardRows`: `boards/act1_boss_01`, `boards/act2_boss_05`.
- `SplitterStorm`: `boards/act2_boss_06`.
- `GravityPulse`: `boards/act2_boss_07`.
- `BasketLock`: `boards/act2_boss_08`.
- `GhostVeil`: `boards/act3_boss_09`.
- `BombCountdown`: `boards/act3_boss_10`.
- `MirrorWalls`: `boards/act3_boss_11`.
- `CurseTax`: `boards/act3_boss_12`.
- `FeverDrain`: `boards/rpg_ch1_05`.

## Validation

- `cargo run -p content_linter`: passes with 242 unique IDs.
- `cargo run -p board_validator`: passes all 80 authored boards.
