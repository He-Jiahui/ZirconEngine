---
handoff_kind: fixed
status: fixed
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
tests:
  - python -m unittest tools.session_coordinator.tests.test_failure_closeout -v
resolved_at: 2026-08-25
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

- 根因：Failure closeout hard-coded Cargo job and run tables even though coordinator-issued non-Cargo tickets execute through validation copies and worker-issued copy/run links.
- 架构修复：Accept a distinct validation-copy evidence contract only when ticket, immutable copy, terminal run, session, command, canonical repository-relative source manifest, and latest worker-issued copy/run links match; bind those durable event identities into the closeout contract while preserving the existing Cargo path and its stricter evidence-side plan-entry comparison.
- 验证：Managed validation ticket 3da872f06aef4f24b294141c717736fc passed all 29 original failure-closeout tests from copy a5db596e02b6418b96a7ac75295957c4 with source manifest d179f50cb080cd0eabbb5211880975c9d441b25e5693d8759c0fa56e147b38a7. Commit 8ee9411db24b7b4bdaf3fe028194642a7557c0b6 loaded through rollover action 403963830a134a4db55654cee570786c into healthy schema67 successor 52372200cd5d4881b459dfb115777cac. Production closeout request a5020730d89243f9bc8ac92fe3516f24 accepted validation-copy ticket 4af7a6c358e247f0a31eee66faead640, copy afe4ae75bf86465381baf558c6abc371, run 4af7a6c358e247f0a31eee66faead640 and advanced to the later preserved-scope-overlap gate. Independent review then caught evidence-side `docs/plans/` filtering that had broadened the Cargo contract plus a noncanonical validation-copy path escape; commit 264b2be6446843ea64606a5accbf465923f6523e restored the Cargo behavior and rejected traversal/noncanonical source paths. Current-source validation passed 31/31 in 352.772s; finalizer request 14af331df4844935a598ccafa531875d completed the two focused regressions and `py_compile`, and independent re-review reported C0/I0/M0.
- 回传：Validation-copy evidence is now a first-class immutable failure-closeout contract; real production replay passed that contract and failed closed only on an unrelated open lifecycle scope overlap.
