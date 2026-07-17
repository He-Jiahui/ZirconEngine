---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: input-event-buffer-visibility
origin_plan: docs/plans/zircon_plugins/02-sound.md
fixing_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
origin_child_dir: docs/plans/zircon_plugins/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/12
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/input/runtime/event_buffer/mod.rs
  - zircon_runtime/src/input/runtime/event_buffer/frame.rs
  - zircon_runtime/src/input/runtime/event_buffer/recorder.rs
  - zircon_runtime/src/input/runtime/input_state.rs
tests:
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --locked tests::kira_graph_sync::send_bus_inherits_target_and_parent_gain_without_bypassing_the_bus_chain -- --exact --nocapture --test-threads=1
resolved_at: 2026-07-17
---


# Runtime12：input event buffer 可见性收敛

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/02-sound.md`
- 来源执行者：`plugins02-sound-m1-kira-core-20260717`
- 来源执行切片：Sound M1 Kira graph send-target bus contract 的受管红测
- 修复责任计划：`docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`
- 交接原因：两个 Runtime12 input 私有模块的可见性边界阻止所有依赖 `zircon_runtime` 的 current-source Cargo 门禁，最低共享修复不属于 Sound。

## 失败现象与复现证据

受管作业 `ca519ce03c9b45b4a0a3b23ad0dbd06a`（run `5aeb1ce2edc14ebcbea5e3fe88b00dd9`）在 Windows 兼容池 `F:\cargo-targets\zircon-engine\pool\4ce48bbc2127c419d3548c675843f2bb9395a48be4a3a078a4899cd21552bd79` 中于 2026-07-17 退出 `101`，尚未编译到 Sound 测试。

`zircon_runtime/src/input/runtime/event_buffer/mod.rs` 使用 `pub(super) use` 重导出私有 `frame::FrameEventBuffer` 与 `recorder::InputEventRecorder`，而同级 `input_state.rs` 需要从 `event_buffer` 导入它们，产生 `E0365` 与 `E0603`。原始 stderr 已由协调器作业日志保存。

## 最低共享层根因

Runtime12 在将 event buffer 切为子模块后，没有把 `event_buffer` 对同级 runtime 模块的内部接口暴露到正确的 crate 可见性；这不是 Sound 的 Kira 图、Cargo lock 或测试筛选问题。

## 架构修复验收

- Runtime12 明确 `FrameEventBuffer` 与 `InputEventRecorder` 的内部所有者和被允许的同级消费者，并以最小 crate 可见性直接导出，不增加旧路径 alias、compat facade 或调用端复制。
- Runtime12 的 input focused gate 通过，并证明 `input_state` 继续经 canonical `event_buffer` 边界访问两项类型。
- 上述 Sound 受管精确红测可编译到 `tests::kira_graph_sync::send_bus_inherits_target_and_parent_gain_without_bypassing_the_bus_chain`；其自身 red/green 结果须单独记录，不能被 Runtime12 编译修复冒充。

## 禁止临时方案

- 不得把类型重新定义、复制到 `input_state`，或为旧导入路径添加 alias、shim、facade。
- 不得放宽 Sound 的 `--locked`、跳过 Runtime 编译，或把该编译失败记为 Sound graph 已验证。

## 修复结果与回传

- 根因：FrameEventBuffer and InputEventRecorder were private child types re-exported to their runtime parent, which violated Rust visibility and blocked InputState sibling imports with E0365/E0603.
- 架构修复：Set both child structs to pub(in crate::input::runtime), kept event_buffer exports pub(super), and retained the types as runtime-internal implementation owners.
- 验证：Managed Runtime job d064840b0a8f40dcb405bab74b493ba1 run 78454dffc1744c858bad697721992c7e: 39 input tests passed, 0 failed, 8202 filtered; E0365/E0603 absent.
- 回传：Runtime12 repaired the lowest input event-buffer visibility boundary without widening it outside crate::input::runtime; Sound may rerun after the Runtime12 immutable milestone SHA.
