# Content Audit

Date: 2026-07-04

Result: pass. Current C4 content pack is lint-clean and all authored boards validate.

## Content Linter

Command: `cargo run -p content_linter`

Exact result line:

```text
content lint passed: 242 unique id(s)
```

Coverage observed:

- Full `game/assets/content` content tree.
- Top-level `content` tree when present.
- Board, relic, ball, shop, RPG gear, RPG skill, and balance-schema content paths recognized by the linter.

## Board Validator

Command: `cargo run -p board_validator`

Result: pass. Validator emitted 80 `PASS` lines and no failures.

Validated board groups:

- Existing boss/feel/RPG boards: `boards/act1_boss_01`, `boards/feel_*`, `boards/rpg_ch1_01` through `boards/rpg_ch1_05`.
- C4 roguelite Act 1 boards: `boards/c4_act1_normal_*`, `boards/c4_act1_elite_*`, and `boards/act1_boss_02` through `boards/act1_boss_04`.
- C4 roguelite Act 2 boards: `boards/c4_act2_normal_*`, `boards/c4_act2_elite_*`, and `boards/act2_boss_05` through `boards/act2_boss_08`.
- C4 roguelite Act 3 boards: `boards/c4_act3_normal_*`, `boards/c4_act3_elite_*`, and `boards/act3_boss_09` through `boards/act3_boss_12`.
- C4 RPG authored boards: `boards/c4_rpg_ch1_01` through `boards/c4_rpg_ch5_02`.

Sample exact output prefix:

```text
PASS boards/act1_boss_01
PASS boards/c4_act1_elite_05
PASS boards/c4_act1_elite_10
PASS boards/c4_act1_normal_01
```

Sample exact output suffix:

```text
PASS boards/rpg_ch1_01
PASS boards/rpg_ch1_02
PASS boards/rpg_ch1_03
PASS boards/rpg_ch1_04
PASS boards/rpg_ch1_05
```

Re-run note: `board_validator` loads authored boards from `board_gen::authored_boards_dir()`, so newly landed C4 boards are included automatically.
