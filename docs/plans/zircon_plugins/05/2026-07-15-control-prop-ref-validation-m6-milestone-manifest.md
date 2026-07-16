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
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
tests:
  - cargo test -p zircon_runtime_interface --test ui_binding_control_prop_ref --locked
  - cargo test -p zircon_runtime --test ui_binding_control_prop_ref --no-default-features --features ui --locked
  - cargo check -p zircon_runtime --no-default-features --features ui --lib --locked
doc_type: milestone-detail
---

# Plugins05 M7 ControlPropRef validation closeout

Plan: docs/plans/zircon_plugins/05-navigation.md
Milestone: M7
Status: completed
Files: ["zircon_runtime_interface/src/ui/template/asset/binding/expression.rs", "zircon_runtime_interface/tests/ui_binding_control_prop_ref.rs", "zircon_runtime/src/ui/template/asset/binding/validation.rs", "zircon_runtime/src/ui/tests/asset_binding.rs", "zircon_runtime/tests/ui_binding_control_prop_ref.rs", "docs/zircon_runtime/ui/v2.md", "docs/plans/zircon_plugins/05/2026-07-15-control-prop-ref-validation-m6-milestone-manifest.md", "docs/plans/zircon_runtime/render/18/fixed-2026-07-15-control-prop-ref-validation-runtime-gate.md"]
Date: 2026-07-15

## Scope Delivered

This record is the exact coordinator commit manifest for the ControlPropRef M7 support slice.

- 新增结构化 `control.<control_id>.prop.<property>` 表达式合同。
- Runtime 为 document root 与每个 component tree 分别建立 descriptor-authoritative control
  property kind 索引；unknown control/property/descriptor 使用稳定 `UnresolvedRef`。
- Navigation action payload 明确校验 `surface_entity: Int` 与
  `force_full_rebuild: Bool`，并避免把表达式源码字符串误判为运行时 payload kind。
- Render18 failure 已按生命周期返回 fixed；本记录只关闭 M6 的 ControlPropRef 校验切片，不代表
  Navigation M6 全部完成。

## Fresh Testing Evidence

- 受管 parser job `de5c2a6887e644e3b90ad8c2292d35f5`：3 passed / 0 failed。
- 受管 focused Runtime job `169c6d6c2a7b449689fbc92a5a2e0faa`：6 passed / 0 failed。
- 受管 Runtime upward compile job `6fb3d30fd54544bfb821adbe2027bb8a`：exit 0，原
  `E0004` absent。
- 受管 Render18 graphics-only job `9c12eca1b7ce4ae39901667fe3434016` 已越过
  `ControlPropRef`；后续外部 Text/Shader03 测试漂移不属于本 manifest。
- failure handoff validator：148 artifacts / 0 errors。
- output-record audit 的 4 项 violation 均位于无关 Editor UI 01/10/11/index 路径。

## Review

- 首轮独立只读复核：Critical 0 / Important 3。
- 修订 payload kind、descriptor authority 与 component scope coverage 后，最终独立只读复核：
  Critical 0 / Important 0。

## 状态和完成项目

| 项目 | 状态 | 证据 |
|---|---|---|
| ControlPropRef parser contract | completed | managed parser 3/3 |
| tree-scoped descriptor validation | completed | managed Runtime behavior 6/6 |
| exhaustive Runtime compile | completed | managed upward compile exit 0 |
| Render18 failure lifecycle return | completed | fixed handoff + 148/148 validator |
| independent review | completed | final Critical 0 / Important 0 |

## Remaining Scope

- `failure-2026-07-15-navigation-bake-selection-operation-arguments.md` 仍为 Plugins05 的 open
  failure，继续由选择态/operation 参数投影切片处理。
- Editor05 viewport provider host failure 仍 open；不得由本校验切片旁路实现。
It deliberately does not claim the dependency-bearing Navigation Editor M6 feature milestone.
