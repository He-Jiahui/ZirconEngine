---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-19
summary_slug: validation-result-caller-authority
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_server.py
  - tools/session_coordinator/tests/test_validation_tickets.py
resolved_at: 2026-08-22
---


# validation-result-caller-authority: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Coordinator validation trust chain
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Coordinator validation trust chain` — Invoke validation.record_result for a queued ticket with status passed and arbitrary evidence; observe ticket becomes passed without a managed validation-copy run.

## 最低共享层根因

CoordinatorApplication.command exposes validation.record_result to every local command caller while the real ValidationTicketWorker already records terminal results internally.

## 架构修复验收

- External command callers cannot transition validation tickets to passed, failed, or snapshot_stale.
- The managed ValidationTicketWorker can still record terminal results derived from durable validation-copy evidence.
- Rejected external result writes leave ticket status and failure artifacts unchanged.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：The public command dispatcher exposed validation.record_result, allowing any local command caller to mark a queued ticket terminal without managed validation-copy evidence.
- 架构修复：Reject all external validation result transitions before reading caller evidence; retain the sole terminal write path inside ValidationTicketWorker to ValidationTicketService.record_result using durable managed-run evidence.
- 验证：Managed ticket 1f77fea4d5974c928b103778bfac9ee7 passed 23 validation-ticket and 4 focused server authority tests; unchanged test_validation_tickets.py remains exact. Current server.py and test_server.py passed their full module coverage in ticket 8207f4f1e5464f499d055ccb169400ab, 130/130.
- 回传：External callers can no longer forge validation ticket terminal status; worker-derived managed evidence is the only result authority.
