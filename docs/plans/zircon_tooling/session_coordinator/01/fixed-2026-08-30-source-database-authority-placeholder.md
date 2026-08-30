---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: source-database-authority-placeholder
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - .gitignore
  - tools/session_coordinator/session_coordinator.db
  - tools/session_coordinator/tests/test_database_authority.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_database_authority -v
resolved_at: 2026-08-30
---

# Coordinator01: source tree tracks a zero-byte database authority placeholder

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P2-003` in `docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the source-tree database authority contract.

## 失败现象与复现证据

Git tracks `tools/session_coordinator/session_coordinator.db` as a zero-byte file. No production source, test, script, or documentation references it. `CoordinatorConfig.database_path` correctly points to `.codex/state/session-coordinator/coordinator.sqlite3`, so the tracked placeholder has no runtime role and advertises a false source-tree authority.

The repository has no ignore rule for the obsolete path. A local tool can therefore recreate it and present the same misleading database candidate again.

## 最低共享层根因

An early database placeholder survived the cutover to repository-local Coordinator state. The source tree retained the filename without an ownership contract, while the authoritative config and migrations moved elsewhere.

## 架构修复验收

- Remove the tracked zero-byte placeholder without touching the real runtime database.
- Ignore only the obsolete source-tree path, not arbitrary `.db` or SQLite fixtures.
- Add a repository test proving the placeholder is absent/ignored and the runtime authority remains `CoordinatorConfig.database_path` under `.codex/state/session-coordinator`.

## 禁止临时方案

- Do not delete, move, open, migrate, or compact `.codex/state/session-coordinator/coordinator.sqlite3`.
- Do not add a broad `*.db` ignore that hides legitimate fixtures.
- Do not replace the placeholder with a generated database or symlink.

## 修复结果与回传

- 根因：The repository retained a zero-byte database placeholder after runtime state moved to the Coordinator state root, leaving a false source-tree authority with no ownership contract.
- 架构修复：Remove the tracked placeholder and ignore only the exact obsolete path; keep CoordinatorConfig.database_path as the sole runtime authority and never touch the real SQLite state.
- 验证：RED failed the placeholder absence and exact-ignore assertions; GREEN passed tools.session_coordinator.tests.test_database_authority 3/3, py_compile, and exact-path diff check.
- 回传：Returned Coordinator01 source-database-authority-placeholder as a fixed child artifact with durable runtime-authority and source-tree hygiene evidence.
