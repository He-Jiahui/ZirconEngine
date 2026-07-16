---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: control-prop-ref-validation-runtime-gate
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_plugins/05-navigation.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_plugins/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime/src/ui/template/asset/binding/validation.rs
tests:
  - cargo test -p zircon_runtime --lib ui_v2_compiler_ --locked
  - cargo test -p zircon_runtime --no-default-features --features graphics render_volumetric --locked
resolved_at: 2026-07-15
---


# Plugins05：ControlPropRef 阻断 Render18 Runtime 上行门禁

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行切片：`render18-af-m2-rebase-20260715`（AF-M3 Runtime upward gate）。
- 来源作业：受管 Runtime job `f9e4addefebd4b9f9ef6915d9e51cff8`。
- 修复责任计划：`docs/plans/zircon_plugins/05-navigation.md`
- 责任 Session：`plugins05-control-prop-binding-ref-20260715`。
- 交接原因：binding expression enum 新增分支后的 exhaustive validation 属于 Plugins05/UI binding 合同边界；Render18 不拥有表达式解析或验证实现。

## 失败现象与复现证据

受管 Runtime job `f9e4addefebd4b9f9ef6915d9e51cff8` 在执行 Render18 AF-M3 测试前以
exit 101 停止。`zircon_runtime_interface/src/ui/template/asset/binding/expression.rs`
已新增 `UiBindingExpression::ControlPropRef { control_id, property }`，但
`zircon_runtime/src/ui/template/asset/binding/validation.rs` 的
`infer_expression_kind` 没有对应 match arm，触发外部 `E0004`。Render18 没有编辑任一
表达式或 validation 路径，暂存总数保持 0。

## 最低共享层根因

expression enum 与 runtime validation 的版本演进没有在同一语义边界完成。ControlPropRef
不是普通 `PropRef`：validation 必须先解析 control identity，再取得该 control 的 component
descriptor/property schema，准确返回 `UiValueKind`；未知 control 或 property 必须产生现有的
明确 diagnostics。用 wildcard、`UiValueKind::Any` 或把它退化为当前 node 的 prop 都会掩盖
cross-control binding 合同错误。

## 架构修复验收

- Plugins05 owner 为 `ControlPropRef` 添加完整 validation 分支：解析 control/property、使用目标 control 的 descriptor 取得真实 kind，并为 unknown control/property 产生稳定的 unresolved diagnostic。
- 添加 focused exhaustive behavior tests，至少覆盖：有效跨 control property 的 kind、unknown control、unknown property、以及与目标 kind 不匹配的 binding；测试不得只验证“编译通过”。
- 先以受管 lane 运行 focused binding/compiler tests，再以新的受管 Runtime graphics-only upward gate `cargo test -p zircon_runtime --no-default-features --features graphics render_volumetric --locked`。只有该上行 gate 真正开始并 exit 0 后，才可回传 Render18 AF-M3。

## 禁止临时方案

- 不得由 Render18 修改 `expression.rs` 或 `validation.rs`，不得用 wildcard、`Any`、当前节点 prop 或 feature fallback 吞掉新 enum variant。
- 不得在上行 Runtime gate 未重跑成功前标记 Render18 AF-M3 accepted/frozen 或 closeout。
- 不得手动操作其他 Session 的 stage、lease 或 Cargo 作业。

## 修复结果与回传

- 根因：UiBindingExpression gained ControlPropRef while Runtime binding validation still matched only same-node expressions.
- 架构修复：Runtime now resolves cross-control properties through a component-tree-scoped descriptor-kind index, reports stable unresolved diagnostics, and validates Navigation payload kinds without authored-prop fallback.
- 验证：Managed focused job 169c6d6c2a7b449689fbc92a5a2e0faa passed 6/6; managed Runtime UI upward compile job 6fb3d30fd54544bfb821adbe2027bb8a exited 0; independent review Critical 0 Important 0.
- 回传：ControlPropRef exhaustive Runtime validation is fixed and ready for Render18/Shader06 rerun.
- Parser 补充证据：managed job `de5c2a6887e644e3b90ad8c2292d35f5` passed 3/3。
