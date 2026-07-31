---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: play-mode-message-producer-missing
origin_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
fixing_plan: docs/plans/zircon_editor/editor/04-pie-and-simulation.md
origin_child_dir: docs/plans/zircon_editor/editor/12
fixing_child_dir: docs/plans/zircon_editor/editor/04
related_code:
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/core/editor_message/message/mode.rs
  - zircon_editor/src/core/editor_message/topics.rs
tests:
  - cargo test -p zircon_editor --lib --locked play
  - cargo test -p zircon_editor --lib --locked editor_message
---

# Editor04: PlaySessionController does not publish typed mode transitions

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 来源执行切片：M1.2 plugin lifecycle message bridge
- 修复责任计划：`docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
- 交接原因：Editor12 只订阅稳定的 editor facts 并把它们转换为外部 plugin lifecycle callbacks；Edit/Building/Playing 的唯一状态权威和其消息出口属于 Editor04。

## 失败现象与复现证据

`EditorPluginLifecycleMessageBridge` 已订阅 `editor.mode` 并可将 `ModeMessage::PlayStateChanged` 的 Playing 边界转换为 `EnteredPlayMode` / `ExitedPlayMode`。当前仓库对 `PlayStateChanged`、`TOPIC_MODE`、`editor.mode` 的生产代码搜索只有类型、topic 和测试夹具，没有 `PlaySessionController` 或 host transition 的 publish 调用。

这使 Play 启停、构建成功/失败和 backend terminal transition 即使已经改变 controller state，也不会到达 Editor12 subscriber；桥接实现不能证明真实产品事件链路已经点火。

## 最低共享层根因

Editor04 的 `PlaySessionController` transition authority 尚未在合法状态迁移提交后发布统一的 `editor.mode` 消息。缺口不在 plugin manager、retained tick 或 UI 反射层。

## 架构修复验收

- 只在 `PlaySessionController` 已接受并提交的状态迁移后发布 `EditorMessagePayload::Mode(ModeMessage::PlayStateChanged { from, to })` 到 `TOPIC_MODE`。
- Edit -> Building、Building -> Playing、Building -> Edit、Playing -> Edit 和 backend crash/stop 的实际终态均有精确一次消息；no-op 与被拒绝的 transition 不发布。
- 发布位于 controller/host 的锁外通知路径，订阅者回调不得重入或延长状态机临界区。
- 增加 transition matrix + bus subscriber 回归；回跑 Editor12 lifecycle bridge 以验证 Entered/Exited callbacks 由真实状态机驱动。

## 禁止临时方案

- 不得让 Editor12 直接调用 plugin manager 来猜测 play state。
- 不得由 retained-host tick、菜单文案、按钮状态或 runtime poll 合成 `PlayStateChanged`。
- 不得恢复旧 play backend/bridge 兼容别名或建立第二份 play mode authority。

## 修复结果与回传

Open state: `未修复`。本记录只固定 Editor04 的生产者责任与验收；在 controller transition matrix、受管 Cargo、独立复审和 Editor12 真实 subscriber 回归完成前，不得生成 fixed return。

## 产出记录与时间

| 时间 | 状态 | 完成项目与证据 |
|---|---|---|
| 2026-07-29 CST | `OPEN / 待修复` | Editor12 M1 事件桥接审计确认 `ModeMessage::PlayStateChanged` 仅存在于模型和测试夹具；按 Editor04 计划第 156 行的 bus 契约回传本 failure。尚未修改 Editor04 生产代码或运行 Cargo。 |
| 2026-07-29 CST | `OPEN / source ready, managed validation queued` | `PlaySessionController` 现接收 `SharedEditorMessageBus`，在每次已提交的跨 `Edit/Building/Playing` 状态边界后、释放 `transition_gate` 后发布唯一 `TOPIC_MODE` typed payload；host 组合根传入 `EditorContext::bus()`，不建立 UI 私有生产者。新增矩阵覆盖 accepted Build/Stop、失败 Build、backend crash 与 noop/rejected 不重复发布。`rustfmt --check`、有 Git 基线的 host 文件 scoped `git diff --check` 及通知锁外静态审计通过；冻结快照 `1255` 已因本记录更新作废，受管 `zircon_editor` CPU lane 仍由其他 Session reservation 占用，尚未启动 Cargo。 |
