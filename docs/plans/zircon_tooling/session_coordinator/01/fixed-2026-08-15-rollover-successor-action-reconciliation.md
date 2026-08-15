---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-15
resolved_at: 2026-08-15
summary_slug: rollover-successor-action-reconciliation
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/client.py
  - tools/session_coordinator/tests/test_deferred_action_client.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_deferred_action_client
---

# Coordinator01: rollover successor action reconciliation repair

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Coordinator01 controlled schema-63 rollover
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the client-side preview, confirmation, and terminal action polling protocol.

## 失败现象与复现证据

Controlled action `4dced749e1ca491b95bd5319782f1104` successfully created
successor `8da999e1e611451d8d31507007b58f8b`. The durable journal later
recorded `succeeded` with intent `7ceeaf64de6d49d085f38f2265e0f44d`, but the
caller had already received `action_instance_mismatch` while querying the successor.

## 最低共享层根因

The client tolerated listener-offline and command-timeout observations during a
rollover, but treated the equally transient successor/predecessor action-identity
handoff as terminal even though the confirmed action was still within its deadline.

## 架构修复验收

`CoordinatorClient.execute_control_action` now retries
`action_instance_mismatch` only while polling a confirmed `service.rollover` action.
The existing monotonic deadline remains authoritative, and the client never repeats
preview or confirmation. Non-rollover actions continue to surface the identity error
immediately.

## 禁止临时方案

- Do not weaken server action identity checks or accept caller-supplied instance IDs.
- Do not retry identity mismatches for validation, Git, or other controlled actions.
- Do not add an unbounded rollover wait.

## 修复结果与回传

- 根因：The rollover client did not account for the successor's transient view of a predecessor-owned action during listener handoff.
- 架构修复：Retry that typed mismatch for `service.rollover` only, under the existing monotonic deadline and without repeating confirmation.
- 验证：The focused successor-reconciliation test passes, the non-rollover identity
  enforcement boundary passes, and the complete deferred-action client suite passes
  12/12. Schema-64 rollover action `07e66e3a28ef41178de21abc5924e64b`
  and intent `a21238a4095c42fca23e8df4c6553f31` both reached `succeeded` with
  successor `c4cc316b608a46b1803e186c8cbf5925`; a caller deadline that elapsed
  shortly before that terminal projection was reconciled from the durable journal
  without repeating preview or confirmation.
- 回传：The post-commit successor is healthy/read-write at schema 64 and its durable action identity is reconciled; no second rollover is required.
