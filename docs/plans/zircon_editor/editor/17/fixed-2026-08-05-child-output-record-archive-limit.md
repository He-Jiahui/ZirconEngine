---
handoff_kind: fixed
status: fixed
created_at: 2026-08-05
summary_slug: child-output-record-archive-limit
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_runtime/text/08-ime-and-text-input.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_runtime/text/08
plan_link_mode: child_record_only
related_code:
  - docs/plans/zircon_runtime/text/08-ime-and-text-input.md
tests:
  - python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
resolved_at: 2026-08-05
---


# Text08: Child output records exceed the archive limit

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M3.3 command localization handoff record audit
- 修复责任计划：`docs/plans/zircon_runtime/text/08-ime-and-text-input.md`
- 交接原因：the Text08 child plan owns its direct output table and its numbered archive. Editor17 discovered the violation while applying the repository-wide output-record audit and must not alter a foreign plan definition.

## 失败现象与复现证据

`audit_plan_output_records.py --repo-root E:\Git\ZirconEngine` reports:

```text
docs/plans/zircon_runtime/text/08-ime-and-text-input.md:216: child-record-limit: child plan contains 13 direct records; move all records to 08/
```

The `## 状态与产出记录` table directly retains thirteen milestone records. The required `docs/plans/zircon_runtime/text/08/` archive directory does not exist, so the plan cannot meet the repository ten-record limit or provide one canonical owner for each archived record.

## 最低共享层根因

Text08 appended accepted milestone evidence directly to its numbered plan past the ten-record threshold instead of performing the all-record archive migration. This is documentation ownership maintenance, not an IME runtime defect; changing or deleting individual rows from another plan would lose canonical evidence.

## 架构修复验收

- Create the matching `docs/plans/zircon_runtime/text/08/` child archive and move all direct concrete output records from the Text08 plan table into coherent dated archive artifact(s), without changing evidence status or dropping commands, diagnostics, or remaining work.
- Retain `## 状态与产出记录` in the Text08 plan with the exact repository notice, a concise current-state overview, and relative links to the numbered archive artifact(s).
- Keep this failure artifact as the only open handoff record in the child archive; do not copy completed milestone rows back into the plan definition.
- Run the output-record audit and require the prior `child-record-limit` diagnostic to be absent. Re-run the handoff validator before lifecycle return.

## 禁止临时方案

- Do not delete, collapse, or rewrite historical record evidence merely to reduce the count.
- Do not move only the overflow rows; the archive migration must move all direct concrete records together.
- Do not place concrete records in `index.md`, an `engine-code-*.md` overview, or an unrelated child-plan directory.
- Do not change Text08 runtime source code or assert IME validation results as part of this documentation repair.

## 修复结果与回传

- 根因：Text08 retained 13 concrete milestone rows directly in its numbered plan instead of performing the mandatory all-record archive migration after exceeding ten records.
- 架构修复：Move all 13 rows unchanged into the Text08 numbered archive and leave the plan definition with the exact repository notice, a concise current-state overview, and one relative archive link.
- 验证：Plan-output audit passed; archive has 13 dated records, plan has zero direct dated records, notice count is one, link resolves, and scoped git diff check passed.
- 回传：Text08 child-record-limit is fixed; Editor17 may resume its repository-wide output-record audit without changing any IME behavior or validation status.
