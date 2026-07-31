---
status: in_progress
created_at: 2026-07-29
origin_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
milestone: M1
slice: 1.2 lifecycle event wiring
---

# Editor12 M1 plugin lifecycle message bridge

## 产出记录与时间

| 时间 | 状态 | 完成项目与证据 |
|---|---|---|
| 2026-07-29 CST | `进行中 / 静态实现完成` | `EditorPluginLifecycleMessageBridge` 已订阅 `editor.mode` 与 `editor.document`，由 retained-host tick 在总线锁外泵送到 `EditorPluginManager::dispatch_lifecycle_event_to_active`。Rust 单测覆盖 Playing 边界、文档 Opened/Saved/Closed 以及 dirty/focus 排除；`rustfmt --edition 2024 --config skip_children=true --check`、scoped `git diff --check` 通过。受管 Cargo 等待 Coordinator01 reload 后的 fresh source-bound copy。 |
| 2026-07-29 CST | `进行中 / 审查修复待复核` | 初次独立审查的两项 Important 与一项 Minor 已修复：manager mutation 时以 FIFO 保留 delivery 并在下一 tick 重试；按 `delivery.topic()` 过滤 broadcast；`callback_failures` 逐条统计 manager diagnostic。新增竞争重试、错误 topic broadcast、双失败插件计数回归测试；Cargo 与独立复审尚未完成，不能作为 accepted evidence。 |
| 2026-07-29 CST | `进行中 / 静态复审通过` | 独立复审 `Critical/Important/Minor = 0/0/0`：确认 FIFO 在 `MutationInProgress` 前端回插、topic 与 payload 双重过滤、回调与 diagnostic 分别精确计数，且 host 在 shell 锁外泵送。仍严格等待 fresh source-bound Rust gate，未将静态结果写为 accepted。 |
| 2026-07-29 CST | `OPEN / 跨计划接线债已路由` | 真实生产端尚未发布 `ModeMessage::PlayStateChanged` 与结构性 `DocumentMessage`；已分别写入 [Editor04 play-mode producer failure](../04/failure-2026-07-29-play-mode-message-producer-missing.md) 与 [Editor01 document producer failure](../01/failure-2026-07-29-document-message-producer-missing.md)。桥接不以 tick、UI 状态或 dirty/focus 伪造事件。 |
| 2026-07-30 CST | `OPEN / Performance01 current-source static review` | 生产端现已可达，但性能验收未成立：bridge锁内整箱drain到第二pending并持锁无预算回调；manager再持`lifecycle_mutation`跨全部callback、深clone完整catalog/history并重建active extensions。PERF-MVP-594负责bounded page、两层lock-out callback、generation commit和bounded audit；PERF-MVP-538负责structural snapshot/extension复用。35/35 core plugin Rust文件、6,318行、51 tests静态读完且rustfmt通过；无managed Cargo/WPR，不得沿用旧“静态复审通过”作为性能accepted。 |

## 接线边界

- `ModeMessage::PlayStateChanged`：仅非 Playing -> Playing 发 `EnteredPlayMode`，Playing -> 非 Playing 发 `ExitedPlayMode`；Edit <-> Building 和同态消息不产生生命周期回调。
- `DocumentMessage::{Opened, Closed, Saved}`：发带 `DocumentId` subject 的 `SceneChanged`；dirty/focus 为投影细节，不重放为场景变更。
- `Loaded`、`Enabled`、`Disabled` 仍由 manager 状态机独占，桥接层不得绕过它们。
- 若 manager 正在 mutation，当前及后续 drained delivery 保留在桥接器 FIFO，下一 host tick 重试；不得因 `MutationInProgress` 丢弃 lossless play/document 事实。
- 性能硬边界：bridge每tick按entries+bytes+deadline拉取，callback不得发生在bridge pending mutex或manager mutation gate内。manager锁内只快照ordered handles+generation，锁外dispatch，结果按generation commit；成功且active state不变的外部event不得换structural catalog/extension generation。
- lifecycle current-call report与retained audit分离；routine成功event不得永久复制进catalog。latest stage state与diagnostic/audit必须有entry+bytes+age硬界，同时保留失败front retry、Faulted与reload/unload语义。

## 未完成依赖

- `AssetChanged` 继续等待 Editor09 资产索引事件的稳定 producer；当前不得以 retained-host 轮询或文档消息伪造该事件。
- `UiMessage` 属于 M2 cdylib 自定义消息物化通道，保持其唯一合法 producer，不在 M1 总线适配器中增加兼容入口。
