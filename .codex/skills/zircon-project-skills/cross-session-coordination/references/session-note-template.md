# Session Note Template

Use one markdown file per active task under `.codex/sessions/`. Name it with a local timestamp plus a short task slug, for example `20260408-0315-runtime-script-hot-reload.md`.

## Lifecycle

- Scan recent `.codex/plans/` and `.codex/sessions/` activity before creating the note.
- Keep the note in `.codex/sessions/` only while the task is active or blocked.
- Update the note whenever the current step, blocker set, touched modules, or related failing tests materially change.
- Keep concrete plan output records out of the session note. Write them to the owning numbered child plan or archive according to `../../write-plan-output-records/SKILL.md`.
- Keep cross-plan failures out of `## Blockers`: publish them through `../../handle-plan-failure-handoffs/SKILL.md`, link the durable `failure-*` artifact in coordination notes, and continue independent source-plan work.
- Delete the note when the task completes and no handoff is needed.
- If another session still needs the result, move the file to `.codex/sessions/archive/`, set `status: completed`, and add a short completion summary.

## Template

```markdown
---
session: 20260408-0315-runtime-script-hot-reload
status: active
updated_at: 2026-04-08 03:15 +08:00
owner: codex
lookback_hours: 4
related_plans:
  - .codex/plans/example plan.md
touched_modules:
  - zircon_runtime::script
related_tests:
  - zircon_runtime/src/script/mod.rs
---

# Session Summary: runtime script hot reload

## Goal
- ...

## Current Step
- ...

## Touched Modules
- ...

## Related Plans
- ...

## Checks / Failing Signals
- ...

## Coordination Notes
- ...
- State any module or test area other sessions should avoid touching blindly.

## Blockers
- None. A cross-plan `failure-*` handoff alone is not a session blocker.

## Next Update
- ...

## Handoff / Completion
- Delete this note when finished if no handoff is needed.
- Otherwise move it to `.codex/sessions/archive/`, set `status: completed`, and write a 2-5 bullet completion summary.
```

## Editing Guidance

- Keep headings stable so other sessions can scan quickly.
- Prefer facts, file paths, test names, and explicit warnings over long narrative explanations.
- Treat this note as coordination state only; it must not become the sole permanent owner of a concrete output record.
- Update `updated_at` each time the note changes materially.
- If the task expands into a new subsystem, add the new touched module explicitly instead of assuming readers will infer it.
