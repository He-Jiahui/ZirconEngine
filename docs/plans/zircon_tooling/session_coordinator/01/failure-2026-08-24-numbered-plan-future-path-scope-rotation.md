---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-24
summary_slug: numbered-plan-future-path-scope-rotation
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/ownership_transfers.py
  - tools/session_coordinator/tests/test_database.py
  - tools/session_coordinator/tests/test_ownership_transfers.py
---

# numbered-plan-future-path-scope-rotation: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Coordinator01 baseline-temp formal return and Frameworks01 immutable-scope successor handoff
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Coordinator01 baseline-temp formal return and Frameworks01 immutable-scope successor handoff` — ownership.transfer.preview for the required fixed-* and *-return.md paths returns path_missing/ineligible; session.set_status completed returns session_goal_close_requires_milestone, so the active numbered-plan primary cannot reserve generated return paths or legally rotate to a successor

## 最低共享层根因

The only audited write-scope extension consumes ownership-transfer candidates that already exist, while child-record-only failure.return requires live leases for two paths it will create; numbered-plan primary completion is correctly milestone-gated, leaving no reachable legal transition for future exact paths

## 架构修复验收

- Allow an executable primary Session to preview and apply an audited exact future-path reservation only when the path is absent, unowned, and has no overlapping foreign lease
- Revalidate missing-path identity, target Session, baseline epoch, attribution, and leases transactionally at apply; materialization or ownership drift after preview must fail closed
- Extend immutable write scope, acquire the exact lease, and persist attribution/transfer evidence without weakening existing abandoned-file transfer rules
- Prove a local child-record-only failure return can create its fixed and return records through the reserved paths while generic numbered-plan completion remains rejected

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

The RED regression proved an unowned missing path was always ineligible with
`path_missing`; the same dead end prevented a preview from acquiring the CAS state
needed to reject later materialization. The existing foreign parent-lease guard
already rejected overlap and remains unchanged.

The current repair extends the audited ownership-transfer model with explicit
`pathState: future`. A future candidate is eligible only when the exact path is
absent from the worktree and baseline, has no attribution, and has no overlapping
foreign lease. Apply re-runs those checks under the transfer transaction, acquires
the exact lease, extends immutable scope, and records a nullable content hash plus
the durable future state in schema 67. Existing transfers migrate as `existing`
with their hashes preserved. Preview replay remains idempotent; any intervening path
creation fails with `ownership_transfer_preview_stale`.

GREEN evidence includes the complete ownership-transfer suite `9/9`, the focused
schema persistence/idempotence/v66 migration/delegated-return suite `4/4`, and
server state-machine guards `2/2`. Python compilation and `git diff --check` also
pass. The ownership proof performs a real local child-record-only
`failure.return` through two reserved missing paths and confirms the fixed artifact
and return receipt are both created. Generic numbered-plan completion remains
rejected with `session_goal_close_requires_milestone`.

Open state: `implementation and regression suite accepted / maintenance commit,
schema-67 rollover, and production self-consumption pending`.
