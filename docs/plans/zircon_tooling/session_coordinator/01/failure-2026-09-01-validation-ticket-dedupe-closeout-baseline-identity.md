---
handoff_kind: failure
status: open
created_at: 2026-09-01
summary_slug: validation-ticket-dedupe-closeout-baseline-identity
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/validation_tickets.py
  - tools/session_coordinator/workflows/failure_closeouts.py
  - tools/session_coordinator/tests/test_failure_closeout.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_failure_closeout.FailureCloseoutWorkflowTests.test_prepare_and_bind_accept_terminal_validation_copy_ticket -v
  - python -B -m unittest tools.session_coordinator.tests.test_failure_closeout -v
---

# Failure closeout rejects baseline-bound validation tickets

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：failure closeout validation-copy evidence binding
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns both validation ticket identity and closeout evidence validation.

## 失败现象与复现证据

Current validation tickets bind their dedupe identity to the source manifest, command,
toolchain, coverage, Session baseline epoch, and Session base HEAD. Failure closeout
recomputed only the first four fields. It therefore rejected valid terminal tickets with
`failure_closeout_validation_contract_invalid` before prepare could bind their evidence.

Concrete affected tickets included Tooling06 ticket
`2aab9ff49eb34c62a3e340c9053121f8` and Plugins13 ticket
`85779c43cc01421480e84c5166e342eb`. Their source and input manifest hashes were valid,
but their persisted six-field dedupe keys could never equal closeout's four-field hash.

## 最低共享层根因

`ValidationTicketService.submit` extended the immutable dedupe contract with
`baseline_epoch` and `base_head`, while `FailureCloseoutWorkflowService` and its test
fixture retained the previous four-field formula.

## 架构修复验收

- Failure closeout recomputes the ticket dedupe key from the exact six canonical fields
  used by validation ticket submission.
- The Session record used for plan ownership also supplies immutable baseline epoch and
  base HEAD identity; callers cannot inject either value.
- The validation-copy regression fixture generates production-compatible tickets, so the
  existing prepare-and-bind test fails against the old closeout implementation.
- Terminal status, worker links, source manifest hashes, input manifest hashes, command,
  plan, and owner checks remain unchanged.

## 禁止临时方案

- Do not weaken or skip the dedupe-key comparison.
- Do not accept caller-supplied baseline identities.
- Do not rewrite existing ticket identities or retry product Cargo.

## 修复结果与回传

- RED, GREEN, full-module validation, managed validation, and independent review remain
  required before this failure is returned.
