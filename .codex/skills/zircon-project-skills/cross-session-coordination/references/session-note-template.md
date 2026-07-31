# Session Note Template

Use this compact offline note only when coordinator status, leases, Failure links, and delayed-patch state cannot convey a material warning to another Session. Do not create one for ordinary active work. Name a retained note with a local timestamp plus a short task slug, for example `20260408-0315-runtime-script-hot-reload.md`.

## Lifecycle

- Scan recent `.codex/plans/` and `.codex/sessions/` activity before deciding whether the note is necessary.
- Start/query `tools/zircon-session.ps1`, register the Session with its numbered `docs/plans` owner, and keep enum status, heartbeat, write scope, leases, and model selection in the service. This Markdown file carries only the warning the service cannot express.
- Keep the note in `.codex/sessions/` only while the task is active or blocked.
- Update the note only when the current step, material blocker, touched modules, or coordination warning changes.
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
related_plan: docs/plans/example/01-numbered-plan.md
---

# Coordination Warning

- Goal / current step: ...
- Touched modules: ...
- Material warning or blocker: None. A cross-plan `failure-*` handoff alone is not a session blocker.
- Next step: ...
```

## Editing Guidance

- Keep the four bullets stable so other sessions can scan quickly; do not add YAML fields that mirror coordinator state.
- Prefer facts, file paths, Failure links, and explicit warnings over prose. Never paste test logs, command transcripts, plan content, or a duplicate change summary.
- Treat this note as coordination state only; it must not become the sole permanent owner of a concrete output record.
- Update `updated_at` each time the note changes materially.
- If the task expands into a new subsystem, add the new touched module explicitly instead of assuming readers will infer it.
- The coordinator task state, not this note, enforces the `5.6-sol` / `5.6-terra` / `5.6-luna` model tier and allowed thinking depth.
