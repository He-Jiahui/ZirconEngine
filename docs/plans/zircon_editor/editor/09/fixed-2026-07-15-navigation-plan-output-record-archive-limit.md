---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: navigation-plan-output-record-archive-limit
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_plugins/05-navigation.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_plugins/05
related_code:
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/05/
tests:
  - python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:/Git/ZirconEngine
resolved_at: 2026-07-15
---

# Plugins05：Navigation 产出记录归档上限治理回传

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `FIXED / 已修复` | 2026-07-15 | Navigation 父计划保留 exact notice、当前状态概述与编号子目录链接；M1-M6 的 7 条 concrete output records 均位于 `docs/plans/zircon_plugins/05/`。当前 plan-output audit 不再报告 Plugins05。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：Editor09 M1 plan-output ownership audit
- 修复责任计划：`docs/plans/zircon_plugins/05-navigation.md`
- 交接原因：Navigation 的 numbered child records 与父计划状态概述由 Plugins05 持有，Editor09 不应移动或删除其业务证据。
- 生命周期键：`navigation-plan-output-record-archive-limit`

## 失败现象与复现证据

历史 audit 曾报告 `05-navigation.md` 含 11 条 direct records，超过 10 条阈值；原 failure 因而要求
Plugins05 把所有 concrete records 收束到 `05/`，父计划只保留 exact notice、概述和 archive links。
当前源码已满足该布局，但 open lifecycle 未同步返回。

## 最低共享层根因

原 failure 记录了历史时点的 11 条 direct records；随后 Plugins05 已把 canonical concrete records
全部收束到编号子目录，并在父计划状态节只保留概述与链接，但 failure 生命周期没有同步返回，形成
“审计已不报错、计划仍标 open”的状态漂移。

## 架构修复验收

- `docs/plans/zircon_plugins/05/` 持有全部 canonical M1-M6 concrete output records。
- `05-navigation.md` 保留 exact notice、精简现状与编号子目录链接，不复制具体证据。
- fresh plan-output audit 不再报告 Plugins05；全库其他违规继续如实保留。
- failure lifecycle 以稳定 slug 返回 Editor09 origin child，handoff validator 为 0 errors。

## 禁止临时方案

- Do not delete historical evidence, weaken the ten-record audit threshold, or keep only the newest record outside the numbered child directory.
- Do not claim the whole-repository plan-output audit passes while unrelated violations remain.
- 禁止把 Navigation 业务记录迁入 Editor09 或以重复摘要建立第二事实源。

## 修复结果与回传

- 根因：Plugins05 已完成 numbered-child 归档，但旧 open failure 与两个父计划链接未跟随当前审计事实收口。
- 架构修复：保留 `05/` 为 concrete record 唯一 owner，并把父计划治理状态与 failure lifecycle 同步为 fixed。
- 验证：fresh plan-output audit 只报告 6 条其他 Editor 计划既存违规，不再包含 Plugins05；Failure handoff validator 为 0 errors；精确文档 diff check 通过。
- 回传：按稳定 slug 返回 Editor09 origin child；Plugins05 M6 仍因 Navigation 选择态 operation 参数与 viewport provider host 等业务 failure 保持进行中，本回传不提升其完成状态。
