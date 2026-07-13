---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
resolved_at: 2026-07-13
summary_slug: runtime-interface-ui-resource-ref-hard-cutover
origin_plan: docs/plans/zircon_plugins/06-ai.md
fixing_plan: docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
origin_child_dir: docs/plans/zircon_plugins/06
fixing_child_dir: docs/plans/zircon_editor/editor_ui/05
related_code:
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/dependency.rs
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/value.rs
tests:
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_runtime --locked --offline --jobs 1 --target-dir E:/cargo-targets/zircon-ai-m1
---

# Editor UI 05：UiResourceRef hard-cutover 遗留导入已修复回传

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/06-ai.md`
- 来源执行切片：M1 owner revoke execution lease/barrier 最终验证
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md`
- 交接原因：最低共享原因位于 `zircon_runtime_interface` UI asset DTO hard-cutover；AI 不拥有该模块，不能在插件切片内保留或修补旧模块路径。

## 失败现象与复现证据

`dependency.rs` 曾导入已删除的 `super::resource_ref::UiResourceRef`，导致 AI 验证在进入自身 crate 前返回 `E0432`。

## 最低共享层根因

UI resource-ref folder-backed hard cutover 只迁移了 owner module declaration/file，未同步同域 sibling import。

## 架构修复验收

- owner 将导入硬切到 `super::value::UiResourceRef`，未恢复旧模块或 compatibility shim。
- AI 聚焦 owner/revoke 测试 3/3、完整 44/44 与标准验证矩阵均通过。

## 禁止临时方案

- 禁止恢复 `resource_ref.rs` compatibility module、`pub use` shim、重复定义或仅对 AI 绕过 UI module 编译。

## 修复结果与回传

- 根因：UI resource-ref folder-backed hard cutover 删除旧 owner 后，`dependency.rs` 的 sibling import 未同步迁移，导致共享 `zircon_runtime_interface` 在 AI crate 编译前失败。
- 架构修复：Editor UI 05 owner 将唯一导入硬切到 `super::value::UiResourceRef`；未恢复旧模块、`pub use` facade、compatibility shim 或重复定义。
- 验证：AI owner/revoke 聚焦测试 3/3、完整 AI runtime 44/44 tests 与标准验证矩阵均通过；M2 后续完整回归为 58/58 tests。
- 回传：`docs/plans/zircon_plugins/06-ai.md` 的 M1 owner revoke barrier 验证已恢复并完成，AI M2 可继续在同一 typed runtime-interface 边界上执行。
