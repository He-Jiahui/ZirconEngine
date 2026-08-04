---
name: zircon-engineering
description: Use when planning, implementing, or reviewing ZirconEngine work that needs the repository-standard MVP-baseline priority gate, delivery cadence, validation scope, and specialist-skill routing.
---

# Zircon Engineering

Use this as the default entry for ZirconEngine work. Keep ordinary delivery simple; load specialist skills only when the task has their trigger.

## Capability Tiers

| Tier | Scope | Required focus |
|---|---|---|
| C1 | Bounded code or docs change | Local ownership and focused regression coverage |
| C2 | Plan milestone or subsystem change | Dependency order, batched validation, one milestone record |
| C3 | Cross-crate, ABI, migration, or architecture change | Explicit boundary design and all affected contract gates |

## MVP Baseline Gate

- Treat `docs/plans/mvp/index.md` as the canonical priority and acceptance source. The MVP baseline is complete only when `00` and `F0` through `F5` all have durable current evidence and are marked accepted there.
- Until that condition is true, do not start or extend advanced engine features. This includes new rendering techniques, optional plugin capabilities, editor polish, scripting breadth, physics, networking, or showcase work that is not required by an MVP gate.
- Prioritize only work that closes the MVP baseline: clean build and lockfile convergence, Runtime/Editor product startup, the canonical project and asset path, basic rendering and input, persistence and authoring, failure repair, removal of obsolete blockers, and acceptance automation.
- When an advanced slice is already in progress, preserve its work and stop at the nearest coherent boundary. Do not revert concurrent or completed changes merely to enforce this gate. Record any necessary handoff, then redirect the Session to an MVP blocker or release its scope.
- A plan checkbox, source-only test, or pending validation receipt does not reopen advanced work. Resume advanced feature expansion only after the canonical MVP index records the full accepted baseline, or after an explicit user instruction changes this policy.

## Default Delivery Loop

1. **Orient.** Read the request, canonical MVP status, active plan milestone, touched code, and directly related tests. Select C1-C3; do not load the complete skill tree.
2. **Build.** Complete coherent slices as one milestone batch. Add tests when behavior or a contract changes. Use formatting, diff checks, and source guards while editing. Do not run Cargo by default during implementation slices.
3. **Validate and record.** Milestone validation follows `docs/plans/milestone-validation-policy.md`. Run the smallest declared batch, correct failures from the lowest shared cause, then write one concise evidence record per accepted milestone.

## Conditional Specialists

- Rust/Cargo command: `zircon-dev`; choose its validation guidance only when a Cargo gate is due.
- New subsystem, public boundary, ABI, or hard move: `zircon-project-skills` architecture and migration skills.
- Active overlap: `cross-session-coordination`; real dependency failure: `handle-plan-failure-handoffs`.
- Concrete plan evidence: `write-plan-output-records`; closeout or commit: `close-session-goal-milestones`.

Do not default to per-slice Cargo checks, per-slice plan rows, coordinator registration, WSL validation, or full architecture reading.
