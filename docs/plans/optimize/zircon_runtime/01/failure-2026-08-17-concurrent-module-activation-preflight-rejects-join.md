---
handoff_kind: failure
status: open
created_at: 2026-08-17
summary_slug: concurrent-module-activation-preflight-rejects-join
origin_plan: docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
origin_child_dir: docs/plans/optimize/zircon_tooling/10
fixing_child_dir: docs/plans/optimize/zircon_runtime/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/activation.rs
tests:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -LibTests -SkipBuild -TestFilter concurrent_activation_shares_one_build_transaction
---

# Runtime 01: concurrent activation preflight rejects a valid transaction join

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md`
- 来源执行切片：runtime lib-test API drift convergence focused behavior gate
- 修复责任计划：`docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md`
- 交接原因：最低共享原因位于 Core module lifecycle coordinator 与 activation state preflight，不属于测试架构或 fixture owner。

## 失败现象与复现证据

协调器管理的真实 Cargo 测试退出 101。单测 `concurrent_activation_shares_one_build_transaction` 启动首个 activation 并阻塞在 build 后，第二个同模块 activation 没有加入同一 coordinator transaction，而是在 `activation.rs` 的全闭包状态预检中看到 `Initializing` 并返回 `InvalidModuleLifecycleTransition`。预期两个调用都成功且 lifecycle build callback 只执行一次；实际第二个调用失败。

## 最低共享层根因

`activate_module_with_ready_timeout` 在进入 `run_module_lifecycle_transition` 前调用 `validate_module_activation_states`。因此 coordinator 尚未来得及返回同命令的 `Completed`/join 结果，状态预检已经把合法的 in-flight `Initializing` 当作非法状态拒绝。模块状态与 transition coordinator 的 admission 顺序不一致。

## 架构修复验收

- 同模块并发 activate 必须由 lifecycle coordinator 串行化或加入同一 transaction，两个调用均成功，build callback 恰好一次。
- activate/deactivate 竞争、递归 lifecycle command 与失败结果复用仍保持 fail-closed，不得通过无条件接受所有 transient state 修复。
- 原始复现和 `core::runtime::tests::activation::behavior::` 行为组必须通过。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 不得通过 sleep、扩大 timeout 或把第二个 activation 的错误改成测试期望来掩盖 admission 顺序错误。

## 修复结果与回传

Open state: `source_recovered_managed_validation_pending`; current `HEAD` already admits
`Initializing` during activation-closure preflight and then delegates same-module serialization
to `LifecycleCoordinator`. This removes the stale source diagnosis that a valid join is rejected
before coordinator admission. The focused managed Windows regression still has to run against a
fresh current-source manifest before this failure can return as fixed.

## 产出记录与时间

| 时间 | 状态 | 完成项目与证据 |
| --- | --- | --- |
| 2026-08-24 CST | `source_recovered_managed_validation_pending` | Current-source re-review verified that `validate_module_activation_states` accepts `LifecycleState::Initializing`, and `activate_module_with_ready_timeout` subsequently calls `run_module_lifecycle_transition` for every closure member. `HEAD` commit `08094b9b9` (2026-08-22) already contains that admission behavior; `concurrent_activation_shares_one_build_transaction` remains present in the canonical activation behavior tree. No Cargo command ran in this review, so no runtime pass or fixed return is claimed. |
