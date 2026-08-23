---
handoff_kind: failure
status: open
created_at: 2026-08-23
summary_slug: validation-ticket-terminalization
origin_plan: docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
origin_child_dir: docs/plans/optimize/zircon_tooling/10
fixing_child_dir: docs/plans/optimize/zircon_tooling/06
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/validation_ticket_worker.py
  - tools/session_coordinator/workspace_copy.py
tests:
  - python -u -B -m unittest tools.session_coordinator.tests.test_validation_tickets -v
  - validation ticket 9cc6e9bab31941b2b8aca8bfc0cd28fd
  - candidate validation ticket 44e8ea02753c448b980a3689cbce58f6 (six-module batch, 181 passed)
---

# Tooling 06: validation ticket loses an interrupted generic materialization

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md`
- 来源执行切片：M0 Hub inline-test reachability / isolated validation batch
- 修复责任计划：`docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 交接原因：validation ticket worker owns the transition from validation-copy state to a durable validation result; Tooling 10 only consumes that result.

## 失败现象与复现证据

Tooling 10 resubmitted its M0 batch as ticket `9cc6e9bab31941b2b8aca8bfc0cd28fd` after completing its dependency-root closure. The coordinator daemon restarted while generic validation-copy job `2ffa7a02678f44088548a06ff90c81db` was materializing and before a run link existed. Startup recovery correctly removed that incomplete generic copy, but the ticket worker then terminalized the still-materializing ticket as failed:

- the copy row had `status=removed`, `materialization_kind=null`, `run_pid=null`, and no `validation_copy_runs` row;
- `ValidationTicketWorker._advance_materializing` treated every removed copy as an already-started run and delegated to `_finish_from_run`;
- `_finish_from_run` recorded `validation_ticket_run_terminal_missing` with `copyStatus=removed`, even though no validation command had started.

The older ticket `e56f23fb55b14274994f3dab84c493b3` did eventually produce a run result and was not evidence for this defect. The replacement ticket is a separate, durable reproduction of the pre-run restart window.

## 最低共享层根因

`WorkspaceCopy.recover_interrupted_jobs()` is correct to remove an incomplete ordinary copy after daemon restart. The lowest broken layer is `ValidationTicketWorker._advance_materializing`: for a non-Cargo ticket with neither `validation.ticket_run_linked` nor a durable run result, `removed` is an interrupted materialization, not terminal run evidence. The worker must recheck the original source manifest and create a replacement generic copy; a run-linked ticket or durable run result must continue through the existing evidence path.

## 架构修复验收

- `test_worker_restarts_a_removed_generic_copy_before_run` proves the worker links a fresh generic copy and leaves the ticket materializing.
- `test_worker_uses_durable_result_from_removed_generic_copy_without_run_link` proves successful and failed durable runs terminalize without a duplicate materialization.
- The focused validation-ticket suite passes in the coordinated batch.
- Tooling 10 resubmits its original M0 batch after the recovered worker is deployed; it must reach a real terminal run result rather than `validation_ticket_run_terminal_missing`.

## 禁止临时方案

- Do not mark a pre-run `removed` copy as passed, failed, or a synthetic run result.
- Do not add a Tooling 10-specific retry or require callers to manually resubmit an interrupted ticket.
- Do not change the recovery rule for Cargo materializations, which own a different durable restart protocol.

## 修复结果与回传

- 候选快照已通过独立六模块回归批：validation ticket
  `44e8ea02753c448b980a3689cbce58f6` 对 validation ticket、workspace copy、
  Cargo runner、Cargo jobs 与 Windows Job Object 的 181 项测试均通过。
- 该批次证明 worker 的恢复分支和本地回归覆盖；它不替代来源 Tooling10 M0
  的真实受管运行回放。Coordinator 必须先加载此 worker，再重放来源批次并取得
  durable terminal run result。

Open state: `待原始受管回放`; no Tooling10 pass or performance qualification is claimed.
