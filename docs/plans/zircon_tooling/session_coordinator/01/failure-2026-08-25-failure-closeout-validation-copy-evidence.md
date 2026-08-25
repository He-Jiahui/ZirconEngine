---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-25
summary_slug: failure-closeout-validation-copy-evidence
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/tests/test_failure_closeout.py
  - tools/session_coordinator/workflows/failure_closeouts.py
---

# failure-closeout-validation-copy-evidence: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：tools/session_coordinator/workflows/failure_closeouts.py::FailureCloseoutWorkflowService._load_validation_contract
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`tools/session_coordinator/workflows/failure_closeouts.py::FailureCloseoutWorkflowService._load_validation_contract` — Closeout prepare for managed validation ticket 4af7a6c358e247f0a31eee66faead640 and validation-copy job afe4ae75bf86465381baf558c6abc371 fails failure_closeout_validation_owner_mismatch because the service only queries cargo_jobs/cargo_job_runs.

## 最低共享层根因

Failure closeout hard-codes Cargo lane evidence tables even though Coordinator-issued non-Cargo validation tickets execute through validation_copies and validation_copy_runs with immutable source manifests and terminal evidence.

## 架构修复验收

- Prepare and bind accept an exit-0 passed validation ticket only when the worker-issued copy/run links, copy, run, ticket, session, command, and immutable non-plan source manifest all match.
- Nonzero, nonterminal, foreign-session, command-drift, source-drift, or mixed Cargo/copy evidence remains fail-closed.
- Existing Cargo closeout validation behavior and payload compatibility remain unchanged.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

Open state: `待修复`; the coordinator must keep the validation ticket and route this Plan to repair work.
