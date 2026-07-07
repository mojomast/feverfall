# Physics Fuzzing

FeverFall fuzzes `physics_core` from an isolated `fuzz/` cargo-fuzz workspace. It is intentionally **not** listed in the root `[workspace].members`, so normal `cargo test --workspace` and CI checks do not require nightly or libFuzzer tooling.

## Property tests

Run deterministic invariants on stable Rust:

```bash
cargo test -p physics_core
```

The proptest suite covers deterministic simulation, terminal `ShotEnded` events, unique peg-hit summaries, finite trajectory samples, and first-bounce prediction consistency.

## Fuzz targets

Install and use nightly cargo-fuzz:

```bash
cargo install cargo-fuzz
cargo +nightly fuzz run first_bounce_consistency
cargo +nightly fuzz run trajectory_sampling
cargo +nightly fuzz run multi_shot_replay
cargo +nightly fuzz run authored_board_corpus
```

Targets use bounded, valid-domain generated boards: finite dimensions, capped pegs/obstacles, finite launch angles/speeds, and capped multi-shot replay sequences.

## Reproduce and minimize

When libFuzzer finds a crash, reproduce from the isolated fuzz workspace:

```bash
cargo +nightly fuzz run first_bounce_consistency fuzz/artifacts/first_bounce_consistency/<crash-file>
```

Minimize the input before filing a bug or turning it into a regression test:

```bash
cargo +nightly fuzz tmin first_bounce_consistency fuzz/artifacts/first_bounce_consistency/<crash-file>
```

If the minimized case reveals a stable invariant violation, add a focused regression under `crates/physics_core/src/` tests and keep the corpus seed under `fuzz/corpus/<target>/` only when it adds coverage beyond the regression.
