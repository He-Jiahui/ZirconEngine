---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: plugin-editor-runtime-mirror-consumer-wiring
origin_plan: docs/plans/zircon_plugins/05-navigation.md
fixing_plan: docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
origin_child_dir: docs/plans/zircon_plugins/05
fixing_child_dir: docs/plans/zircon_plugins/12
related_code:
  - zircon_plugins/plugin_sdk/src/editor.rs
  - zircon_editor/src/core/runtime_event_consumer/host.rs
  - zircon_plugins/navigation/runtime/src/plugin.rs
  - zircon_plugins/navigation/editor/src/runtime_mirror.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_editor -TargetDir E:/cargo-targets/zircon-navigation-m6-editor -SkipBuild
resolved_at: 2026-07-15
---


# Plugins 12：Editor/Runtime mirror consumer SDK 接线缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/05-navigation.md`
- 来源执行切片：M6-T3 Agent/avoidance PIE read-only mirror
- 修复责任计划：`docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md`
- 交接原因：最低共享原因是 SDK declaration 尚不能注册 runtime typed-event consumer；每个插件自行监听会复制 session/sequence/lifecycle 逻辑。

## 失败现象与复现证据

Runtime 已注册并生产共享、serde-safe `NavAgentTickReport`，Navigation editor mirror 可消费 report 并拒绝错 session/乱序，但没有 SDK/host subscription 把 runtime event、PIE session id 与 sequence 送入 consumer。当前 `apply_tick_report` 仅被测试直接调用。

## 最低共享层根因

Review D9 要求 editor/runtime mirror consumer 走 SDK declaration；`authoring_plugin!` 与当时的 editor runtime consumer host 没有 typed event subscription/consumer 注册面和 lifecycle 解绑语义。

## 架构修复验收

- Plugins 12 SDK declaration 支持按 plugin event id/payload schema 注册 typed mirror consumer，并在 package/descriptor parity 中单源投影。
- Host/runtime client 只在 PIE session 与 consumer capability 启用时订阅，生成单调 sequence，session 结束时解绑并清 mirror。
- Navigation 使用该 SDK consumer，实际 runtime tick event 到达 mirror；错误 schema、跨 session、乱序与 disable 后事件均有测试。
- Debug capture resource 随订阅启停，禁用时 runtime 不分配 agent debug payload。

## 禁止临时方案

- 禁止 editor 直接持有 Runtime World、每插件私有轮询线程、手工测试调用冒充订阅、无条件每帧 debug capture 或复制一份事件目录。
- 禁止 aliases、compatibility shims、silent fallback、duplicated truth、test-only bypasses 或 call-site exceptions。
- 禁止削弱测试或 M6 验收标准以隐藏失败。

## 修复结果与回传

- 根因：Runtime mirror contracts existed without a product editor consumer lifecycle; partial subscribe and unsubscribe failures could lose ownership and prevent retry.
- 架构修复：Added event-id reference counting, linked-World forwarding, transactional editor host subscriptions, navigation runtime/editor consumers, and retry-safe Enter/Exit Play ordering with typed cleanup errors.
- 验证：Independent review accepted with 0 Critical and 0 Important; focused Python owner guards passed; dynamic, schedule, and performance structure audits report risks=[]; rollback/retry regression tests cover partial subscribe and unsubscribe failure.
- 回传：Plugins12 now owns the complete editor/runtime mirror consumer path and returns the fixed handoff to Plugins05.
