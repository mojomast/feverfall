# Feverfall

Feverfall is a deterministic Peggle-style ball-drop prototype with two progression modes: a roguelite run mode and a persistent RPG campaign mode. Checkpoint 4 is feature complete, with physics, board validation, replay hashing, content linting, RPG campaign coverage, roguelite Acts 1-4 smoke coverage, and placeholder UI/VFX/audio systems under automated gates.

## Current Status

- Physics core: deterministic fixed-step simulation, continuous collision handling, first-bounce prediction, bucket catches, replay hashes, trajectory sampling, and stress tests are implemented.
- Roguelite: C2 vertical slice, C3 Acts 1-3 structure, C4 optional Act 4 final-seed smoke path, relic trigger coverage, meta progression skeleton, and replay fixtures are present.
- RPG: Chapter 1 implementation smoke, C4 campaign catalog samples for Chapters 1, 3, and 5, mastery-mode unlock/normalized mastery contracts, gear/skills, save/load, and replay fixtures are present.
- Content: authored boards, boss boards, C4 board packs, RPG boards, relics, balls, gear, skills, shops, and balance/content manifests are validated by `content_linter` and `board_validator`.
- Runtime: default CLI smoke is non-interactive. The optional Bevy feel-test scene is feature-gated.

## Build

Desktop smoke build:

```bash
cargo run -p feverfall_game -- --smoke
```

Full release smoke gate:

```bash
cargo run -p feverfall_game -- --smoke-full
```

Playable Bevy feel-test desktop build:

```bash
cargo run -p feverfall_game --features bevy_feel_test -- --feel-test
```

Web build status: no dedicated WebAssembly build pipeline is currently wired. Treat web as a known gap until a Bevy/WASM target, asset packaging, and browser smoke gate are added.

## Validation

Run the full workspace gate before tagging a public build:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p feverfall_game -- --smoke-full
```

`--smoke-full` runs C2 vertical-slice smoke, RPG Chapter 1 and campaign Chapter 1/3/5 smoke, roguelite Acts 1-4 smoke, feel-test smoke, full content lint, board validation, and every golden replay fixture.

## Known Gaps

- Public Windows Checkpoint 4 artifact requires the GitHub workflow to run from a pushed ref containing the C4 `--smoke-full` workflow update.
- Web build/release automation is not implemented.
- Runtime visuals/audio remain placeholder systems until production assets are added.
