---
name: write-plan-output-records
description: Use when ZirconEngine work needs a minimal canonical plan record for an accepted milestone, cross-plan failure/fix, or terminal closeout under `docs/plans`.
---

# Write Plan Output Records

## Core Rule

Treat one accepted milestone outcome as child-plan-owned evidence. Never use an `index.md`, an `engine-code-*.md`, or a `.codex/sessions/*.md` note as the canonical owner of that evidence.

Keep plan output minimal: one short status row per accepted milestone, plus a failure/fixed handoff only when cross-plan work truly needs it. A test result by itself is not a record event. Do not create a separate narrative report, command-log transcript, per-slice progress row, or duplicated documentation record.

Use this exact notice at every output-record position retained in an `index.md` or `engine-code-*.md` after moving concrete records out:

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Keep only a concise current-state overview and links to the owning child plan or numbered archive beside the notice.

## Concrete Record Definition

Treat an item as a concrete output record when it captures an accepted milestone outcome, migration, critical fix, or acceptance result and includes details such as:

- a milestone or accepted scope name;
- a machine-readable status anchor;
- a completion or attempt date;
- a concise evidence reference such as a validation suite, action ID, or linked handoff;
- a specific list of changed files, resolved failures, remaining blockers, or acceptance claims.

Do not treat a short aggregate status, current milestone summary, child-plan link, or risk overview as a concrete record when it does not reproduce individual evidence rows.

## Failure / Fixed Handoff Exception

Apply `../handle-plan-failure-handoffs/SKILL.md` whenever a record uses the `failure-*` or `fixed-*` lifecycle.

- While open, the canonical artifact belongs in the fixing numbered child directory as `failure-{date}-{summary}.md`.
- After architectural repair and upward validation, move the same artifact to the origin numbered child directory as `fixed-{date}-{summary}.md`.
- The fixing plan retains only a concise fixed-status summary and a relative link to the moved artifact; never retain a duplicate canonical copy.
- These filenames and move semantics override the ordinary `{date}-{summary}.md` archive naming rule. Do not rename or relocate a handoff merely because a child plan crosses the ten-record limit.
- A concise plan link/status line is not a concrete output row and does not change the ordinary ten-record count. The artifact itself remains canonical evidence and must preserve all reproduction and fix results.

## Placement Workflow

1. Identify the owning plan family under `docs/plans/{plans_path}`.
2. Select the child plan whose numbered filename and topic own the work, such as `01-*.md` through `NN-*.md`.
3. When the coordinator is available, register that numbered plan and run `tools/zircon-session.ps1 plan authorize <target>`. A denied decision is a routing error; do not bypass it with direct editing.
4. Append one short accepted-milestone status row under that child plan's existing `## 状态与产出记录` heading or equivalent output-record heading. Preserve the local table or list schema.
5. If no output-record heading exists, add `## 状态与产出记录` and use the repository-standard milestone/slice/status/date/evidence table.
6. Never append a concrete record to any `index.md` or `engine-code-*.md`. Replace an existing concrete record there with the exact notice, a current-state overview, and a link to the new owner.
7. Move records without dropping evidence, changing their status meaning, or leaving duplicate canonical copies.

The coordinator treats every `index.md`, `engine-code-*.md`, and numbered plan-definition Markdown as read-only for ordinary business Sessions. Only an explicit maintenance operation may update those files, and maintenance never relaxes the repository path boundary.

Use this standard table when a child plan does not already define a compatible schema:

```markdown
## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | ... | `...` | YYYY-MM-DD | ... |
```

## Ten-Record Limit

Count concrete records before archive maintenance, not before ordinary implementation progress.

- Keep all records directly in the child plan when the resulting count is 10 or fewer.
- When the resulting count exceeds 10, move all concrete records from that section, not only the overflow, into the numbered child output directory.
- Use `docs/plans/{plans_path}/{nn}/{date}-{summary}.md`, where `{nn}` matches the child-plan prefix, `{date}` is `YYYY-MM-DD`, and `{summary}` is a short lowercase hyphenated topic.
- Leave the child plan's output-record heading, the exact notice, a concise current-state overview, and relative links to the archive files.
- Split archives by coherent topic or date only when one archive would become difficult to scan. Do not duplicate records between archives.

Example:

```text
docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
docs/plans/zircon_runtime/runtime/07/2026-07-10-owner-budget-validation.md
```

## Session Requirements

Apply this workflow only when a session writes an accepted milestone, critical failure/fix, or terminal closeout output record.

- Use `.codex/sessions/*.md` only for live coordination state, current steps, blockers, touched modules, and handoff notes.
- Do not use a session note as the permanent or sole copy of a concrete plan output record.
- Resolve the owning numbered child plan before writing a record.
- Do not write records for ordinary implementation slices, testing-stage runs, isolated test attempts, document-only cleanup, or transient diagnostics. Write one concise accepted milestone outcome after its validation gate; failures and closeouts remain separate lifecycle events.
- If accepted outcomes make a child plan difficult to scan, perform the archive migration before closing that plan or as a dedicated maintenance task.
- If a session updates an index or engine-code overview, include only the exact notice, current-state summary, and owner links.

## Audit

Run the bundled read-only audit after reorganizing records, before closing a plan that owns records, or when a plan-output guard reports a violation:

```powershell
python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
```

The audit reports concrete-record signatures in forbidden files, missing exact notices, child output sections exceeding 10 records, and broken or misrouted numbered archive links. Treat findings as migration work; do not weaken the checks to preserve an invalid layout.

## Completion Checklist

- Confirm every moved record has one canonical child-plan or numbered-archive owner.
- Confirm every affected `index.md` and `engine-code-*.md` contains the exact notice and no concrete evidence rows.
- Confirm every child plan contains concise milestone-level records and no per-slice progress rows.
- Confirm every archive link resolves and its directory number matches the child-plan prefix.
- Confirm `.codex/sessions` contains coordination state only, not the sole permanent output record.
- Run the audit and `git diff --check -- docs/plans .codex/skills` before claiming completion.
