# Local Validation

Checkpoint 0 baseline commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p replay_runner
cargo run -p board_validator
cargo run -p content_linter
cargo run -p seed_browser
cargo run -p feverfall_game
```

Replay, board, content, seed, and app commands are placeholders until their corresponding workstreams add real gameplay validation.
