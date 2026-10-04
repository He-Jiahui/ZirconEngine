# Handle Plan Failure Handoffs

## Core Rule

Route cross-plan failures to the numbered child plan that owns the lowest shared cause. Do not pause unrelated source-plan progress, and do not accept a local bypass in place of an architectural repair.

Read `references/handoff-template.md` before creating or closing a handoff.

## Failure Priority Gate

An applicable open handoff takes priority at the fixing plan's next repair window, immediately when no independent slice remains. Record the active repair and exact diagnostic/fix scope in the canonical failure artifact, repair the owning contract, validate upward, and return the handoff before accepting dependent work. Both owners may continue genuinely independent authorized slices.

## Historical Coordinator Validation Failures

- Treat a canonical `failure-*.md` materialized from coordinator validation as an immediate forward-fix continuation for its fixing Plan, even when the original source snapshot is already on `main` as `integrated_validation_pending`.
- Preserve the original integrated commit for ordinary test or behavioral failures. Repair forward and return `fixed-*`; consider rollback only for ownership/provenance failure, false compile evidence, or demonstrated irreversible repository or persistent-data damage.
- Do not wait for another validation ticket before diagnosing the Failure. The fixing primary follows the repair-window priority above; the origin Plan continues dependency-independent work.

## Start-of-Session Priority

Before normal feature work, scan the current numbered child-plan directory for `failure-*.md`.

The coordinator is retired. Inspect canonical Markdown failures and their owner/links directly; no import, registration, lease or failure-query API is required. Preserve archived SQLite graph records for migration.

- Resolve applicable failures before dependent feature acceptance; preserve independent work.
- Apply `support-first-regression-testing` when an upper-layer symptom may come from shared support.
- Fix the lowest broken shared layer and validate upward through the originating failure.
- Do not mark a session blocked merely because another plan owns the failure. Publish the handoff and continue every independent owned slice.

## Create a Failure Handoff

1. Prove the failure and identify the numbered child plan that owns its lowest shared cause.
2. Create `docs/plans/{fixing-family}/{fixing-id}/failure-{YYYY-MM-DD}-{summary}.md`.
3. Keep `{summary}` lowercase, hyphenated, specific, and stable for the entire failure/fix lifecycle.
4. Record the originating executor plan and slice, the fixing plan, reproduction evidence, lowest known cause, architectural acceptance criteria, and forbidden temporary workarounds.
5. Default: add a concise open-status summary and relative link in both numbered plan documents. When repository policy prohibits writes to global plan definitions, set `plan_link_mode: child_record_only` in the artifact instead; the canonical child-plan record carries the status, and no plan-definition links are required.
6. Continue independent work in the originating session. Do not claim its affected gate passed.

Use a handoff only for a repository failure owned by another numbered plan. Fix current-plan failures locally. Treat external outages without a repository owner as environment evidence, not a plan handoff.

## Resolve and Return

1. Repair the architecture; do not add aliases, compatibility shims, silent fallback, test-only bypasses, duplicated truth, or one-call-site exceptions.
2. Run focused lower-layer tests, the original reproduction, and the declared upward acceptance gate.
3. Update the artifact to `handoff_kind: fixed`, `status: fixed`, and add `resolved_at`, root cause, changed owners, commands, and results.
4. Preserve the stable lifecycle key and the root cause, changed owners, commands and results. Move the canonical artifact within the authorized scope to `docs/plans/{origin-family}/{origin-id}/fixed-{YYYY-MM-DD}-{summary}.md`; validate both relative links before the move and preserve the original artifact if any step cannot complete.
5. Update links only in authorized output records. In `child_record_only` mode, retain the canonical artifact and its lifecycle identity without editing read-only plan definitions.
6. Resume the originating plan's affected gate using the returned fixed artifact.

The moved `fixed-*` file is canonical. Do not leave a duplicate in the fixing directory.

## Validate

Run:

```powershell
python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py --repo-root E:\Git\ZirconEngine
```

Resolve naming, provenance, placement, duplicate, and link errors in the touched handoff scope. Record unrelated preexisting findings for their owner; do not expand this task or claim the repository-wide audit is clean.

## Red Flags

- Date-first or `*-handoff.md` filenames
- Missing origin/fixer plan identity
- Stopping all source work after publishing a handoff
- Accepting dependent work while its applicable failure remains open
- Deferring a failure beyond its repair window without a real dependency
- Calling a fallback, alias, shim, or special case a fix
- Copying instead of moving the fixed artifact back
- Absolute or stale plan links
