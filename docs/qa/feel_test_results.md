# Feel Test Results

Date: 2026-07-04

Tester: `[C4-QA] automated proxy`

Protocol source: `docs/playtesting/feel_survey.md`

Input source: `cargo run -p feverfall_game -- --smoke` and `cargo run -p feverfall_game --features bevy_feel_test -- --smoke`

Scope note: This is not a human 10-15 minute playtest. It is a deterministic proxy pass against smoke output and event counts. Human-only comprehension/desire benchmarks remain open until actual playtest responses are collected.

## Session Metadata

- Build ID or commit: uncommitted C4 worktree, QA validation run on 2026-07-04.
- Board IDs played or sampled by smoke: `boards/feel_fortress_stone_01`, `boards/act1_boss_01`, `boards/rpg_ch1_01`, `boards/rpg_ch3_01`, `boards/rpg_ch5_mastery_01`, `boards/rpg_ch1_05`.
- Replay hashes captured: C2 run summary `18202124e6b686d8`; feel-test replay `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`; RPG Ch1 `c18385eaa33af638`; RPG Ch3 `01efbd0f270af2e8`; RPG Ch5 `ef2ae2140c5abcdf`; RPG campaign `04029810211125c5`; RPG Ch1 preserved summary `3364e243ba2065f4`; roguelite Act 1-3 summary `e72374145338c3b3`; roguelite full-run summary `152fc850303d8356`.
- Shot indices reviewed: smoke shot summaries only; feel-test smoke reports `shots=1` on `boards/act1_boss_01`, C2 vertical slice reports `shots=3`.
- Final score/progression outcome: C2 smoke `BoardWon`, feel-test smoke `Continue`, RPG campaign `all_5` completion/mastery smoke, roguelite full-run Act 4 smoke complete.
- Input device: automated scripted input.
- Prior Peggle/Peglin experience: not applicable for automated proxy.

## Rating Proxy Answers

Scale: 1 strongly disagree to 5 strongly agree.

| Criterion | Proxy rating | Evidence | Result |
|---|---:|---|---|
| I understood where the first bounce would go before firing. | 5 | Smoke reports `first_bounce=true`; workspace tests include first-bounce prediction exactness. | Pass proxy |
| The physics felt fair, even when later bounces surprised me. | 4 | Golden replay hashes match; no tunneling/stress physics tests pass; smoke has deterministic replay hashes. | Pass proxy |
| The ball felt too floaty. | 2 | Fixed physics config and human-approved Checkpoint 1 tuning remain in place; no smoke metric indicates stalls. | Pass proxy |
| The ball felt too pinball-like or chaotic. | 2 | First-bounce, wall damping, deterministic replay, and readable event counts pass. | Pass proxy |
| Bucket catches felt skillful and satisfying. | 3 | Current smoke shot had `bucket=0`, but board diagnostics in workspace tests cover catch opportunities. Requires human confirmation. | Proxy inconclusive |
| Peg hits were readable. | 5 | Feel smoke reports `pegs=7`, `feedback_events=12`, `feedback_cues=12`; VFX trigger list includes blue/orange/purple/green peg hits. | Pass proxy |
| Near misses were understandable. | 5 | C4 trigger list includes `near_miss`; audio/VFX tests verify near miss is distinct from loss/victory. | Pass proxy |
| Board clear payoff was exciting without becoming annoying. | 4 | Trigger list includes `extreme_fever`; accessibility tests cap/reduce flash and shake. Human annoyance cannot be automated. | Pass proxy |
| I wanted to take one more shot after the test ended. | 3 | No automated proxy can measure desire; smoke remains deterministic and playable path exists. | Human required |
| Reward choices were understandable before I selected one. | 4 | Plugin registration reports `reward_ui(cards=3, relic_metadata=true, smoke_auto=true)`. | Pass proxy |
| The selected reward's effect was visible or easy to explain on the next board. | 4 | C2 run summary includes collected relic IDs and run-state effects; feedback triggers include relic category flashes. | Pass proxy |
| The node map made it clear where I was and where I would go next. | 4 | Plugin registration reports `node_map(visible=2, current_highlighted=true, hidden_future=2)`. | Pass proxy |
| Moving from board 1 to board 2 felt connected rather than like a reset. | 4 | C2 smoke carries resources/relics through 3 boards and emits one run summary hash. | Pass proxy |

## Comprehension Check Proxy

- What do orange pegs mean? Proxy answer: clear/win-condition pegs; C2 summary reports `hit_oranges=22` and board-won progression.
- What does the bucket do? Proxy answer: grants a free shot/catch reward; smoke did not catch, but rules and feedback triggers cover `bucket_catch`.
- What helped you aim? Proxy answer: `first_bounce=true` and debug aim reuse.
- Which collision or bounce felt least fair? Proxy answer: none flagged by deterministic tests; no unfair-shot tag emitted.
- Which reward did you choose, and why? Proxy answer: smoke path auto-selected deterministic rewards, ending with `relics/act1/wide_cup_rim` and `relics/act1/stone_chisel`.
- After choosing a reward, what did you expect it to change? Proxy answer: run-state relic list and next-board behavior; smoke records relic IDs in run summary.
- On the node map, which node were you on and which node was next? Proxy answer: node-map model exposes current highlight and two visible nodes; no human ambiguity response available.
- Was anything about the node map path, current node, reward source, or next board unclear? Proxy answer: no automated ambiguity flag; human confirmation required.

## Open Notes

- Best-feeling moment proxy: feel-test smoke emits 21 C4 VFX triggers including launch, peg colors, combo thresholds, long shot, lucky bounce, relic category flashes, extreme fever, near miss, and board failure.
- Worst-feeling moment proxy: current feel-test shot ends as `result=miss`; no bucket catch occurred in smoke.
- Replay hash for any bug or unfair-feeling shot: none tagged; primary feel replay `e70c8f293c5c5db192ef4620c03cb7e7000dc30433a0aab12f25e1706263a384`.
- Catch/failure note: `bucket=0`, `exits=1`, result `miss`, progression `Continue`.
- Reward-choice note: proxy clear via metadata and deterministic selection; human expectation test remains open.
- Node-map note: proxy clear via current highlight/visible nodes; human path comprehension remains open.
- Suggested tags: `FirstBounceReadable`, `BucketCatchMissed`, `VerticalSliceFailure` not triggered; `BucketCatchSatisfying` requires human validation.
