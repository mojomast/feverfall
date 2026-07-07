# Rendering Technical Notes

Checkpoint 5 adds a Bevy ECS rendering pass for the production screen models.
The deterministic gameplay crates remain render-agnostic; screen systems adapt
models under `game/src/plugins/screens/` into tagged Bevy entities.

## Screen coverage

The render pass covers 13 screen states: main menu, settings, board/HUD with
relic bar, reward choice, node map, shop, forge, event, run summary/failure,
RPG chapter select, RPG gear, RPG skill tree, and campaign progress.

Each active state spawns exactly one `ScreenRoot`. Child entities are tagged as
panels, buttons, cards, reachable map nodes, relic icons, labels, or skill
nodes. Exiting a screen despawns all `ScreenRoot` entities.

## Assets and licensing

Generated runtime sprites live under `game/assets/sprites/` and are documented
in `game/assets/sprites/ATTRIBUTION.md`. Font licensing is tracked under
`game/assets/fonts/`; no unclear-license assets are bundled.

The Bevy feel-test window remains launched with:

```bash
cargo run -p feverfall_game --features bevy_feel_test
```

Render validation includes package tests plus feature-gated screen ECS tests.
