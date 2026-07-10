---
name: cross-session-coordination
description: Use when working in `zirconEngine` and other recent Codex sessions may be touching related plans, crates, failing tests, or numbered-plan `failure-*.md` handoffs.
---

# Cross Session Coordination

## Overview

Coordinate nearby `zirconEngine` sessions through recent plan intent, numbered child-plan failure handoffs, and live markdown state under `.codex/sessions/`.

## When to Use

- Start of any non-trivial `zirconEngine` task that may overlap shared runtime, module wiring, scripting, graphics, editor, asset, networking, or test work already in flight.
- Before editing a module or test area that appears in a recent plan from the last few hours.
- When a failing test or broken task might belong to another active session rather than the current change.
- When handing off work, warning other sessions about a risky area, or recording a blocker that affects coupled modules.
- When the active numbered plan contains `failure-*.md`, or a failure must be returned as `fixed-*.md`.
- When `.codex/sessions/` does not exist yet and a shared live-status location is needed.

Do not use this skill for isolated one-file edits with no plausible overlap, or as a replacement for full design documents inside `.codex/plans/`.

## Non-Negotiable Rules

- Default lookback window: `4` hours unless the user says otherwise.
- Before overlap-sensitive edits or debugging, scan `.codex/plans/` and `.codex/sessions/` sorted by `LastWriteTime` descending. Freshness is mandatory.
- Independently scan the active `docs/plans/{family}/{id}/` directory for `failure-*.md`; open handoffs do not expire with the four-hour lookback.
- Read modification times before trusting a plan or session note. Stale notes are not active coordination signals.
- Treat only root-level `.codex/sessions/*.md` files as active. Ignore archived or completed notes unless a handoff explicitly requires them.
- Create or update exactly one session note for the current task in `.codex/sessions/`.
- Every active session note must state the current goal, touched modules, related plans/tests, blockers, next step, and any explicit coordination warning.
- Treat session notes as live coordination state, not as canonical plan output records. Before any session writes a concrete output record under `docs/plans`, apply `../write-plan-output-records/SKILL.md` and write to the owning numbered child plan or archive.
- If a failing test or broken behavior appears outside the current task's scope, check recent plans and session notes before trying to "fix" it.
- If another numbered child plan owns the lowest shared cause, apply `../handle-plan-failure-handoffs/SKILL.md`. Publish the handoff, continue independent source work, and do not mark the session blocked solely because the handoff is open.
- When a failure is in a lower shared layer, also apply `../support-first-regression-testing/SKILL.md`.
- On completion, remove the note from active circulation: delete it if no handoff record is needed, or move it to `.codex/sessions/archive/` with `status: completed`. Never leave completed notes in the active root.

## Workflow

1. Scan recent coordination context.
- Run `scripts/Get-RecentCoordinationContext.ps1 -RepoRoot <repo> -LookbackHours 4`.
- Review fresh plans first, then fresh session notes, then decide whether overlap exists.
- Scan the active numbered child-plan directory for `failure-*.md`; resolve applicable handoffs before ordinary feature progress.

2. Read the related work before changing coupled modules.
- Open the recent plan when its title or summary touches the same module, subsystem, or failing test.
- Open the recent session note when another session is actively editing or diagnosing the same area.

3. Publish the current session state.
- Create `.codex/sessions/` if it does not exist yet.
- Copy `references/session-note-template.md` into a new note such as `.codex/sessions/20260408-0315-parser-import-cleanup.md`.
- Record only live state: current goal, current step, touched modules, related plans/tests, blockers, and warnings for other sessions.
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
- `scripts/Get-RecentCoordinationContext.ps1` prints a markdown digest of recent plans and active session notes.
- `../handle-plan-failure-handoffs/SKILL.md` owns durable failure/fixed artifact naming, routing, priority, and return rules.

## Quick Commands

```powershell
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
