---
related_code:
  - zircon_editor/src/core/editor_event/workbench/activity_drawer_slot.rs
  - zircon_editor/src/ui/workbench/layout/activity_drawer_slot.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/event/core_event_conversion.rs
  - zircon_editor/src/ui/activity/slot.rs
related_tests:
  - tools/tests/test_editor06_activity_drawer_slot_hardcut.py
  - zircon_editor/src/tests/workbench/layout/window_drawer_ownership.rs
  - zircon_editor/src/tests/ui/control/activity_descriptors.rs
plan_sources:
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/SLevelEditor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/TabManager.h
status: in_progress
---

# Editor06 activity drawer bottom-slot hard cut

## 架构裁决

Workbench 运行时 drawer 只有五个实际位置：`LeftTop`、`LeftBottom`、`RightTop`、
`RightBottom`、`Bottom`。退役 `BottomLeft` / `BottomRight` enum、serde alias、
`canonical()` 和 layout normalize 合并算法；旧持久化输入直接失败，不保留兼容迁移。

Unreal LevelEditor 默认布局以显式 `FTabManager::NewSplitter()` / `NewStack()` 表达区域：
ContentBrowser、Sequencer、OutputLog 共用一个底部 stack；额外的 `BottomLeftPanel` 是布局树中的
独立 stack extension。Zircon 后续如需第二个底部区域，也必须新增显式 splitter/stack owner，不能把
旧输入别名重新解释成物理区域。`ActivityDrawerWindow` 模板内部的
`bottom_left_activity` / `bottom_right_activity` 是专用窗口的局部静态槽，不属于本运行时 enum，
因此保持不变。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据与后续 |
|---|---|---|---|
| 2026-08-29 14:34 +08:00 | `implementation-complete / static-contract-green / managed-rust-pending` | core event 与 UI layout 两套 `ActivityDrawerSlot` 同步硬切为五态；删除 serde alias、全消费方 canonical 分支、layout normalize 旧 slot 合并算法及其过期性能基准；默认 view fixture 中 Console、Runtime Diagnostics、Build Export 统一进入 `Bottom` tab stack；旧值的 Rust serde 测试改为拒绝。 | `test_editor06_activity_drawer_slot_hardcut.py` 4/4；与 Workbench 单一所有权契约合批 8/8、28.303s；tracked Rust `BottomLeft/BottomRight` enum 引用 0；fixture JSON 解析通过、Bottom host 3；定向 `rustfmt --check` 通过。共享托管 Cargo 池此前返回 `cargo_reuse_pool_busy`，本轮按目标要求不轮询、不旁路执行 Cargo；待后续受管 compile/focused tests 通过前保持 `in_progress`，不提交、不发送企微。 |
