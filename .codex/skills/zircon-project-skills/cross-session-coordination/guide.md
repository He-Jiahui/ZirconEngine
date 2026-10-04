# Cross Session Coordination

Use current chats, diffs, owning plans and canonical `failure-*.md` records when another task may edit the same files or a shared dependency is changing. The local coordinator is retired; registration, heartbeat, leases, attribution APIs and delayed-patch queues are not required.

## Establish ownership

1. Inspect the relevant current diff and available task context. Read matching plan/handoff notes when overlap is plausible; re-read only when their content changes.
2. Establish the exact authorized write scope. Preserve foreign and preexisting edits, including staged and untracked paths. Compare current file hashes before applying a prepared change.
3. For an actively overlapping file, resolve the ownership or handoff before writing. Do not invent a numbered plan for isolated maintenance.
4. Preserve live OS/Cargo locks and shared-index ownership. Do not change ACLs, set source files read-only, delete a live lock, or rewrite another task's index to obtain ownership. Diagnose unexpected Windows locks with Restart Manager; stop a helper only after identity and authorization are verified.

## Dependencies and progress

- Identify the owning plan before changing a failure outside the current scope. Use [failure handoffs](../handle-plan-failure-handoffs/guide.md) for a cross-plan cause.
- An applicable failure blocks dependent acceptance. Repair it at the fixing owner's next repair window, immediately when no independent slice remains; unrelated failures do not stop independent authorized work.
- Follow [receipt lifecycle](references/receipt-lifecycle.md). Preserve historical receipt IDs and results for migration; do not submit them again to the retired service or infer a pass from pending work.
- For Jenkins coordinator functional tests and bounded command submission, use [jenkins-coordination](../../jenkins-coordination/SKILL.md) and verify its active configuration and evidence. The Jenkins tray handles the current service lifecycle; pending requests retain their exact identities across chats and restarts. Jenkins command acceptance does not grant a milestone, whole-workspace or production migration pass.
- Follow the current environment's status and delegation rules. Use [model selection](references/model-tier-policy.md) only when delegation is permitted; inherit the active model and effort by default.

## Minimal coordination records

Use a compact `.codex/sessions/*.md` note only for a material live warning or handoff. Link the owning plan/failure, without duplicating plan prose, model metadata or test logs. On completion remove only this task's obsolete note, or archive it when another owner needs the handoff.

Durable accepted milestone evidence follows [plan output records](../write-plan-output-records/guide.md). A chat ending or task note does not provide acceptance evidence.

Optional local context discovery remains available:

```powershell
.\.codex\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot E:\Git\ZirconEngine -LookbackHours 4
```
