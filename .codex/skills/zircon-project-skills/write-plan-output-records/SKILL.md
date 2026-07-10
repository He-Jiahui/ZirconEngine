---
name: write-plan-output-records
description: Enforce ZirconEngine plan output-record ownership, placement, migration, and audit rules. Use whenever any Codex session creates, appends, moves, summarizes, or validates concrete output records under `docs/plans`, including milestone slice evidence, testing-stage results, session closeout records, `index.md`, `engine-code-*.md`, numbered child plans such as `01-*.md`, and numbered output archive directories.
---

# Write Plan Output Records

## Core Rule

Treat concrete output records as child-plan-owned evidence. Never use an `index.md`, an `engine-code-*.md`, or a `.codex/sessions/*.md` note as the canonical owner of those records.

Use this exact notice at every output-record position retained in an `index.md` or `engine-code-*.md` after moving concrete records out:

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Keep only a concise current-state overview and links to the owning child plan or numbered archive beside the notice.

## Concrete Record Definition

Treat an item as a concrete output record when it captures one completed or attempted work slice, test stage, migration, fix, or acceptance result and includes details such as:

- a milestone or slice name;
- a machine-readable status anchor;
- a completion or attempt date;
- commands, logs, screenshots, hashes, pass/fail counts, or other evidence;
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
4. Append the record under that child plan's existing `## 状态与产出记录` heading or equivalent output-record heading. Preserve the local table or list schema.
5. If no output-record heading exists, add `## 状态与产出记录` and use the repository-standard milestone/slice/status/date/evidence table.
6. Never append a concrete record to any `index.md` or `engine-code-*.md`. Replace an existing concrete record there with the exact notice, a current-state overview, and a link to the new owner.
7. Move records without dropping evidence, changing their status meaning, or leaving duplicate canonical copies.

The coordinator treats every `index.md`, `engine-code-*.md`, and numbered plan-definition Markdown as read-only for ordinary business Sessions. Only an explicit maintenance operation may update those files, and maintenance never relaxes the repository path boundary.

Use this standard table when a child plan does not already define a compatible schema:

```markdown
## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1 | ... | `...` | YYYY-MM-DD | ... |
```

## Ten-Record Limit

Count concrete records across the child plan's output-record section before every write.

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

Apply this workflow whenever a session writes a plan output record, including after an implementation slice, during a testing stage, and at session closeout.

- Use `.codex/sessions/*.md` only for live coordination state, current steps, blockers, touched modules, and handoff notes.
- Do not use a session note as the permanent or sole copy of a concrete plan output record.
- Before writing from a session, resolve the owning numbered child plan and recount its records.
- Write exactly one record for each completed or attempted slice at the time required by the active milestone policy.
- If the new record crosses the 10-record limit, perform the complete archive migration in the same session before reporting the write as complete.
- If a session updates an index or engine-code overview, include only the exact notice, current-state summary, and owner links.

## Audit

Run the bundled read-only audit after reorganizing records or before closing a session that wrote plan output:

```powershell
python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
```

The audit reports concrete-record signatures in forbidden files, missing exact notices, child output sections exceeding 10 records, and broken or misrouted numbered archive links. Treat findings as migration work; do not weaken the checks to preserve an invalid layout.

## Completion Checklist

- Confirm every moved record has one canonical child-plan or numbered-archive owner.
- Confirm every affected `index.md` and `engine-code-*.md` contains the exact notice and no concrete evidence rows.
- Confirm every child plan contains at most 10 direct concrete records.
- Confirm every archive link resolves and its directory number matches the child-plan prefix.
- Confirm `.codex/sessions` contains coordination state only, not the sole permanent output record.
- Run the audit and `git diff --check -- docs/plans .codex/skills` before claiming completion.
