---
handoff_kind: fixed
status: fixed
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
resolved_at: 2026-08-24
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

- 根因：Ownership transfer rejected every absent path as path_missing while local child-record-only failure return requires live leases for the fixed artifact and return receipt it will create, leaving no legal numbered-plan scope rotation transition.
- 架构修复：Persist explicit existing/future transfer state in schema67; admit only absent, unowned, baseline-free exact paths with no foreign overlapping lease, then transactionally revalidate CAS identity, extend immutable scope, acquire leases, and record nullable content-hash transfer evidence.
- 验证：Complete ownership-transfer suite 9/9; focused database migration/delegated-return suite 4/4; server state-machine guards 2/2; maintenance commit 0a5f22c944d802b0677ebeee5fc3168361bbac5c; healthy schema67 successor 8aa6dda23051465c9b855db429380551; production preview 68864a486a64447798f9c8c2709589cbd7e65d26926d66051346f4d4a89f8fca and apply eac49a63f9834c1c81dc486c722006ff enabled real failure.return dbce0bad6247463f82345c152582ff03, committed as 8a5bd5580000debd99bdd96e437cc7bc017468a7.
- 回传：Numbered-plan owners can now reserve exact future artifacts through audited CAS-bound transfer, enabling formal failure return and successor scope rotation without caller-controlled paths or weakened completion gates.
