---
name: cross-session-coordination
description: Use when working in `zirconEngine` and other recent Codex sessions may be touching related plans, crates, failing tests, or numbered-plan `failure-*.md` handoffs.
---

# Cross Session Coordination

## Overview

Coordinate nearby `zirconEngine` Sessions through the local coordinator service and numbered child-plan Failure graph. Use a compact `.codex/sessions/` Markdown note only when service state cannot carry a material human coordination warning.

## When to Use

- Start of any non-trivial `zirconEngine` task that may overlap shared runtime, module wiring, scripting, graphics, editor, asset, networking, or test work already in flight.
- Before editing a module or test area that appears in a recent plan from the last few hours.
- When a failing test or broken task might belong to another active session rather than the current change.
- When handing off work, warning other sessions about a risky area, or recording a blocker that affects coupled modules.
- When the active numbered plan contains `failure-*.md`, or a failure must be returned as `fixed-*.md`.
- When `.codex/sessions/` does not exist yet and a shared live-status location is needed.

Do not use this skill for isolated one-file edits with no plausible overlap, or as a replacement for the current numbered plan.

## Non-Negotiable Rules

- Default lookback window: `4` hours unless the user says otherwise.
- Start or query `tools/zircon-session.ps1`, register the current Session and numbered plan, and use the service as the first coordination source. Markdown notes are the offline fallback, not a competing state database.
- **Coordinator leases are the only write-exclusion mechanism in the shared checkout.** Never create an OS-level source-file lock (including a `FileShare.Read` loop), and never use ACL changes or `ReadOnly` attributes to block another Session. If an unmanaged lock is suspected, diagnose it with Restart Manager, preserve live managed Cargo trees, terminate only the verified locking helper, and record the owner/evidence in the current Session note before continuing through coordinator leases and delayed patches.
- Before overlap-sensitive edits or debugging, scan `.codex/plans/` and `.codex/sessions/` sorted by `LastWriteTime` descending. Freshness is mandatory.
- Independently scan the active `docs/plans/{family}/{id}/` directory for `failure-*.md`; open handoffs do not expire with the four-hour lookback.
- Read modification times before trusting a plan or session note. Stale notes are not active coordination signals.
- Treat only root-level `.codex/sessions/*.md` files as active. Ignore archived or completed notes unless a handoff explicitly requires them.
- Create or update at most one compact session note for the current task, and only when another Session needs context beyond coordinator enum state, leases, Failure links, and delayed-patch data. Do not create a note merely because the directory is absent.
- When retained, a session note must state only the current goal, touched modules, material blocker or warning, and next step. Do not duplicate coordinator fields, model selection, plan prose, validation results, or test logs.
- Treat session notes as live coordination state, not as canonical plan output records. Before any session writes a concrete output record under `docs/plans`, apply `../write-plan-output-records/SKILL.md` and write to the owning numbered child plan or archive.
- If a failing test or broken behavior appears outside the current task's scope, check recent plans and session notes before trying to "fix" it.
- If another numbered child plan owns the lowest shared cause, apply `../handle-plan-failure-handoffs/SKILL.md`. Publish the handoff, continue independent source work, and do not mark the session blocked solely because the handoff is open.
- **Failure Priority Gate:** when the registered plan is the fixing owner of an applicable open handoff, switch to `resolving_failure` and do not begin ordinary feature slices until the handoff is returned as `fixed-*`.
- **Model Tier Gate:** before any cross-Session task, read [the model-tier policy](references/model-tier-policy.md), declare its allowed `5.6-sol` / `5.6-terra` / `5.6-luna` tier and thinking depth in the task/session state, and reject `gpt-5.5` or lower fallback. When the platform cannot set a model, verify the active runtime model is allowed before dispatch.
- When a failure is in a lower shared layer, also apply `../support-first-regression-testing/SKILL.md`.
- On completion, remove the note from active circulation: delete it if no handoff record is needed, or move it to `.codex/sessions/archive/` with `status: completed`. Never leave completed notes in the active root.

## Receipt-Driven Progress

- After a coordinator validation, integration, mutation, or offline-queue receipt is durable, return to executable Goal work immediately. Never turn `queued`, `materializing`, `running`, coordinator recovery, or database maintenance into `waiting_validation` or a polling loop.
- Treat an actual coordinator integration SHA as the authority that an owned snapshot is on `main`; keep it `integrated_validation_pending` until complete validation evidence arrives. A receipt without an integration SHA is not permission to use direct Git or to claim `accepted`.
- Preserve integrated snapshots while full validation is pending. When the coordinator materializes a canonical `failure-*.md`, route it to its fixing Plan and wake or continue that Plan's primary Session for forward repair; do not roll back ordinary test failures or block unrelated Plans.
- When no ordinary Goal slice remains, perform the required final in-scope review and release the Session for coordinator wakeup. A pending ticket delays only `accepted` closeout, never consumes an active Session turn.

## Blocked-Status Gate

Before setting a Session to `blocked`, re-scan the registered Goal and execute every safe, in-scope item that does not require the blocked dependency. Apply the **Failure Priority Gate** first when it is applicable; resolving its owned failure is actionable work, not a reason to idle.

| Tempting reason to wait | Required response |
| --- | --- |
| “Wait for the coordinator, lease, or CI worker.” | Record the affected dependency, then continue independent implementation, review, or local validation. |
| “Wait for the other Session to finish.” | Follow up or publish a handoff, then continue approved Goal work that has no shared dependency or write conflict. |
| “A manager says to wait for coordination.” | Treat it as a dependency-specific restriction unless the current user explicitly pauses the entire Goal. |
| “Strict task order means later work must wait.” | Start an approved downstream Goal item when its entry criteria are met and no dependency conflicts exist. |
| “Monitoring the blocker is the next step.” | Keep monitoring lightweight; rebuild and execute the actionable queue instead of idling. |

- Do not let another Session, coordinator state, lease holder, reviewer, or indirect management message pause the whole Goal. Treat a request to avoid a specific shared dependency as applying only to that dependency; pause the full Goal only on an explicit current-user instruction to do so.
- Include independent implementation slices, approved downstream Goal work, completed-code review, and in-scope validation/repair in the re-scan.
- Set the Session to `blocked` only after the re-scan shows every remaining in-scope item requires the same unresolved external dependency or a genuine user product decision.
- Do not treat a pending validation ticket or a coordinator receipt without a terminal result as an unresolved external dependency for this gate.

## Workflow

1. Scan recent coordination context.
- Run `tools/zircon-session.ps1 status -Json`. When online, register the current Session with its numbered plan and declared write scope, then query `failure open <plan-path>` before ordinary work.
- Run `scripts/Get-RecentCoordinationContext.ps1 -RepoRoot <repo> -LookbackHours 4`.
- Review fresh plans first, then fresh session notes, then decide whether overlap exists.
- Scan the active numbered child-plan directory for `failure-*.md`; a fixing owner must resolve and return every applicable handoff before ordinary feature progress, while an origin owner may continue only dependency-independent slices.

2. Read the related work before changing coupled modules.
- Open the recent plan when its title or summary touches the same module, subsystem, or failing test.
- Open the recent session note when another session is actively editing or diagnosing the same area.

3. Publish the current session state.
- Keep enum status, heartbeat, plan owner, write scope and file leases in the coordinator. Use `lease claim` before editing concrete shared files; enqueue a delayed patch when another Session owns the path.
- Keep `model_tier`, `thinking_depth`, `selection_reason`, and `primary_session` in the coordinator task state; mirror none of them into Markdown. A temporary cross-Session task must return to `primary_session` after its accepted evidence; do not keep spawning detours while the primary task is executable.
- Create `.codex/sessions/` only when the coordinator cannot express a material warning and a compact offline note is required.
- Only when the coordinator cannot express the required warning or handoff context, copy the compact `references/session-note-template.md` into a new note such as `.codex/sessions/20260408-0315-parser-import-cleanup.md`.
- Record only one-line live state: current goal/step, touched modules, one material blocker or warning, and next step. Link a Failure or plan instead of copying its prose.
- When the session produces a plan output record, write it through `../write-plan-output-records/SKILL.md`; do not leave the session note as its sole permanent copy.
- Update the note when the scope, blocker set, failing tests, or touched modules materially change.

4. Coordinate rather than guessing.
- If a test failure seems to belong to another active task, inspect the matching plan and session note before editing shared code or reverting behavior.
- Use the session note to communicate concrete facts and avoid speculative fixes across somebody else's active area.
- Summarize cross-module coupling explicitly when one task can invalidate another task's assumptions.

5. Retire the note cleanly.
- Delete the active note when the task ends cleanly and nobody else needs a handoff.
- Move the note to `.codex/sessions/archive/`, set `status: completed`, and add a short completion summary only when another session still needs the result.

## Resources

- `references/session-note-template.md` contains the active-note template and lifecycle rules.
- `references/model-tier-policy.md` is the mandatory allowed-model, cost, thinking-depth, and no-fallback contract.
- `scripts/Get-RecentCoordinationContext.ps1` prints a markdown digest of recent plans and active session notes.
- `../handle-plan-failure-handoffs/SKILL.md` owns durable failure/fixed artifact naming, routing, priority, and return rules.

## Quick Commands

```powershell
.\tools\zircon-session.ps1 start -Json
.\tools\zircon-session.ps1 session register --plan-path docs/plans/<family>/<id>-<plan>.md --write-scope <path>
.\tools\zircon-session.ps1 failure open docs/plans/<family>/<id>-<plan>.md
.\.codex\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot E:\Git\ZirconEngine -LookbackHours 4
New-Item -ItemType Directory -Force .\.codex\sessions
```

## Common Mistakes

- Reading plan or session bodies without checking `LastWriteTime`.
- Leaving completed notes in `.codex/sessions/` root where they look active.
- Using session notes for long design prose instead of concise live coordination state.
- Editing another session's coupled module after seeing a failure but before reading its recent plan or note.
- Treating every nearby failure as current-session ownership without checking overlap first.
- Treating an open cross-plan handoff as permission to stop all independent work.
