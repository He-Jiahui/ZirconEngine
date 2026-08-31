---
related_code:
  - zircon_runtime/src/ui/event_ui/manager/registration.rs
  - zircon_editor/src/ui/control/service.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/workbench/reflection/route_registration
related_tests:
  - tools/tests/test_editor06_binding_route_authority_contract.py
plan_sources:
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
status: in_progress
---

# Editor06 binding-only route authority hard cut

## 范围与架构裁决

Editor workbench 的菜单、资产、Inspector、Docking、Animation、Draft 与 Viewport reflection
route 不是未实现的 runtime handler。它们只在 runtime event manager 保存 typed binding 与稳定
`UiRouteId`，执行由 `EditorHostEventController` 解析 binding 后进入唯一 editor dispatch/operation
authority。旧 `register_route_stub` / `register_stub_route` 命名把正式跨模块合同误写成占位实现，现按
硬切规则统一为 `register_binding_route`，不保留 alias、shim 或兼容入口。

Runtime manager 对 binding-only route 仍不安装 handler；没有 host dispatcher 的调用方必须使用带
handler 的 `register_route`。这保证 runtime 直接 invocation 会显式失败，而 editor control request
不会绕过 typed binding normalization 与 event/operation authority。

## 测试阶段

- 源码阶段：旧符号全仓 Rust 清零；结构契约锁定 runtime/editor API、workbench 单 owner 与
  EditorHost route-to-binding dispatch 路径。
- 里程碑验收：等待受管 Windows Rust compile、route invocation 行为回归与上层 retained-host
  回归；未取得这些证据前不标记 completed，不提交里程碑 commit，不发送企微。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据与后续 |
|---|---|---|---|
| 2026-08-28 01:35 +08:00 | `implementation-complete / static-contract-green / managed-rust-pending` | 完成 runtime `UiEventManager`、editor `EditorUiControlService` 与 workbench reflection helper 的 binding-only route authority 硬切；菜单、资产、Inspector、Docking、Animation、Draft、Viewport 与 template projection 消费方同批迁移，旧 helper 文件删除，无兼容层。 | `test_editor06_binding_route_authority_contract.py` 3/3；精确 Rust `rustfmt --check` 与 scoped `git diff --check` 通过；`register_route_stub`、`register_stub_route`、`mod stub_route` 全仓 Rust 命中 0。按当前目标继续非验收任务，本轮未启动 Cargo；受管 Windows compile/behavior/retained-host 回归仍待测试阶段，故状态保持 `in_progress`。 |
