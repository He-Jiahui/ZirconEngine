---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: settings-registry-job-category-quota-migration
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_workflow_node: M1.1
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/jobs/limits.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/settings/
tests:
  - User quota setting range validation and current-shell persistence
  - EditorJobSystem construction consumes resolved limits
  - invalid quota falls back without admitting zero-capacity jobs
---

# Editor14: JobCategory 配额尚未迁入 SettingsRegistry

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：Editor17 M1.1 JobCategory quota User-setting migration
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：Editor14 是 jobs admission 与 resolved limits 的唯一权威，Editor17 只提供 Settings schema、作用域与持久化。

## 失败现象与复现证据

`EditorJobLimits` 在 `core/jobs/limits.rs:5-39` 将 Thumbnail=2 与 Export=1 固化为私有常量，`with_runtime_defaults` 再把 Import 绑定 runtime parallelism；`EditorJobSystem::with_scheduler_and_bus` 在 `system/mod.rs:39-59` 直接消费这一对象。Editor17 计划要求类别配额成为 User 设置项，但尚无 SettingsRegistry 定义或热应用路径。

这不是可由 Editor17 单独替换的常量：jobs admission 是 Editor14 的权威，Settings 只能提供 schema、作用域和持久化，不能另建第二个 scheduler limits 真相。

## 最低共享层根因

用户可配置的 category quota 仍固化在 Editor14 私有常量中，SettingsRegistry 没有 typed schema，也没有把 resolved limits 接入唯一 admission owner。

## 架构修复验收

- Editor14 为允许配置的类别定义 User scoped、最小值为 1 的整数 Settings 项；保留 runtime-derived Import 默认逻辑的明确优先级。
- 系统构造与设置变更消费同一已解析 limits；`requires_restart` 语义必须明示，不能在运行队列中无序改写配额。
- 删除硬编码用户默认的平行配置入口，不保留私有文件或环境变量覆盖。
- 覆盖非法 0/负数拒绝、User current-shell round-trip、默认回退、启动/热应用时的 admission 限制。

## 禁止临时方案

- 不得在 `EditorJobSystem` 外新增第二个配额 map。
- 不得把 Settings 值仅用于 UI 显示而继续以常量控制 admission。

## 修复结果与回传

Open state: `等待 Editor14 将 typed User quota 接入唯一 admission owner，并完成范围、持久化、启动或热应用及 current-source Cargo 验证`。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-23 | Editor17 M1.1 -> Editor14 quota migration handoff | open | `DEFAULT_THUMBNAIL_LIMIT`、`DEFAULT_EXPORT_LIMIT` 与 runtime Import 默认仍是唯一 admission 输入；User SettingsRegistry 已可承载值，等待 Editor14 接线与旧入口删除。 |
