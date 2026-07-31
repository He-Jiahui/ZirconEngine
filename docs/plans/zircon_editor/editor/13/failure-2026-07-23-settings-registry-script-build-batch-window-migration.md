---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: settings-registry-script-build-batch-window-migration
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_workflow_node: M1.1
fixing_plan: docs/plans/zircon_editor/editor/13-script-compilation-management.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/13
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/script_build/
  - zircon_editor/src/core/settings/
tests:
  - User batch-window setting range and current-shell persistence
  - orchestrator consumes resolved debounce/window policy
  - setting change preserves bounded admission and generation cancellation
---

# Editor13: ScriptBuild 合批窗口尚未迁入 SettingsRegistry

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：Editor17 M1.1 script-build batch-window User-setting migration
- 修复责任计划：`docs/plans/zircon_editor/editor/13-script-compilation-management.md`
- 交接原因：Editor13 拥有 debounce、batch window、generation 与 bounded admission 策略，Editor17 只提供共享 Settings owner。

## 失败现象与复现证据

Editor13 的编译编排仍拥有 debounce、合批与 admission 背压策略；其 2026-07-22 open failure 已明确窗口需要 first-event max latency、entry/bytes/age 预算与 generation single-flight。Editor17 计划要求“13 合批窗口（User）”成为 SettingsRegistry 首批项，但当前 `core/script_build/` 尚未从 User settings 读取该窗口，也没有以设置变更更新策略的单一入口。

在未建立显式范围和 lifecycle 的前提下，Editor17 不会给编排器塞一个未消费的数字设置，更不会绕过 Editor13 的既有 bounded admission 约束。

## 最低共享层根因

script-build batch window 尚未成为有界 typed User setting，编排器也没有从 SettingsRegistry 消费 resolved policy 的单一入口。

## 架构修复验收

- Editor13 将 batch/debounce window 定义为有界 User Setting，明确默认、最小/最大和 `requires_restart`/热应用语义。
- 编排器只消费 SettingsRegistry 的解析结果；设置变更不得绕过 first-event latency、entry/bytes/age 预算或 generation single-flight。
- 删除任何私有持久化或环境变量配置路径，不与 User SettingsStore 双写。
- 覆盖 current-shell round-trip、范围拒绝、首次事件 deadline、队列预算和设置变更下取消/合并不变量。

## 禁止临时方案

- 不得把窗口设置只作为日志/面板显示值。
- 不得为迁移放宽队列预算、增加无界缓存或保留旧配置回退。

## 修复结果与回传

Open state: `等待 Editor13 以 typed bounded setting 接线 batch window，并证明设置变更不破坏 deadline、预算、取消与 single-flight 不变量`。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-23 | Editor17 M1.1 -> Editor13 batch-window migration handoff | open | 编译合批仍由 Editor13 编排策略唯一拥有，且其背压 failure 未关闭；User SettingsRegistry 迁移位已确定，等待 Editor13 以不放宽准入契约的方式接线。 |
