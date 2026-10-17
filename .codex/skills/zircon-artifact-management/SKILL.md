---
name: zircon-artifact-management
description: Audit ZirconEngine Codex write directories and route Git, worktree, generation, artifact reuse and cleanup lifecycle operations through Jenkins.
---

# Zircon Artifact Management

Use for directory disorder, legacy source copies, generated outputs, or planning
and closing Jenkins-owned Git/worktree and artifact operations. Read the relevant
sections of the [management standard](../../../docs/tools/agent-artifact-management.md).
Use the user's [Artifact Lifecycle](D:/CodexData/.codex/skills/artifact-lifecycle/SKILL.md)
for its manifest and retention schema; in ZirconEngine, Jenkins performs its
generation, plan execution and cleanup. Its generic local-build/cleanup defaults
do not replace the user's Jenkins ownership requirement.

## Ownership

Jenkins owns all version-management and file-generation workflows: branches,
worktrees, multi-worktree merges, commits/reverts/pushes, source baselines and
candidates, generated code/resources/exports, builds/tests, artifact reuse,
post-task cleanup and scheduled cleanup. Respect any explicit user-authorized
exception; a writable directory or a failed Jenkins request supplies none.

Agents may read, design, edit explicitly authorized canonical source/doc/skill
files, prepare scoped incremental inputs, and reconcile actual receipts. Do not
create ad hoc output trees, source copies, worktrees, private targets, local build
or generation runs, cleanup commands or independent scheduled jobs. Follow the
[source ownership policy](../jenkins-coordination/references/incremental-patch-validation.md).

## Audit and route

For task delivery, the user's latest 2026-10-17 order covers **any session**:
Jenkins first checks the real per-worktree index.lock, commits exact task files
on that task's branch, and registers the immutable commit in the single scheduled
merge queue. Keep shared main clean and perform task edits in its assigned
worktree. `zircon-merge-queue` validates the candidate containing current main,
then requires an actual merged PR with the same result tree before fast-forwarding
local main. Branch commit success alone is not validation or integration; direct
local candidate publication is disabled.
Use [the delivery contract](../../../docs/tools/jenkins-branch-integration.md) and
`tools/dev/jenkins_branch_queue.py`. No separate session/commit/merge approval is
required. Preserve foreign dirt, unknown locks and original pending results;
Jenkins may recover a lock only after actual owner/native reconciliation. The
older transition exception below does not revoke this later authorization.

- Read the current `.codex/config.toml`; distinguish explicit write grants from
  actual ownership and sandbox enforcement. Do not change grants during cleanup.
- Inspect current selected Jenkins driver/runtime, the relevant job, shared
  changes and chat/handoff notes. Use existing `zircon-flow`, `zircon-execution`
  and `zircon-maintenance` responsibilities. Verify implementation, activation
  and actual acceptance separately. Missing capabilities stay pending; do not
  substitute a local workflow, captured bundle or retired coordinator.
- For inventory, run `python -B -X utf8 -m tools.dev.artifact_inventory` from the
  repository root. It reads directories and emits JSON to stdout; Jenkins owns
  generated report files and archiving. An inventory is not authoritative GC
  intake or a deletion/acceptance receipt.
- Mark unclaimed legacy content for owner review. Check unique source changes,
  retained references, process birth/termination and physical path identity
  before proposing a bounded Jenkins maintenance request. Protect source/control
  files, shared caches, Jenkins state, locks and retired historical resources.
- Bind operations to their original request/attempt, exact input hashes, source
  and coverage, artifact generation and runtime/driver identities. Reconcile an
  uncertain result before retrying. Multi-worktree merge acceptance covers its
  actual result tree; source claims and version actions remain owner-scoped.
- At closeout, verify delivery and Jenkins maintenance results, including native
  process/Job termination and EOF. Report actual deletions, retained paths and
  pending acceptance accurately. Do not independently delete owned build or
  generated artifacts because the task ended.

Keep all compiler products/caches physically inside approved drive-root
`D/E/F:\cargo-targets` namespaces. Report logical bytes as logical bytes; neither
directory age nor a missing manifest establishes safe reclamation.

## Explicit transition exception

The user authorized daily file/health review and temporary merge takeover on
2026-10-11. Follow the [transition protocol](../../../docs/tools/jenkins-transition-monitor.md)
for that authorization. The user requires a **Codex scheduled task**, daily at
09:00 Asia/Shanghai, in the actual project directory without a new worktree.
Use the [saved task prompt](references/daily-codex-review.md) and the supported
Codex task interface; verify the returned task identity and schedule. If that
interface is unavailable, do not claim to have created a task. A user-created
task may be observed through its existing local registration record: verify its
identity, active status, recurrence, target chat and prompt read-only, and keep
actual run acceptance separate. Do not substitute
Windows Task Scheduler, a background polling service or direct Codex data edits.
The task records observations and reports meaningful changes or required action.
Reviewed operator batches integrate exact paths with current evidence. Record
health and merge blockers separately.
This daily-review exception itself does not authorize automatic commits; task
delivery has the separate standing authority above. It does not authorize source copies, cleanup,
service adoption/restart, pushes, or restoration of the retired coordinator.
Retire the temporary scheduler after the actual Jenkins lifecycle handoff has
been recorded. Jenkins Ready alone does not satisfy that handoff.
