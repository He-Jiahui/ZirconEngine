---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: milestone-manifest-directory-lease-coverage
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/runtime/12
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/leases.py
  - tools/session_coordinator/workflows/milestones.py
tests:
  - tools/session_coordinator/tests/test_leases.py::LeaseTests::test_directory_owner_satisfies_an_exact_child_ownership_check
  - tools/session_coordinator/tests/test_workflow_commit.py::WorkflowCommitTests::test_bind_manifest_allows_directory_leases_to_cover_child_files
resolved_at: 2026-07-17
---


# Coordinator01: milestone manifest ignores directory lease coverage

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`
- 来源执行者：`runtime12-input-event-bounds-closeout-r2-20260717`
- 来源执行切片：M5 current-source input event bounds closeout
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Runtime12 owns coherent input module directories, but M5 manifest binding rejects their changed child files as unleased.

## 失败现象与复现证据

- Runtime12 owns live leases for `zircon_runtime/src/core/framework/input`, `zircon_runtime/src/input`, and the M5 child-plan directory.
- Its M5 record declares only current attributed files, yet `milestone validate` rejects `milestone_manifest_unleased` for every child file below those owned directories.
- `LeaseService.require_owned_live` already accepts an exact child path under an active directory lease; only milestone manifest binding bypasses that API.

## 最低共享层根因

`MilestoneWorkflowService._record_manifest` compared each manifest path against
`LeaseService.owned_paths()` by exact string membership. That bypasses the
canonical live hierarchy check, so an exclusive directory owner cannot bind a
manifest containing its own child files.

## 架构修复验收

- milestone binding must call the canonical live ownership check for every
  manifest path;
- a directory lease covers an exact descendant manifest file only for the same
  active Session;
- a child-file lease never grants ownership of a parent directory or unrelated
  sibling;
- expiry remains enforced by the same live-lease check; and
- the focused workflow regression must bind a manifest containing files under
  leased `src/` and `tests/` directories without duplicate leaf leases.

## 禁止临时方案

- 不得为每个 child file 额外创建重复 lease 以规避目录 owner 规则。
- 不得放宽为任意重叠路径均可覆盖 manifest。
- 不得手工 stage、commit 或删除 Runtime12 M5 记录。

## 修复结果与回传

- 根因：Milestone manifest binding compared paths against exact owned_paths membership and bypassed the canonical live directory hierarchy check.
- 架构修复：Milestone manifest binding now delegates every declared file to LeaseService.require_owned_live, where a same-Session directory lease covers exact descendants without granting parents or siblings.
- 验证：Current-source focused regressions passed 4 tests in 9.087s, including exact-child directory ownership, directory-backed milestone binding, and the cross-plan allow/reject controls; daemon instance 9c58203b34bb47ac9afd746a3a4290ac loaded schema 48 before this live return.
- 回传：Loaded and proved directory-backed milestone manifest ownership under the current daemon.
