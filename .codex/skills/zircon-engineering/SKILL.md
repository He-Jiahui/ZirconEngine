---
name: zircon-engineering
description: Use when planning, implementing, or reviewing ZirconEngine work that needs the repository-standard delivery cadence, validation scope, and specialist-skill routing.
---

# Zircon Engineering

Use this as the default entry for ZirconEngine work. Keep ordinary delivery simple; load specialist skills only when the task has their trigger.

## Capability Tiers

| Tier | Scope | Required focus |
|---|---|---|
| C1 | Bounded code or docs change | Local ownership and focused regression coverage |
| C2 | Plan milestone or subsystem change | Dependency order, batched validation, one milestone record |
| C3 | Cross-crate, ABI, migration, or architecture change | Explicit boundary design and all affected contract gates |

## Default Delivery Loop

1. **Orient.** Read the request, active plan milestone, touched code, and directly related tests. Select C1-C3; do not load the complete skill tree.
2. **Build.** Complete coherent slices as one milestone batch. Add tests when behavior or a contract changes. Use formatting, diff checks, and source guards while editing. Do not run Cargo by default during implementation slices.
3. **Validate and record.** Milestone validation follows `docs/plans/milestone-validation-policy.md`. Run the smallest declared batch, correct failures from the lowest shared cause, then write one concise evidence record per accepted milestone.

## Conditional Specialists

- Rust/Cargo command: `zircon-dev`; choose its validation guidance only when a Cargo gate is due.
- New subsystem, public boundary, ABI, or hard move: `zircon-project-skills` architecture and migration skills.
- Active overlap: `cross-session-coordination`; real dependency failure: `handle-plan-failure-handoffs`.
- Concrete plan evidence: `write-plan-output-records`; closeout or commit: `close-session-goal-milestones`.

Do not default to per-slice Cargo checks, per-slice plan rows, coordinator registration, WSL validation, or full architecture reading.
