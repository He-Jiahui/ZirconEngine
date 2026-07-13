---
name: handle-plan-failure-handoffs
description: Use when a ZirconEngine plan executor encounters a failure owned by another numbered child plan, starts work in a child-plan directory containing `failure-*.md`, or returns a verified cross-plan fix.
---

# Handle Plan Failure Handoffs

## Core Rule

Route cross-plan failures to the numbered child plan that owns the lowest shared cause. Do not pause unrelated source-plan progress, and do not accept a local bypass in place of an architectural repair.

Read `references/handoff-template.md` before creating or closing a handoff.

## Failure Priority Gate

An open `failure-*.md` in the registered fixing child plan is the first executable item. Set the Session to `resolving_failure`, claim only the diagnostic/fix scope, and complete architectural repair, upward validation, and `failure return` before starting any normal fixing-plan slice. The originating plan may continue only slices that do not depend on the failed contract.

## Start-of-Session Priority

Before normal feature work, scan the current numbered child-plan directory for `failure-*.md`.

When the local coordinator is available, run `tools/zircon-session.ps1 failure import` followed by `failure open <fixing-plan>`. Markdown remains canonical; SQLite is only the queryable graph index.

- Resolve every applicable open failure before advancing that child plan's normal features.
- Apply `support-first-regression-testing` when an upper-layer symptom may come from shared support.
- Fix the lowest broken shared layer and validate upward through the originating failure.
- Do not mark a session blocked merely because another plan owns the failure. Publish the handoff and continue every independent owned slice.

## Create a Failure Handoff

1. Prove the failure and identify the numbered child plan that owns its lowest shared cause.
2. Create `docs/plans/{fixing-family}/{fixing-id}/failure-{YYYY-MM-DD}-{summary}.md`.
3. Keep `{summary}` lowercase, hyphenated, specific, and stable for the entire failure/fix lifecycle.
4. Record the originating executor plan and slice, the fixing plan, reproduction evidence, lowest known cause, architectural acceptance criteria, and forbidden temporary workarounds.
5. Add a concise open-status summary and relative link in both numbered plan documents.
6. Continue independent work in the originating session. Do not claim its affected gate passed.

Use a handoff only for a repository failure owned by another numbered plan. Fix current-plan failures locally. Treat external outages without a repository owner as environment evidence, not a plan handoff.

## Resolve and Return

1. Repair the architecture; do not add aliases, compatibility shims, silent fallback, test-only bypasses, duplicated truth, or one-call-site exceptions.
2. Run focused lower-layer tests, the original reproduction, and the declared upward acceptance gate.
3. Update the artifact to `handoff_kind: fixed`, `status: fixed`, and add `resolved_at`, root cause, changed owners, commands, and results.
4. Use the coordinator `failure return` command with the stable lifecycle key and four non-empty resolution fields. It atomically moves the artifact to `docs/plans/{origin-family}/{origin-id}/fixed-{YYYY-MM-DD}-{summary}.md`, updates both relative links, and rolls back on a partial file failure.
5. Replace the fixing plan's open entry with a concise fixed-status summary and a relative link to the moved artifact.
6. Update the originating plan to link to the returned fixed artifact and resume its affected gate.

The moved `fixed-*` file is canonical. Do not leave a duplicate in the fixing directory.

## Validate

Run:

```powershell
python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py --repo-root E:\Git\ZirconEngine
.\tools\zircon-session.ps1 failure audit -Json
```

Treat every reported naming, provenance, placement, duplicate, or link error as unfinished handoff work.

## Red Flags

- Date-first or `*-handoff.md` filenames
- Missing origin/fixer plan identity
- Stopping all source work after publishing a handoff
- Advancing a fixing plan while its `failure-*.md` remains open
- Treating a failure handoff as a backlog item while starting a new normal fixing-plan slice
- Calling a fallback, alias, shim, or special case a fix
- Copying instead of moving the fixed artifact back
- Absolute or stale plan links
