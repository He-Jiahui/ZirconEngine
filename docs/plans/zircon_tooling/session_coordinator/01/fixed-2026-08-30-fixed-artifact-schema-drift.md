---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: fixed-artifact-schema-drift
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/tests/test_failures.py
tests:
  - python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py --repo-root E:\\Git\\ZirconEngine
  - python -u -m tools.session_coordinator.cli failure audit
resolved_at: 2026-08-30
---

# Coordinator01: fixed handoff artifacts omit the canonical schema sections

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：2026-08-30 Coordinator fixed-artifact audit
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the fixed handoff records and the validator contract that protects their provenance and lifecycle evidence.

## 失败现象与复现证据

The current Coordinator child directory contains seven recent `fixed-*` artifacts whose
body headings and source provenance labels are in an incomplete English-only form. The
repository handoff validator therefore reports nine schema errors per artifact: five
missing canonical headings and four missing source executor fields. The existing fix
evidence and frontmatter remain present, but the records cannot be independently audited
through the canonical schema.

## 最低共享层根因

Fixed-artifact writers emitted a partial English section shape instead of the repository's
canonical handoff headings and provenance labels. The lifecycle move preserved the bytes,
but no schema check ran before the fixed record was accepted.

## 架构修复验收

- Normalize only the seven Coordinator-owned fixed records to the canonical headings and provenance fields without changing their factual evidence.
- The handoff validator must report no schema errors for those seven records.
- `failure audit` must retain all open ownership and graph diagnostics; this repair must not relabel, close, or suppress any product failure.

## 禁止临时方案

- Do not weaken the validator, add a fixed-artifact allowlist, or ignore schema errors for `fixed-*` records.
- Do not rewrite unrelated foreign or historical artifacts, alter frontmatter ownership, or delete evidence to make the audit smaller.

## 修复结果与回传

- 根因：Coordinator fixed artifacts were emitted with incomplete English-only headings and provenance labels, so canonical handoff schema checks could not audit them.
- 架构修复：Normalized the seven Coordinator-owned fixed records to the canonical Chinese headings and provenance fields without changing ownership, frontmatter, or factual evidence.
- 验证：The handoff validator no longer reports errors for the seven normalized Coordinator records; failure audit retains the existing cross-plan diagnostics; scoped git diff check passed.
- 回传：Coordinator fixed-artifact schema drift is repaired and returned; foreign and historical diagnostics remain with their owners.
