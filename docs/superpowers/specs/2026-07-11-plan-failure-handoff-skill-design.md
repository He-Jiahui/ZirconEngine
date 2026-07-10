# Plan Failure Handoff Skill Design

## Goal

Add a repository-local discipline for cross-plan failures so one session's failure does not pause unrelated feature progress. Route each failure to the numbered child-plan directory that owns the lowest shared cause, make that owner resolve it first with an architectural fix, and return verified resolution evidence to the originating executor.

## Artifact Lifecycle

1. An originating executor encounters a failure owned by another numbered child plan.
2. It identifies the owning plan family and child-plan id, then creates:
   `docs/plans/{owner-plan-family}/{owner-id}/failure-{YYYY-MM-DD}-{summary}.md`.
3. The failure artifact identifies both the originating executor plan and the target fixing plan, records the lowest known cause, reproduction evidence, architectural acceptance criteria, and forbidden temporary workarounds.
4. The originating executor records the handoff and continues all independent slices. It must not mark the whole session blocked merely because another plan owns this failure.
5. A session working in the target child plan scans its directory for `failure-*.md` before normal feature work and resolves those failures first. The repair must fix the lowest shared architectural layer and validate upward; aliases, fallbacks, compatibility shims, test-only bypasses, and one-call-site patches do not qualify.
6. After validation, the fixer updates the artifact with root cause, implementation, and evidence; moves it to the originating executor's numbered child-plan directory; and renames it to:
   `fixed-{YYYY-MM-DD}-{summary}.md`.
7. The fixing child plan retains only a concise fixed-status summary and a relative link to the moved `fixed-*` artifact. The originating plan uses that returned artifact to resume and close its original gate.

The fixed date is the date the repair was accepted. The summary slug remains stable across the move so the failure and fix are traceable as one lifecycle.

## Skill Shape

Create a focused `handle-plan-failure-handoffs` child skill under `.codex/skills/zircon-project-skills/`. Keep its `SKILL.md` concise and put the required Markdown schema in a reusable template reference. Add a read-only validator that checks filenames, required provenance fields, numbered-directory placement, fixed-record return routing, and relative links retained by the fixer.

Update the parent project-skill index and `cross-session-coordination` so every overlap-sensitive session scans numbered plan folders for failure artifacts in addition to live `.codex/sessions` notes. Update `write-plan-output-records` to classify failure/fixed handoffs as specialized child-plan-owned evidence whose move lifecycle overrides ordinary archive naming while preserving the ten-record and canonical-owner rules.

## Existing Artifact Migration

Rename the three current `2026-07-11-editor-m1-failure-handoff.md` files to the required `failure-2026-07-11-editor-m1-*.md` form with distinct stable summaries, and update all parent-plan links. Preserve their evidence and add explicit originating/fixing plan provenance required by the new template.

Do not manufacture `fixed-*` artifacts for unresolved failures.

## Validation

- Validate the new skill metadata with the standard skill validator.
- Exercise the handoff validator against the migrated real artifacts.
- Add negative fixture checks for old date-first names, missing source/fixer provenance, misplaced files, non-relative fixer links, and fixed artifacts left canonically in the fixing directory.
- Run the plan-output audit and scoped Markdown/diff checks.
- Confirm every old failure-handoff link and filename is gone and every migrated link resolves.

## Non-Goals

- Do not use handoff artifacts for failures owned by the current child plan; record and fix those locally.
- Do not replace `.codex/sessions` live coordination notes or ordinary child-plan output records.
- Do not let a handoff authorize broad edits outside the fixing plan's architectural ownership.
- Do not treat external infrastructure outages with no repository owner as cross-plan failure handoffs.
