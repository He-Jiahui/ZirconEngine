---
name: zircon-engineering
description: Plan and deliver ZirconEngine milestones using MVP priorities and the repository validation cadence.
---

# Zircon Engineering

Use this entry for delivery policy and milestone planning. Complete the user's authorized scope; select a specialist only for an actual affected boundary. Reuse policies already read and unchanged.

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

1. **Orient.** Read the request, touched code, and directly related tests. Read the canonical MVP status when choosing new work, and the active milestone when executing a plan. Select C1-C3 without touring the skill tree.
2. **Build.** Complete coherent slices as one milestone batch. Add tests when behavior or a contract changes. Use formatting, diff checks, and source guards while editing. Do not run Cargo by default during implementation slices.
3. **Validate and record.** Milestone validation follows `docs/plans/milestone-validation-policy.md`. Run the smallest declared batch, correct failures from the lowest shared cause, then write one concise evidence record per accepted milestone.

## Source preparation

Agents must not create source snapshots or backup copies for this project, including in outbox or temporary directories. Edit authorized canonical files directly and hand off scoped patches and hashes. The independent Jenkins coordinator owns source baseline, candidate and bundle preparation under the [source ownership policy](../jenkins-coordination/references/incremental-patch-validation.md). If a verified handoff is unavailable, keep the required coordinator acceptance pending.

## Select the needed guidance

| Current need | Owner |
| --- | --- |
| Rust editing or Cargo | [Zircon Dev](../zircon-dev/SKILL.md) |
| New subsystem, changed ownership, ABI, or structural migration | [Architecture](../zircon-project-skills/zr-architecture-first-engineering/SKILL.md) |
| Application layout | [UI layout](../zircon-project-skills/zr-ui-layout-reference/SKILL.md) |
| Unclear defect or regression | [Debugging](../superpowers/review-and-stabilization/systematic-debugging/SKILL.md) |
| Approved plan execution | [Execution loop](../zircon-project-skills/continuous-milestone-execution/execution-loop/index.md); [layering](../zircon-project-skills/layered-milestone-development/guide.md) when ordering dependencies |
| Active checkout overlap or pending asynchronous evidence | [Coordination](../zircon-project-skills/cross-session-coordination/guide.md) |
| A failure owned by another plan | [Failure handoff](../zircon-project-skills/handle-plan-failure-handoffs/guide.md) |
| Accepted milestone evidence or authorized integration | [Output records](../zircon-project-skills/write-plan-output-records/guide.md) or [closeout](../zircon-project-skills/close-session-goal-milestones/guide.md) |
| A public contract or operator document becomes false | [Documentation](../zircon-project-skills/code-module-docs-maintenance/guide.md) |

The reference index is available for lookup; it is not a prerequisite for ordinary source work. No per-slice Cargo checks, per-slice plan rows, coordinator registration, duplicate WSL runs, or automatic full architecture reading.
