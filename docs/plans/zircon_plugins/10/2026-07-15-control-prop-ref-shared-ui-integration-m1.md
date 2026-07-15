---
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/tests/ui_binding_control_prop_ref.rs
  - zircon_runtime/src/ui/template/asset/binding/validation.rs
  - zircon_runtime/src/ui/tests/asset_binding.rs
  - zircon_runtime/tests/ui_binding_control_prop_ref.rs
implementation_files:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/tests/ui_binding_control_prop_ref.rs
  - zircon_runtime/src/ui/template/asset/binding/validation.rs
  - zircon_runtime/src/ui/tests/asset_binding.rs
  - zircon_runtime/tests/ui_binding_control_prop_ref.rs
plan_sources:
  - docs/plans/zircon_plugins/10-editor-integration.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_runtime_interface --test ui_binding_control_prop_ref --locked
  - cargo test -p zircon_runtime --test ui_binding_control_prop_ref --no-default-features --features ui --locked
  - cargo check -p zircon_runtime --no-default-features --features ui --lib --locked
doc_type: milestone-detail
---

# Plugins10 M1 shared ControlPropRef integration gate

Plan: docs/plans/zircon_plugins/10-editor-integration.md
Milestone: M1
Status: completed
Files: ["zircon_runtime_interface/src/ui/template/asset/binding/expression.rs", "zircon_runtime_interface/tests/ui_binding_control_prop_ref.rs", "zircon_runtime/src/ui/template/asset/binding/validation.rs", "zircon_runtime/src/ui/tests/asset_binding.rs", "zircon_runtime/tests/ui_binding_control_prop_ref.rs", "docs/zircon_runtime/ui/v2.md", "docs/plans/zircon_plugins/10/2026-07-15-control-prop-ref-shared-ui-integration-m1.md"]
Date: 2026-07-15

## Scope Delivered

- Plugins05 实现 cross-control binding parser 与 descriptor-authoritative Runtime validator。
- Plugins10 作为无功能前置依赖的共享 UI 集成门禁，记录精确 manifest、受管验证、独立复审与
  Render18 fixed return；不宣称 Navigation Editor M6 或其他功能里程碑完成。

编号计划定义改动由计划维护会话单独接管，不纳入本业务提交清单。
本记录的 `Files` 字段是本次七文件业务提交的唯一受管 manifest。

## Fresh Testing Evidence

- managed parser `de5c2a6887e644e3b90ad8c2292d35f5`: 3 passed / 0 failed。
- managed Runtime behavior `169c6d6c2a7b449689fbc92a5a2e0faa`: 6 passed / 0 failed。
- managed Runtime UI upward compile `6fb3d30fd54544bfb821adbe2027bb8a`: exit 0。
- independent re-review: Critical 0 / Important 0。
- handoff validator: 148 artifacts / 0 errors。

## Review

独立复审已接受：Critical 0 / Important 0。

## 状态和完成项目

| 项目 | 状态 | 证据 |
|---|---|---|
| cross-control parser | completed | 3/3 |
| descriptor/tree-scope behavior | completed | 6/6 |
| shared-source upward compile | completed | exit 0, E0004 absent |
| fixed return | completed | Render18 fixed artifact |
| independent review | completed | Critical 0 / Important 0 |

## Remaining Scope

- Navigation selected-surface operation projection 与 Editor05 viewport provider 仍按原 owner/open
  failure 推进，不属于本共享集成提交。
