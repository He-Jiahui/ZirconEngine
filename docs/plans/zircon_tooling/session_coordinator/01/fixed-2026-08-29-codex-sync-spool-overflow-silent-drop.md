---
handoff_kind: fixed
status: fixed
created_at: 2026-08-29
summary_slug: codex-sync-spool-overflow-silent-drop
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/codex_sync/spool.py
  - tools/session_coordinator/codex_sync/worker.py
  - tools/session_coordinator/tests/test_codex_spool.py
  - tools/session_coordinator/tests/test_codex_worker.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_codex_spool -v
  - python -m unittest tools.session_coordinator.tests.test_codex_worker -v
resolved_at: 2026-08-29
---

# Coordinator01: Codex sync spool overflow silently drops pending triggers

## Source executor

- Origin plan: `docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- Guidance: `TOOL-COORD-P1-039` in `docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- Fix owner: Coordinator01

## Failure and reproduction

`CodexTriggerSpool.enqueue()` writes a new trigger and then `_enforce_cap()` unlinks the oldest pending JSON files whenever the queue exceeds `max_pending`. The deleted trigger was already accepted by the hook, but no rejection, overflow marker, receipt, metric, or control-plane state records the loss.

A spool with `max_pending=3` accepts five triggers, silently deletes sessions 0 and 1, and reports only the surviving three items. Operators cannot distinguish this data loss from a normally drained queue.

## Lowest shared cause

Queue admission and overflow reporting are coupled to a destructive post-write trim. The spool has no durable overflow state, and `CodexSyncWorker.snapshot()` exposes no spool overflow projection. Capacity therefore preserves a numeric bound by discarding accepted work without an observable terminal outcome.

## Architecture acceptance

- Never delete an older accepted pending trigger to admit a newer trigger.
- When the queue is full, reject the new trigger and preserve every accepted pending file.
- Persist a repository-scoped, privacy-bounded overflow marker before rejecting the new trigger.
- Project the marker through the Codex sync worker snapshot so control health makes overflow visible across process restarts.
- Keep corrupt marker handling fail-closed and avoid exposing trigger payload or session identifiers.
- Preserve valid pending reconciliation and acknowledgement behavior.

## Forbidden shortcuts

- Do not increase the 1,024 item cap or silently rotate overflow markers.
- Do not move accepted triggers into quarantine or claim that deletion is acknowledgement.
- Do not block the Codex hook on coordinator availability or add a retry loop.
- Do not include trigger payload, cwd, model, prompt, session ID, or turn ID in overflow evidence.

## 修复结果与回传

- 根因：CodexTriggerSpool._enforce_cap wrote each new trigger before destructively unlinking accepted oldest pending files, so accepted hook work was silently lost when the bounded queue filled.
- 架构修复：Queue admission now records a bounded repository-scoped overflow marker, removes only the unaccepted new trigger, and raises OverflowError; all previously accepted pending files remain durable. CodexSyncWorker projects sanitized marker status through its service snapshot, including invalid-marker fail-closed state.
- 验证：Focused managed-equivalent Coordinator tests: codex spool and worker 14/14, Codex hook 10/10, server Codex wiring 1/1, worker evidence projection consumer 1/1, py_compile and diff check passed; no raw Cargo.
- 回传：Coordinator01 Codex sync spool overflow now rejects unaccepted triggers without dropping accepted work and exposes durable bounded overflow evidence.
