---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: source-template-strategy-owner-guard-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/source_template.py
  - tools/zircon_export/source_template_plan_command.py
tests:
  - tools/tests/test_zircon_export_stage_handoff_strategy_owner_boundaries.py
---

# Plugins13 source template strategy owner guard drift

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：export stage handoff strategy ownership guard
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns the source-template plan and export strategy boundary.

## 失败现象与复现证据

The strategy owner-boundary suite required `source_template.py` to import the
strategy leaf directly. Current committed source delegates plan construction to
`source_template_plan_command.py`, which is the file that imports and calls the
strategy helpers, so the guard failed despite a correct production boundary.

## 最低共享层根因

The structural consumer inventory was not rotated when source-template plan
semantics moved out of orchestration. It encoded the former file location
instead of the current direct dependency edge.

## 架构修复验收

- Name `source_template_plan_command.py` as the direct strategy consumer.
- Assert that `source_template.py` delegates to the plan-command owner.
- Assert that orchestration does not regain a direct strategy dependency.
- Pass the complete stage-handoff strategy owner-boundary suite.

## 禁止临时方案

- Do not add an unused strategy import to `source_template.py`.
- Do not weaken direct-consumer checks for the other export stages.
- Do not move plan semantics back into orchestration to satisfy a stale guard.

## 修复结果与回传

The guard now follows the committed direct import and separately protects the
orchestration delegation boundary. The complete strategy and adjacent
source-template owner suites pass 20/20, the changed test compiles, and the
scoped diff gate is clean. The exact-two coordinator finalizer must reproduce
the same owner-boundary suite without foreign worktree inputs.

Open state: `source_validated / failure_return_pending`.
