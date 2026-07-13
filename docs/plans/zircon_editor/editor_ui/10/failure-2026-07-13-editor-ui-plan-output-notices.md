---
handoff_kind: failure
status: open
created_at: 2026-07-13
summary_slug: editor-ui-plan-output-notices
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_editor/editor_ui/10
related_code:
  - docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
  - docs/plans/zircon_editor/editor_ui/index.md
tests:
  - python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
---

# EditorUI10：结构计划与 EditorUI 索引缺少固定产出提示语

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：Editor09 M1.3 产出记录全仓审计
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 交接原因：EditorUI 计划结构与索引格式属于 EditorUI10 的文档/模块约束所有权，Editor09 只回传审计证据。

## 失败现象与复现证据

全仓审计报告两项：EditorUI10 第 161 行存在编号归档链接但缺少 `missing-archive-notice`；`editor_ui/index.md` 第 271 行的产出位置缺少规定的精确提示语。

## 最低共享层根因

EditorUI 的结构约定计划与顶层索引没有同时采用当前 numbered-plan output notice 规则。

## 架构修复验收

- 在 EditorUI10 的产出位置补齐精确 archive notice，保留现有链接与概述。
- 在 `editor_ui/index.md` 的产出位置补齐同一精确提示语，不把详细记录回填到 index。
- 重新运行全仓产出记录审计，两项 notice 诊断均消失。

## 禁止临时方案

- 禁止删除编号归档链接、放宽审计器或复制详细记录到 index。
- 禁止把 EditorUI 结构记录放入 Editor09 子目录。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| EditorUI10 文档收敛 | plan/index output notice | `open-待功能owner处理` | 2026-07-13 | 审计分别在 EditorUI10:161 与 `editor_ui/index.md:271` 报告 notice 缺失。 |

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
