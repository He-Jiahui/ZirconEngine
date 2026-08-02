---
handoff_kind: failure
status: open
created_at: 2026-07-30
summary_slug: fallible-exclusive-transition-context-update
origin_plan: docs/plans/zircon_editor/editor/04-pie-and-simulation.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor/04
fixing_child_dir: docs/plans/zircon_editor/editor/03
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/editing/context.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_project.rs
tests:
  - cargo test -p zircon_editor --lib clear_history_and_context --locked
  - cargo test -p zircon_editor --lib --locked
---

# Editor03: ExclusiveTransition 无法安全执行可失败 context 更新

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
- 来源执行切片：M2.4 authoring facade hard cut
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：`ExclusiveTransition::clear_history_and_context` 是 transaction/history 的唯一生命周期 owner；Editor04 只能提交可失败的 project world 切换，不能在 UI 侧复制或绕过 undo/context 原子性。

## 失败现象与复现证据

`zircon_editor/src/core/editing/engine/transaction.rs` 的
`ExclusiveTransition::clear_history_and_context` 当前要求
`update: impl FnOnce(&mut T)`，而 `CoreEditContext::clear_scene` 返回
`Result<(), EditCommandError>`。M2.4 的 `EditorState::{replace_world,clear_project}`
将该函数直接传入，因返回类型不匹配会在 Rust 类型检查时报错。

独立静态复审还确认不能只把调用改成丢弃结果，或在 helper 尾部简单追加 `?`：当前 helper 在调用 update 前已经移除并 clear
`HistoryContextId::Global` 的 history。若 `clear_scene` 的 generation 检查或随后 authoring gateway replacement 失败，旧 project world 仍存在，而 undo history 与 selection context 已被清空。

本记录是静态确定的 source failure；尚未运行 Cargo，因此不主张任何运行时通过结论。

## 最低共享层根因

Editor03 的 exclusive transition 契约把 context update 建模为不可失败副作用，并先提交 history destruction；但 project lifecycle 需要一个会失败的 context/world boundary。transaction engine 没有定义该失败的提交顺序、context restore 与 history rollback 语义，导致上层只能出现类型错误或破坏性地吞掉错误。

## 架构修复验收

- `ExclusiveTransition` 提供明确的 fallible context update 契约，`EditCommandError` 必须原样向上传播。
- update 失败时，context、selection、global undo/redo history 和 active project authoring facade 保持调用前一致；不得留下半切换状态。
- history destruction 与可失败 context/world update 的提交顺序必须由 Editor03 单一 owner 定义并用回归测试固定；成功路径仍只清理一次旧 history 并保持 finalize 顺序。
- 增加 focused transaction tests，覆盖 context update error、selection generation exhaustion 或等价可重现 error、成功 replace/clear、history preservation/clear 和 context restore。
- 重新运行本 Editor04 的 authoring facade attach/detach 以及 project replace/clear gate，之后再进行 managed `cargo test -p zircon_editor --lib --locked`、独立复审与 failure return。

## 禁止临时方案

- 不得把 `Result` 用 `let _ =`、`unwrap`、`expect` 或 call-site exception 吞掉。
- 不得在 Editor04/UI 侧手工 clear history、复制 selection 或补建 undo stack 绕过 transaction owner。
- 不得先 destroy history 再把 failure 当作可接受的 project transition 结果。
- 不得用 compatibility shim、第二份 authoring world 或 test-only bypass 掩盖原子性缺失。

## 修复结果与回传

Open state: `待修复`。Editor04 已完成不依赖该 API 的 typed authoring/play gateway 路由静态切换，但 project replace/clear 的 current-source Cargo、独立 review 和本 failure 的 fixed return 均不得在 Editor03 修复后才推进。

## 产出记录与时间

| 时间 | 状态 | 完成项目与证据 |
| --- | --- | --- |
| 2026-07-30 CST | `OPEN / cross-plan handoff` | M2.4 static review 定位 `clear_history_and_context` 的 infallible callback 与 `CoreEditContext::clear_scene -> Result` 不兼容；进一步确认 history 在 fallible update 前销毁，存在旧世界与已清空 undo/selection 的半切换风险。责任移交 Editor03 transaction owner；未运行 Cargo，未声明通过。 |
