---
related_code:
  - zircon_editor/src/ui/workbench/state/editor_world_slot.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_construction.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_project.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/tests/editing/history.rs
  - zircon_editor/src/tests/editing/import.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
implementation_files:
  - zircon_editor/src/ui/workbench/state/editor_world_slot.rs
tests:
  - rustfmt --edition 2021 --check zircon_editor/src/ui/workbench/state/editor_world_slot.rs
  - editor UI 10 editor_world_slot dead_code suppression scan
doc_type: module-doc
status: active
---

# Workbench Editor World Slot

## 职责

`EditorWorldSlot` 是 workbench editor state 持有 runtime `LevelSystem` 的边界对象。它允许 welcome 模式保持未加载状态，也允许 project 模式持有已加载 world，并把“无项目打开”转化为显式 `Option` 结果。

## 生产契约

- `loaded(world)` 和 `unloaded()` 用于 `EditorState` 启动构造。
- `is_loaded()` 用于状态判断。
- `try_snapshot()`、`try_with_world(...)`、`try_with_world_mut(...)` 是生产路径访问 world 的标准入口；调用方必须处理 `None`，避免 welcome 模式下 panic。
- `replace(world)` 和 `clear()` 用于项目打开/关闭流程切换 world。

## 测试契约

`snapshot()`、`with_world(...)`、`with_world_mut(...)` 是测试侧便捷入口，会在 world 未加载时 panic。它们只在 `cfg(test)` 下编译，用于编辑历史、导入、命令调度、host binding 等测试直接断言 scene 内容。

## 2026-06-22 结构治理记录

按 `engine-code-structure-convention.md` E6 与 Editor UI 10 M5.T2，`editor_world_slot.rs` 已清除 4 处陈旧生产 `#[allow(dead_code)]` 抑制：

- `loaded(world)` 保持生产方法，因为 `EditorState::new(...)` 与 `EditorState::project(...)` 会直接调用。
- `snapshot()`、`with_world(...)`、`with_world_mut(...)` 收口为 `cfg(test)`，生产路径继续使用 `try_*` API。

窄范围验证确认 `registry.rs` 与 `editor_world_slot.rs` 两个 S8 证据路径均无剩余 `dead_code` 抑制；生产 `ui/**` 中没有继续调用测试专用 world 访问方法。
