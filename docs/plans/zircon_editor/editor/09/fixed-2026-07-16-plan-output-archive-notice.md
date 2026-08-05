---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: plan-output-archive-notice
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_editor/editor_ui/11
related_code:
  - docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md
tests:
  - python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
resolved_at: 2026-07-16
---


# EditorUI11：编号归档链接缺少固定产出提示语

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：Editor09 M1.3 产出记录全仓审计
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md`
- 交接原因：ZUI 后缀收敛计划的历史记录与归档链接由 EditorUI11 维护，Editor09 不跨功能修改其计划内容。

## 失败现象与复现证据

全仓产出记录审计在 EditorUI11 第 154 行报告 `missing-archive-notice`：存在编号归档链接，但缺少规定的精确提示语。

## 最低共享层根因

EditorUI11 的 `## 产出记录与时间` 段落没有同步当前 archive notice 约束。

## 架构修复验收

- 在 EditorUI11 产出位置加入规定的精确提示语，保留既有后缀收敛记录与链接。
- 重新运行全仓产出记录审计，EditorUI11 不再报告 `missing-archive-notice`。

## 禁止临时方案

- 禁止删除归档链接、改写审计器或用近似文案替代精确提示语。
- 禁止将后缀收敛记录迁入 Editor09。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| EditorUI11 文档收敛 | archive notice | `fixed-已修复` | 2026-07-16 | 全仓产出记录审计通过；`ad2c6f989c` 证明规范提示语已于该日落地。 |

## 修复结果与回传

- 根因：EditorUI11 的状态与产出记录段曾保留编号归档链接，但未同步 plan-output 审计器要求的精确归档提示语。
- 架构修复：提交 ad2c6f989c 于 2026-07-16 在 EditorUI11 计划加入规范归档提示语；编号归档以产出记录归档标题标识自身并回链唯一所属计划，计划概述与历史明细保持分离。
- 验证：python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root .：audit passed；git blame 证明规范提示语由 ad2c6f989c 于 2026-07-16 落地。
- 回传：EditorUI11 的归档通知契约已恢复，Editor09 的产出记录全仓审计门可以继续。
