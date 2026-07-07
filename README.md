# FeverFall

FeverFall is a deterministic Peggle-style ball-drop Rust prototype with two progression modes: roguelite runs and a persistent RPG campaign. Checkpoint 4 is feature complete; Checkpoint 5 is underway to turn the prototype into a playable alpha candidate with stronger presentation, persistence safety, balance measurement, release automation, and repository hygiene.

## Project Status

- **Physics core:** deterministic fixed-step simulation, continuous collision handling, first-bounce prediction, bucket catches, replay hashes, trajectory sampling, and stress tests are implemented.
- **Roguelite mode:** C2 vertical slice, Acts 1-4 smoke coverage, relic trigger coverage, meta progression skeleton, and replay fixtures are present.
- **RPG mode:** Chapter 1 implementation smoke, Chapter 1/3/5 campaign smoke samples, mastery contracts, gear, skills, saves, and replay fixtures are present.
- **Content:** boards, boss boards, RPG boards, relics, balls, gear, skills, shops, and manifests are validated by content and board tools.
- **Runtime:** default CLI paths are non-interactive smoke gates. The optional Bevy feel-test scene is feature-gated and is the current local play path.

## Play Path

Run the default smoke path:

```bash
cargo run -p feverfall_game -- --smoke
```

Run the current desktop feel-test path:

```bash
cargo run -p feverfall_game --features bevy_feel_test -- --feel-test
```

Run the full game smoke gate:

```bash
cargo run -p feverfall_game -- --smoke-full
```

`--smoke-full` runs C2 vertical-slice smoke, RPG Chapter 1 and campaign Chapter 1/3/5 smoke, roguelite Acts 1-4 smoke, feel-test smoke, content lint, board validation, and golden replay fixtures.

## Developer Quick Start

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p feverfall_game -- --smoke-full
```

Useful validation tools:

```bash
cargo run -p content_linter
cargo run -p board_validator
cargo run -p replay_runner
```

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for contribution expectations and [`CHANGELOG.md`](./CHANGELOG.md) for release notes.

## Workspace Layout

- `crates/` — deterministic gameplay libraries: physics, rules, board generation, run mode, RPG mode, feedback events, telemetry, and content schema.
- `game/` — `feverfall_game` binary and optional Bevy desktop feel-test integration.
- `content/` — authored game data and balance/content manifests.
- `tools/` — validation and support tools: replay runner, board validator, content linter, seed browser, and balance simulator.
- `tests/` — workspace-level fixtures and validation inputs.
- `docs/` — design, QA, technical, playtesting, and agent planning documentation.
- `docs/agent/` — relocated autonomous-agent plans, prompts, and checkpoint scaffolding.

## Release Process

1. Run the developer quick-start gate and resolve failures.
2. Run the full smoke gate with `cargo run -p feverfall_game -- --smoke-full`.
3. Review [`CHANGELOG.md`](./CHANGELOG.md) and update `Unreleased` entries for player-facing, release-facing, or repository-facing changes.
4. For Windows artifacts, use the existing GitHub Actions Windows release workflow by manual dispatch or by pushing a version tag such as `v0.1.0`.
5. Attach generated artifacts and checksums to the release when the workflow succeeds.

Release automation is still expanding during Checkpoint 5; macOS publishing, coverage baselines, and richer changelog automation are tracked by the C5 CI workstream.

## Known Gaps

- Web/WASM build and release automation is not implemented.
- Production rendering and audio are Checkpoint 5 workstreams; runtime presentation remains placeholder until those land.
- Human feel/comprehension benchmarks, large balance cohorts, and long-running fuzz campaigns are not yet mandatory release gates.
- Code signing/notarization is not configured for desktop releases.

## License Status

The Cargo workspace currently declares `UNLICENSED`, and this repository does not yet include a root `LICENSE` file. Treat all source, content, and assets as not licensed for redistribution until a project license and asset provenance policy are finalized.
