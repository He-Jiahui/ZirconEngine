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

Open state: `current-source 修复、本地静态证据与独立审查已完成；受管静态票据排队中，等待外部 zr_vm worktree 清洁后执行 source-bound managed Cargo，再完成 failure return 与 closeout`。

## 2026-09-21 current-source 修复与受管验证回执

- Editor13 新增唯一 `ScriptBuildBatchPolicy` 与 User setting `editor.script_build.batch_window_ms`：范围 `50..=1000 ms`、步进 `50 ms`、默认 `300 ms`、`requires_restart=false`。定义通过共享 `SettingsRegistry` 注册；生产编排器只从 `SettingsAuthority::resolved_setting` 构造或热同步，不存在私有文件、环境变量或双写回退。
- pending watch 现在同时保留 first/last observation。热更新只按 `min(last + debounce, first + 1000 ms)` 重算 deadline，不改变 20-entry/64 KiB 路径预算、full-rebuild sentinel、active request、单 coalesced pending generation、request id 或取消回执。裸毫秒构造仅在 `cfg(test)` 中保留给确定性时序夹具。
- 新增 6 项 Rust 回归：bounded User schema、越界/类型拒绝、en/zh-CN 直接词条、current-shell read-your-write、新旧窗口的 first-event 硬截止、预算/single-flight/cancel identity。现有 Editor13 静态合同 `8/8`、精确 `rustfmt --check`、双 TOML 解析与 scoped `git diff --check` 均通过。
- 独立审查先发现本地化测试会经过英文 fallback，不能证明 `zh-CN` 直译存在；测试已改为直接解析两份嵌入 TOML 并逐键校验，同一审查者最终确认 Critical/Important/Moderate=`0/0/0`。旧 snapshot `3698` 因该测试修正被取代；post-review snapshot `3699` 重新封存 7 个当前产品/测试/词条输入，manifest hash=`5726760987f01e15981963b7a34483a5228253b25bd5017e630fcc9ac0f4cc49`。
- 受管静态票据 `371d11b995d94a9499e6360b0be22d3b`（请求 `9e275cbaa5bb4c2ca2d46c9c3df41b7f`）已按 snapshot `3699` 的精确清单入队，覆盖 Rustfmt、设置/编排合同和 en/zh-CN 直接词条；`staticParseOnly=true`、`upwardAcceptance=false`，当前不写成通过。先前受管请求 `c4a2f6710e0c4a89a5f85f4c4197d773` 提交精确命令 `cargo +1.94.1 test --locked -p zircon_editor --lib script_build:: -- --nocapture --test-threads=1`，但在执行前由 admission 返回 `validation_ticket_external_worktree_dirty`：外部 owner 的 `E:\Git\zr_vm` 尚有未提交改动。未生成 validation ticket 或 Cargo run，不把此回执写成动态通过，也不重复提交。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-23 | Editor17 M1.1 -> Editor13 batch-window migration handoff | open | 编译合批仍由 Editor13 编排策略唯一拥有，且其背压 failure 未关闭；User SettingsRegistry 迁移位已确定，等待 Editor13 以不放宽准入契约的方式接线。 |
| 2026-09-21 | Editor13 typed batch-window setting current-source repair | source_integrated_local_static_green_review_clear_managed_static_queued_cargo_blocked | Post-review snapshot 3699；Editor13 静态合同 8/8、rustfmt、TOML direct-key 与 diff guard GREEN；独立审查 C/I/M=`0/0/0`。受管静态票据 `371d11b...` queued；受管 Cargo 请求 `c4a2f...` 在执行前因 `E:\Git\zr_vm` foreign dirty 被拒绝；动态验收、return/closeout 仍 open。 |
