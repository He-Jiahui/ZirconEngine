---
handoff_kind: fixed
status: fixed
created_at: 2026-08-29
summary_slug: control-http-admission-mock-background-interference
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/tests/test_control_http.py
tests:
  - python -u -B -m unittest -v tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_manual_queue_advance_uses_the_shared_non_reentrant_worker_gate
  - python -u -B -m unittest -q tools.session_coordinator.tests.test_control_http
resolved_at: 2026-08-29
---

# Coordinator01 control HTTP admission mock background interference

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：unoccupied failure-chain sweep of control HTTP identity and recovery tests
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：失败位于 Coordinator01 自有的 loopback validation queue regression，且根因是测试与同一 Coordinator 的后台 Codex worker 共享全局 admission mock。

## 失败现象与复现证据

control HTTP、auth、security 与 recovery 的 31 项组合测试中，仅
`test_manual_queue_advance_uses_the_shared_non_reentrant_worker_gate` 失败：测试预期
`require_mutation_allowed` 被调用两次，实际为三次。focused 重复可通过，表明第三次调用依赖
后台调度。随后在 mock 窗口中显式执行一次生产 `_codex_sync_writable()`，focused 测试稳定
复现同一 `AssertionError: 2 != 3`。

## 最低共享层根因

`RunningCoordinator.start()` 会同时启动 Codex reconcile worker。测试用 `patch.object` 替换
整个 `SupervisionService.require_mutation_allowed`，却把全局 mock 的总调用数当成 validation
queue 自身调用数。后台 worker 合法检查 `codex.sessions.reconcile` 时，断言便随线程调度漂移；
这不代表 queue 多做了一次 admission，也不影响其非重入锁行为。

## 架构修复验收

- 回归必须在同一 mock 窗口显式包含一次无关 `codex.sessions.reconcile` admission 检查。
- 只对精确 operation `validation.queue_continue` 断言两次 admission，不接受 operation 别名。
- 保持一次成功 tick、锁被占用时 `validation_queue_busy`、且 worker tick 不重入的原契约。
- focused regression 与完整 control HTTP 模块均通过。

## 禁止临时方案

- 不得停止 Codex worker、禁用后台 admission 或把 `RunningCoordinator` 换成不启动线程的伪对象。
- 不得删除 admission 次数断言或放宽为任意调用次数。
- 不得修改生产 queue/worker 行为来适配全局 mock 的测试噪声。

## 修复结果与回传

- 根因：The test treated the process-wide SupervisionService admission mock call count as validation-queue-local even though RunningCoordinator also runs the Codex reconcile worker, so a legitimate codex.sessions.reconcile admission produced a nondeterministic third call.
- 架构修复：Keep the real background worker active, inject one deterministic unrelated admission while the shared mock is installed, and assert exactly the validation.queue_continue operations so the two queue admission checks and non-reentrant validation_queue_busy contract remain strict.
- 验证：Source commit f660cfa9f3f84bff0903e4564ff1af4d065aee73; focused regression 1/1 in 4.657s; full tools.session_coordinator.tests.test_control_http 18/18 in 117.791s.
- 回传：Coordinator01 control HTTP admission regression is fixed and fresh focused/full verification is green; the canonical lifecycle may return to the origin plan.
