# Agent Scaffolding

This directory contains planning and orchestration material for autonomous agents working on FeverFall. These files are development scaffolding, not player-facing game documentation or runtime content.

## Contents

- [`03_parallel_subagent_development_plan.MD`](./03_parallel_subagent_development_plan.MD) — original parallel workstream plan used to bootstrap early implementation.
- [`04_standalone_orchestrator_prompt.MD`](./04_standalone_orchestrator_prompt.MD) — standalone orchestrator prompt and coordination rules.
- [`05_checkpoint5_devplan.MD`](./05_checkpoint5_devplan.MD) — Checkpoint 5 workstream ownership, dispatch order, and exit criteria.
- [`05_checkpoint5_resumption_prompt.MD`](./05_checkpoint5_resumption_prompt.MD) — Checkpoint 5 session-resumption context for agents.

## Maintenance Rules

- Keep root-level project entry points focused on humans: `README.md`, `CONTRIBUTING.md`, and `CHANGELOG.md`.
- Keep agent-only plans, prompts, and task handoff scaffolding under `docs/agent/` unless a future checkpoint explicitly chooses another location.
- Do not place runtime assets, source code, CI workflows, or release artifacts in this directory.
- When a checkpoint agent completes its owned task, update the active devplan entry here and leave a concise handoff for the next agent.
