---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: play-pending-edit-decision-notification-contract
origin_plan: docs/plans/zircon_editor/editor/04-pie-and-simulation.md
origin_workflow_node: M1
fixing_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_child_dir: docs/plans/zircon_editor/editor/04
fixing_child_dir: docs/plans/zircon_editor/editor/17
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/play/transition_report.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/retained_host/workbench_notifications.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/notifications.rs
tests:
  - pending-edit Decision notification publish and receipt roundtrip
  - apply/discard callback routing and repeated receipt idempotence
  - stop/crash prompt parity and next-Play blocking integration
---

# Editor17：Play pending edit Decision 通知与回执契约缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
- 来源执行者：`editor04-m1-play-edit-protection-pending-r1-20260722`
- 来源执行切片：M1.3 `PlayEditPolicy`、`pending_edits` 与退出决策提示
- 修复责任计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 目标里程碑：Editor17 M3.2 notifications 三类契约
- 交接原因：Decision 的数据源、呈现生命周期、用户回执与去重属于 Editor17 通知中心统一 owner，Editor04 不得在菜单路径私造临时对话框或 status-line 兼容语义。

## 失败现象与复现证据

Editor04 current-source snapshot `856` 已实现并复审通过 typed `PendingEditDecisionPrompt`：成功 stop/crash 后报告 pending 数量，未决或 resolving 状态会阻断下一次 Play，controller 提供 typed apply/discard resolution API。

当前 `MenuAction::ExitPlayMode` 在 `request_stop()` 后只消费 activation diagnostics，未消费 `transition.pending_edit_prompt`。现有 `WorkbenchNotification` 只有 severity、toast queue 与 notification history；仓库内没有 Editor17 计划要求的 `Decision` kind、选项 payload、receipt ID、选择回调或幂等回执 API。因此内核提示无法合法呈现和回传 apply/discard 选择。

这不是 Editor04 policy 缺陷：把 pending 数量写入 status line 不能产生用户选择；在 `menu_action.rs` 直接调用原生 message box 会绕开 retained UI、headless、测试与通知中心单源契约。

## 最低共享层根因

Editor17 M3.2 仍为 planned。`core/notifications/{center,kinds}` 的 `Toast / Progress / Decision` 三类注册契约尚未落地，现有 retained `WorkbenchNotification` 是呈现雏形而非可回执 Decision authority。

## 架构修复验收

- 在 Editor17 owner 建立 typed `DecisionNotification`：稳定 notification/receipt ID、标题/正文 key、typed option IDs、默认/取消选项、pending/resolved 生命周期和 producer ownership。
- receipt 必须幂等；重复点击、UI 重建或 replay 不得重复执行 apply/discard。关闭窗口必须按显式取消策略返回，不能静默清空队列。
- Editor04 adapter 消费 `PendingEditDecisionPrompt` 并发布一个 Decision；Apply 回执调用 `PlaySessionController::apply_pending_edits`，Discard 回执调用 `discard_pending_edits`，逐项 apply failure 汇入结果通知并保留完整失败 intent 诊断。
- stop 与 backend crash 两条退出路径复用同一 producer；pending 未决期间下一次 Play 保持 typed 阻断，回执终结后才放行。
- retained UI、headless 测试与通知中心 history 消费同一数据源，不在 `menu_action.rs` 或平台窗口层建立第二套状态。
- 契约测试覆盖 publish -> present -> receipt -> apply/discard -> next Play；重复 receipt 幂等；apply callback 阻塞期间 resolving barrier 继续生效。

## 禁止临时方案

- 不得用 status line、toast 自动消退或日志条目替代必须选择的 Decision。
- 不得在 Editor04 直接调用 Win32/平台 message box、保存 bool 或维护第二个 pending 列表。
- 不得默认 apply 或默认 discard，也不得为了 UI 解锁绕过 `PendingEditResolutionInProgress` / `PendingEditDecisionRequired`。

## 修复结果与回传

Open state：等待 Editor17 M3.2 notifications owner 落地 typed Decision + receipt authority，并以 fixed return 回传 Editor04。Editor04 typed producer 与 resolution barrier 已冻结在 snapshot `856`；在回传前 M1.3 状态保持 `core source complete / UI decision consumer blocked`，不得宣称产品闭环。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-22 | Editor04 M1.3 -> Editor17 M3.2 failure handoff | open | 实地确认 Exit Play 未消费 `pending_edit_prompt`，现有 WorkbenchNotification 无 Decision kind/option/receipt；Editor04 snapshot 856 已提供 typed prompt、apply/discard API 与 resolving barrier。缺口路由到 Editor17，禁止 status-line/平台对话框临时双轨。 |
| 2026-07-26 | Editor17 M3.2 P0 Decision authority | `source_complete_static_green / managed_validation_blocked` | 已新增 `core/notifications/decision` 的稳定 ID、typed option、pending/resolved receipt 与幂等重放；`EditorContext` 持有通知 authority；stop 与 backend crash 共用 producer；保留式 NotificationCenter 投影为 Apply/Discard 行，关闭/ Escape 不会静默消解，Apply failure 写入完整 intent 诊断通知。模块与保留式测试、`rustfmt --check`、`git diff --check` 均通过。受管 Cargo/原子 commit 暂不能冻结：共享 `core/mod.rs` 还引用未提交的 `recovery/settings`（本计划）及 `script_build/sync`（Editor13/Editor02）子树。外部 façade 闭包已由 Editor13 `failure-2026-07-22-script-build-facade-validation-copy-closure.md` 跟踪；不得删除 façade、吸收外部业务文件或把共享工作树当验证输入。 |
