---
name: zircon-artifact-management
description: Audit ZirconEngine output ownership and manage scoped Git, worktree, generation, artifact reuse and cleanup directly within the task authorization.
---

# Zircon Artifact Management

Use for directory disorder, legacy source copies, generated outputs, or planning and closing scoped Git/worktree and artifact operations. The [Agent management standard](../../../docs/tools/agent-artifact-management.md) is the canonical workflow owner. Use the user's [Artifact Lifecycle](D:/CodexData/.codex/skills/artifact-lifecycle/SKILL.md) for its manifest and retention schema, not as additional permission.

## Ownership

Agents directly manage authorized native Git/worktree operations, generation, local validation, compatible artifact reuse and bounded task cleanup. Follow [checkout ownership](../zircon-dev/references/main-branch-development-policy.md) and the [validation policy](../../../docs/testing-and-validation/milestone-validation-policy.md). A writable directory or completed milestone does not expand the task or authorize deletion, commits or remote actions.

Keep shared main for synchronization/inspection and use the task's assigned or authorized branch/worktree. Before Git writes inspect the real Git directory, index lock and exact owned changes. An existing scoped commit grant is sufficient; stage only those paths/increments and report the actual SHA. Preserve foreign dirt, unknown locks, original pending results and merge conflicts. Push, remote PR actions, publication and external messages retain separate authorization.

## Audit and act

- Read the current `.codex/config.toml`; distinguish execution permissions, task scope and actual ownership. Full Access does not require the optional strict-profile allowlist. Do not change permissions merely to make cleanup work.
- Inspect shared changes and relevant chat/handoff notes before overlapping edits. Record repository/worktree/base, owner, exact source inputs, commands, outputs and retention conditions. Use a general-purpose Python interpreter for inspection and skill maintenance.
- For inventory, run `python -B -X utf8 -m tools.dev.artifact_inventory` from the repository root with the authorized scope. It reads directories and emits JSON to stdout. An inventory is not a deletion, terminal-process or acceptance receipt.
- Generate required task evidence in its approved output namespace. Do not create ad hoc source backups, snapshots, archives, temporary validation copies or per-session full targets.
- Mark unclaimed content for owner review. Check unique source changes, retained references, process birth/termination, native locks and physical path identity before any authorized bounded cleanup. Protect source/control files, shared caches, historical Jenkins/coordinator data and other tasks' resources.
- Bind asynchronous work to the original job/request/attempt and source identity. Reconcile an uncertain result before retrying. Verify the actual merged result tree rather than attributing combined-tree evidence to an untested subset.
- At closeout, verify actual generation, validation, delivery and authorized maintenance outcomes, including native process/Job termination and EOF where applicable. Report actual deletions, retained paths and pending acceptance accurately. Task completion alone is not permission to delete outputs.

All compiler products, build directories, caches and temporary compiler outputs remain physically inside approved drive-root `D/E/F:\cargo-targets` namespaces, with the 35 GiB reserve and native Cargo locks. Reuse compatible outputs without duplicating full caches. Report logical bytes as logical bytes; neither directory age nor a missing manifest establishes safe reclamation.

## Historical scheduling and Jenkins records

The [2026-10-11 transition protocol](../../../docs/tools/jenkins-transition-monitor.md) and [saved daily-review prompt](references/daily-codex-review.md) document an older Jenkins handoff. They are historical references, not current ordinary-development entry points. Do not execute their commands or use them to restore services, alter task databases or create or change schedules.

Retain historical identities, credentials, queued work and acceptance results. Replacing the repository workflow does not prove any host service stopped or any pending migration/production gate passed.
