---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-29
summary_slug: health-validation-copy-blocker-index-scan
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/supervision/service.py
  - tools/session_coordinator/tests/test_database.py
  - tools/session_coordinator/tests/test_migrations.py
---

# health-validation-copy-blocker-index-scan: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：schema68 live health/status projection diagnostics
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`schema68 live health/status projection diagnostics` — On schema68, GET /identity completed in 0.282s while GET /health took 8.889s. Direct timing isolated supervision.snapshot at 7.529s and its validation_copies blocker query at 4.979s over 2322 terminal rows; EXPLAIN QUERY PLAN reported a primary-key index scan and returned zero live rows.

## 最低共享层根因

The supervision health projection has no status-selective index for validation_copies, so every health request scans terminal copy history to prove that no running or cleanup_pending copy exists.

## 架构修复验收

- Add a migration-owned partial covering index for running and cleanup_pending validation copies without deleting terminal history.
- Prove the supervision blocker query uses the bounded index and still returns both active statuses with exact job/session identity.
- Prove schema upgrade and fresh schema reach the new version idempotently.
- Recheck live health under the default three-second client deadline after controlled rollover.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

Open state: `待修复`; the coordinator must keep the validation ticket and route this Plan to repair work.
