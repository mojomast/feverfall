---
name: Release checklist
about: Track a FeverFall release candidate
title: "[Release]: v"
labels: release
assignees: ""
---

## Candidate

- Version/tag:
- Target commit:

## Required checks

- [ ] Ubuntu CI is green
- [ ] Coverage workflow uploaded `lcov.info`
- [ ] Manual release workflow dry-run completed without publishing
- [ ] `v*` tag release attached Windows and macOS artifacts
- [ ] SHA-256 checksum files are attached for every artifact
- [ ] Release notes generated from `git-cliff` were reviewed

## Smoke commands

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo run -p feverfall_game -- --smoke-full`
