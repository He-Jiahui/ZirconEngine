---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
resolved_at: 2026-07-14
summary_slug: manager-service-reactivation-lifecycle
origin_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/runtime/15
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
related_code:
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/batch.rs
  - zircon_runtime/src/core/runtime/handle/activation/module_lifecycle.rs
  - zircon_runtime/src/core/runtime/handle/activation/service_lifecycle.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/reactivation.rs
  - zircon_runtime/src/core/runtime/tests/activation/structure/reactivation.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior.rs
tests:
  - cargo test -p zircon_runtime --lib core::runtime::tests::resolution::behavior::deactivation_invalidates_registered_manager_identity_before_reactivation --locked -- --exact --nocapture
  - cargo test -p zircon_runtime --lib core::runtime::tests::activation::behavior::reactivation --locked -- --nocapture
  - rustc --edition 2021 --test zircon_runtime/src/core/runtime/tests/activation/structure/mod.rs
---


# Frameworks05: manager service 重新激活生命周期回归

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 来源执行切片：Runtime15 resolution test owner split 后的 current-source focused behavior gate
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：失败位于 Frameworks05 M4 新增的 versioned manager slot 生命周期；Runtime15 只拥有结构与回归守卫，不拥有模块重新激活语义。

## 失败现象与复现证据

当前源码 lib-test 二进制运行 `deactivation_invalidates_registered_manager_identity_before_reactivation` 为 **0 passed / 1 failed**。模块停用后旧 identity 正确变 stale，但 `activate_module("StaleIdentityModule")` 没有把该模块的 service slot 从 `Unloaded` 恢复为可解析状态；随后 `registered_manager_identity` 在 `behavior.rs:292` 返回 `ServiceUnavailable("StaleIdentityModule.Manager.IdentityManager")`。

## 最低共享层根因

`deactivate_module` 会对完整模块 service owner 列表执行 `invalidate_for_unload`，清除实例、推进 generation 并把 slot 置为 `Unloaded`；单模块与批量 activation 目前只把 module entry 置为 `Initializing`，没有恢复其完整 `service_names`。Immediate startup service 因而在工厂执行前被 availability gate 拒绝，lazy service 也永久保持 `Unloaded`。失败 activation 的 module/service rollback 同样只恢复首次注册语义，不能恢复重新激活前的 `Unloaded` 状态。

## 架构修复验收

- 单模块与 `activate_registered_modules` 重新激活都恢复完整 module-owned service slot；Immediate service 在 activation 内解析，lazy service 保持未构造直到 use-point resolution。
- slot index 保持稳定；停用推进 generation 一次，失败重新激活若丢弃已构造实例则再次推进 generation，未构造 lazy slot 不产生虚假 generation。
- 重新激活失败后 module 与全部 service slot 回到 `Unloaded`，不遗留 `Registered`/`Initializing` 半状态；首次 activation 的原有 rollback 语义不回归。
- 原始 Runtime15 reproduction、单模块/批量/lazy/rollback focused tests 和 Frameworks05 manager contract gate通过。

## 禁止临时方案

- 禁止放宽 `ensure_service_resolution_available`、跳过 stale generation 校验或在 `registered_manager_identity` 中特殊处理 reactivation。
- 禁止恢复旧 manager resolver/Arc holder、添加兼容 shim、双轨生命周期或 test-only bypass。
- 禁止只恢复 startup service；完整 `service_names`（包括 lazy service）必须共享同一 module-owned transition。
- 禁止削弱 Runtime15 测试或计划验收条件来隐藏失败。

## 修复结果与回传

- 根因：停用把完整 module-owned service slot 置为 `Unloaded`，而 activation 只恢复 module entry，并错误复用首次 activation rollback 语义。
- 架构修复：`activation/service_lifecycle.rs` 统一拥有单模块/批量完整 `service_names` 的 prepare 与 rollback；`ServiceEntry` 按是否丢弃实例推进 generation，transition 在释放 registry lock 后通知 waiter。
- 验证：受管默认特性 `zircon_runtime` build 通过；当前公共 API probe 3/3；activation structure 7/7；Cargo behavior 3/3、原始 Runtime15 reproduction 1/1；Frameworks05、全局文件预算与 F18 guard 均通过。
- 回传：Runtime15 manager reactivation gate 已恢复；Render18 的三个独立文件预算失败仍保留。
