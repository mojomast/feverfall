# Release Checklist

Complete every item before tagging a public build.

- Confirm `rust-toolchain.toml` and CI use the intended Rust toolchain.
- Run `cargo fmt --all -- --check`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.
- Run `cargo test --workspace`.
- Run `cargo run -p feverfall_game -- --smoke-full` and verify it exits 0.
- Confirm `--smoke-full` prints C2, RPG Chapter 1, RPG campaign Chapter 1/3/5, roguelite Acts 1-4, feel-test, content lint, board validation, all replay hashes, and a final PASS summary.
- Run or confirm CI has run all golden replay fixtures under `tests/golden_replays`.
- Run or confirm CI has run `cargo run -p content_linter` and `cargo run -p board_validator`.
- Confirm current golden hashes in docs match accepted replay output.
- Confirm release notes list known gaps, especially web-build status and placeholder asset status.
- Trigger `.github/workflows/windows-feel-test.yml` from the release ref after the C4 workflow changes are pushed.
- Confirm the Windows workflow runs `cargo run -p feverfall_game -- --smoke-full` before building.
- Download and smoke the uploaded Windows artifact, or record why human artifact verification was skipped.
- Record the Windows artifact name, workflow run URL/ID, and SHA-256 checksum.
- Ensure no generated saves, local logs, or target artifacts are staged.
- Inspect `git status`, `git diff`, and recent commits before tagging.
- Tag only after validation, artifact upload, and release notes are complete.
