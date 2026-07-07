# FeverFall Release Process

FeverFall keeps pull request CI Ubuntu-focused and reserves cross-platform binaries for manual release dry-runs and `v*` tags.

## CI smoke commands

Run these locally before tagging when possible:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p feverfall_game -- --smoke-full
cargo run -p replay_runner
cargo run -p board_validator
cargo run -p content_linter
```

## Coverage baseline

The **Coverage** workflow runs on Ubuntu and manual dispatch. It installs `cargo-llvm-cov` and uploads `lcov.info` as a workflow artifact. C5 establishes a baseline only; coverage thresholds are intentionally deferred.

Manual equivalent:

```sh
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

## Release dry-run

Use **Release Builds** from the Actions tab on any branch with `upload_release=false`. This builds:

- Windows x86_64 on `windows-latest`
- macOS arm64 on `macos-15`
- macOS x86_64 on `macos-15-intel`

The dry-run uploads archives and `.sha256` files as workflow artifacts but does not publish a GitHub Release unless the ref is a `v*` tag and `upload_release=true`.

## Tagged release

1. Confirm Ubuntu CI and coverage are green.
2. Create and push an annotated tag named `vMAJOR.MINOR.PATCH`.
3. The **Release Builds** workflow builds Windows/macOS artifacts, writes SHA-256 checksum files, and publishes a GitHub Release from the final Ubuntu publish job with `contents: write`.
4. Review generated notes and attached files before announcing the release.

## Changelog notes

`git-cliff` is configured by `cliff.toml`. Release publishing uses it for tag-scoped notes once commit hygiene is adequate. Locally, preview notes with:

```sh
git cliff --current --strip header
```

## Workflow validation

If available, validate workflow syntax with:

```sh
actionlint .github/workflows/*.yml
```

If `actionlint` is not installed, rely on GitHub Actions dry-run/manual dispatch for platform validation and record that limitation in the release checklist.
