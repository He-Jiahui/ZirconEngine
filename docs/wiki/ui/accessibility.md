---
related_code:
  - zircon_runtime/src/ui/accessibility/mod.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/accessibility/accesskit.rs
implementation_files:
  - zircon_runtime/src/ui/accessibility/name.rs
  - zircon_runtime/src/ui/accessibility/semantic_text.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine UI Wiki
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_accessibility.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_accessibility_widget_actions.rs
  - zircon_runtime/src/ui/accessibility/accesskit/performance_tests.rs
doc_type: module-detail
---

# 无障碍

## 语义快照

`accessibility_snapshot` 从 arranged tree 提取可见节点的角色、名称、值、状态、范围、文本和几何，生成平台无关语义树；`accessibility_snapshot_bounded` 使用 `AccessibilityBuildBudget` 限制节点/文本工作量，避免超大虚拟列表阻塞帧。

名称解析优先级为显式 label、关联文本、组件描述和可见文本；secure text 只导出掩码语义。折叠、禁用、不可命中节点按可访问性可见性规则过滤。

## 动作分发

`dispatch_accessibility_action` 接受 focus、activate、value、expanded、scroll、text 等动作，转换为对应组件事件或表面输入效果。动作结果包含 Applied/Rejected 原因；焦点动作遵守 modal scope。

启用 `accessibility-accesskit` feature 时，`accesskit` 适配器将快照和增量更新发送给操作系统辅助技术；未启用时仍可生成诊断快照供测试使用。

```rust
let snapshot = surface.accessibility_snapshot();
// 运行时内部的 bounded 提取和动作分发由 UiRuntimeDriver 调用。
```

## 限制与验证

快照必须在 arranged tree 完成后生成，否则几何和可见性可能过期。预算耗尽返回 `AccessibilitySnapshotBudgetError`；测试覆盖索引焦点、组件动作和性能上限。
