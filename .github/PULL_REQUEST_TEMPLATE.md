## Summary

-

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo run -p feverfall_game -- --smoke-full`
- [ ] Content/replay/board gates if affected

## Release/CI impact

- [ ] No CI workflow changes
- [ ] CI workflow changes were syntax-checked with `actionlint` or documented as not available
- [ ] Release artifact, coverage, or changelog behavior is documented in `docs/release.md`

## Ownership check

- [ ] Changes stay within the assigned file ownership for this task/agent
