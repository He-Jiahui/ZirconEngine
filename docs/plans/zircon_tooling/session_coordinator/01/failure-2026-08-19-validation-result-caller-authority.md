---
handoff_kind: failure
status: open
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

- RED：`python -m unittest tools.session_coordinator.tests.test_server.ServerTests.test_external_validation_result_cannot_mark_queued_ticket_passed -v` 在修复前失败，外部 `validation.record_result` 没有抛出错误并把 queued ticket 写成 `passed`。
- 修复：公共命令入口现在在读取 ticket、evidence 或 failure payload 前统一返回 `validation_ticket_result_worker_only`；唯一保留的终态写路径是 daemon 内部 `ValidationTicketWorker -> ValidationTicketService.record_result`。
- 无副作用证明：外部 `passed`/`failed` 写入均保持 ticket 为 `queued`，且不会创建 failure artifact；内部 worker/service 路径仍能基于 validation-copy terminal evidence 写入终态并供 integration candidate 使用。
- GREEN：`python -m unittest tools.session_coordinator.tests.test_validation_tickets -v` 为 23/23；四个受影响的 `ServerTests` 聚焦用例为 4/4。生产修复后的完整 `test_server` 模块用例也全部通过，随后仅因命令中误写了不存在的 `test_validation_ticket_worker` 模块名和旧 boundary 预期而返回非零；旧预期已修正并由上述 23/23 覆盖。
- 尚待受管收口：基于四个精确路径提交 immutable validation ticket，通过后由 Coordinator integration candidate 提交并 rollover；在此之前保持 `status: open`。
