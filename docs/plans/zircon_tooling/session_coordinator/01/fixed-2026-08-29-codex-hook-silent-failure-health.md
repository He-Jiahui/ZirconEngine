---
handoff_kind: fixed
status: fixed
created_at: 2026-08-29
summary_slug: codex-hook-silent-failure-health
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/codex_sync/hook.py
  - tools/session_coordinator/codex_sync/spool.py
  - tools/session_coordinator/codex_sync/worker.py
  - tools/session_coordinator/tests/test_codex_hook.py
  - tools/session_coordinator/tests/test_codex_worker.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_codex_hook -v
  - python -m unittest tools.session_coordinator.tests.test_codex_worker -v
resolved_at: 2026-08-29
---

# Coordinator01: Codex Hook failures have no side-channel health evidence

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P1-041` in `docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the Codex Hook health evidence contract.

## 失败现象与复现证据

`run_hook()` intentionally returns success and emits valid Stop continuation output after invalid input, spool I/O failures, or coordinator signal failures. That non-blocking behavior is required, but every exception and rejected input is silently discarded. The durable spool proves successful enqueue only; neither a dropped input nor a failed wake signal appears in worker/service health.

Focused tests submit invalid private input and a valid trigger whose signaler returns false. The current source has no `hook_health_status()` and `CodexSyncWorker.snapshot()` has no `hookHealth` projection, so an operator cannot distinguish healthy inactivity from a Hook that has failed for every invocation.

## 最低共享层根因

The Hook's fail-open lifecycle boundary has no separate privacy-bounded diagnostic channel. Operational evidence is incorrectly coupled to the primary spool write and online wake request, even though those are exactly the operations that may fail.

## 架构修复验收

- Preserve non-blocking Hook exit and valid Stop continuation output for all failures.
- Persist repository-scoped, bounded health evidence for the latest success, error, and input drop without prompt, response, cwd, session, turn, model, or exception text.
- Distinguish a failed signal after durable enqueue from an enqueue failure, including whether pending work was persisted.
- Preserve prior error/drop timestamps when a later success occurs.
- Treat corrupt/oversized health evidence as invalid without surfacing its contents.
- Project sanitized health status through `CodexSyncWorker.snapshot()` so service health remains observable after restart.

## 禁止临时方案

- Do not make Hook failure block Codex or return a nonzero exit.
- Do not log payload, exception messages, absolute paths, identifiers, prompt, assistant content, tokens, or webhook data.
- Do not require coordinator availability to record a local health outcome.
- Do not add an unbounded event log or retry loop.

## 修复结果与回传

- 根因：The non-blocking Hook boundary discarded invalid input, spool failures, and wake failures without any independent durable health evidence.
- 架构修复：A repository-scoped bounded hook-health marker now atomically records sanitized success/error/drop state and pending durability, while the worker projects validated state through its snapshot.
- 验证：RED proved hook_health_status and hookHealth absent; GREEN focused 28/28, consumer 2/2, slow-daemon timing 5/5, py_compile and diff check passed.
- 回传：Codex Hook remains non-blocking while durable privacy-bounded health evidence now makes drops, enqueue errors, and deferred wake failures observable after restart.
