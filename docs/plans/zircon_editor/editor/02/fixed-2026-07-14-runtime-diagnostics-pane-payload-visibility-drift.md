---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: runtime-diagnostics-pane-payload-visibility-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_editor/editor_ui/09
related_code:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/mod.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
tests:
  - cargo test -p zircon_editor --lib --locked
resolved_at: 2026-07-14
---


# EditorUI09：runtime diagnostics pane payload 私有 owner 导入漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M1 最终 editor consumer 门禁
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md`
- 交接原因：失败测试属于 EditorUI09 Runtime Diagnostics 真实数据投影与 workbench pane payload owner 边界；Editor02 world-sync 不拥有 UI layout module 可见性。

## 失败现象与复现证据

受管 Windows job `3ca02648cc76456ca54b868203e88827` 执行 `cargo test -p zircon_editor --lib --locked` 时出现：

```text
error[E0603]: module `pane_payload` is private
zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs:471:61
```

测试从 `workbench_host_window::pane_payload::RuntimeDiagnosticsPanePayload` 下钻私有 child module；同一 parent owner 已在 `workbench_host_window/mod.rs` 以 `pub(crate) use` 暴露该 DTO。完整日志：`E:\ZirconBuilds\editor02-m1-editor-consumer-after-render18-20260714.log`。

## 最低共享层根因

Runtime Diagnostics 投影测试仍引用拆分前的 child owner 路径，没有随 workbench layout 的 folder-backed owner 收口迁移到 canonical parent surface。这是测试消费路径漂移，不应通过公开整个 `pane_payload` child module 修复。

## 架构修复验收

- runtime diagnostics 测试只从 `workbench_host_window` canonical parent surface 导入 `RuntimeDiagnosticsPanePayload`。
- `pane_payload` child module 继续保持私有；不新增兼容 re-export module 或旧路径 alias。
- 原 `cargo test -p zircon_editor --lib --locked` 不再出现该 E0603。

## 禁止临时方案

- 不把 `mod pane_payload` 改成 public/pub(crate) 来迁就旧下钻路径。
- 不复制 DTO、不在测试中定义影子 payload。
- 不修改 Editor02 world-sync 或 Shader04 文件。

## 修复结果与回传

- 根因：The Runtime Diagnostics projection test still imported RuntimeDiagnosticsPanePayload through the private pane_payload child module after the workbench layout owner had converged on its canonical parent surface.
- 架构修复：Changed the test consumer to import RuntimeDiagnosticsPanePayload from workbench_host_window, preserving pane_payload as a private child and adding no compatibility module or duplicate DTO.
- 验证：Managed Windows job 9a0b6f1cdff144548d082fdd9b5ea636 ran cargo test -p zircon_editor --lib --locked, compiled zircon_runtime and zircon_editor lib-test successfully with no E0603 and reached test execution; the Runtime Diagnostics pane presentation tests also passed during the running suite.
- 回传：EditorUI09 now consumes the pane payload only through the canonical parent owner. The private child remains private and the original E0603 is closed.
