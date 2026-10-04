---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-09-14
summary_slug: validation-copy-manifest-missing-validation-ticket-policy
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
resolved_at: 2026-09-18
---

# validation-copy-manifest-missing-validation-ticket-policy: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Coordinator01 immutable-copy workspace-copy regression replay after validation-copies subset
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Coordinator01 immutable-copy workspace-copy regression replay after validation-copies subset` — With baseline a8e2f6aa3e42f832500a770f9f48ce188e6d9773 plus current owner overlays, importing tools.session_coordinator.tests.test_workspace_copy fails before test collection with ModuleNotFoundError: tools.session_coordinator.validation_ticket_policy.

## 最低共享层根因

The sealed source manifest cannot provide the reviewer-owned validation_ticket_policy.py module that validation_ticket_worker imports at module load; the current primary Session does not own that path.

## 架构修复验收

- A legitimate sealed immutable copy contains validation_ticket_policy.py at the reviewed current hash, and python -B -m unittest tools.session_coordinator.tests.test_workspace_copy -v executes with zero errors.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：The sealed immutable source manifest omitted tools/session_coordinator/validation_ticket_policy.py, which validation_ticket_worker imports at module load, so workspace-copy validation could not collect the requested regression suite.
- 架构修复：The coordinator now seals the legitimate reviewed immutable import closure including validation_ticket_policy.py and its worker dependencies; no compatibility shim, alias, fallback, or test bypass was added.
- 验证：Managed Windows Python 3.14 ticket 3331e178a3134441bb9b7c8d86d89b19, copy/job 10ca14b5f7684f8890b92bb99cdcc7ad, run 3331e178a3134441bb9b7c8d86d89b19, source manifest 200895f67ddafd3e3ff3a6a9fb2d8ca8334068d66a26e7b4e95e8c4b78d2eed4; python -B -m unittest tools.session_coordinator.tests.test_validation_copies tools.session_coordinator.tests.test_workspace_copy -v; 150 tests ran and OK.
- 回传：Coordinator generated the canonical fixed-* and return artifacts and atomically removed the original child failure record after the complete immutable-copy replay.
