---
handoff_kind: fixed
status: fixed
created_at: 2026-08-23
summary_slug: validation-copy-external-source-error-details
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/tests/test_workspace_copy.py
  - tools/session_coordinator/workspace_copy.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_workspace_copy -v
  - python -m unittest tools.session_coordinator.tests.test_validation_copies -v
resolved_at: 2026-08-23
---

# Coordinator01: validation-copy external-source details are discarded

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Frameworks01 current-source managed validation-copy materialization
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Frameworks01 supplied the production reproduction, while Coordinator01
  owns both sides of the validation-copy failure persistence contract.

## 失败现象与复现证据

Frameworks01 validation copy `423dc385d9934db18af21304e621ae3f` failed during
`closure_planning` with `validation_copy_external_source_missing`. The durable
`validation_copies` row persisted `error_path=NULL` and `error_details_json={}` even
though every production raise for that error supplies `manifestPath`, and the
restricted sibling-repository branch also supplies `repoRoot`. The owning Session
therefore cannot identify the missing Cargo manifest without reproducing planner
internals outside the managed copy.

RED proof reproduces the same asynchronous worker path with an exact
`CoordinatorError` carrying `manifestPath` and `repoRoot`: before the fix,
`WorkspaceCopyService.status()` returns `error_path=None` and empty `errorDetails`.

## 最低共享层根因

`WorkspaceCopyService._fail_materialization()` receives the complete structured
details, but `_materialization_error_details()` uses a bounded allowlist that omits
both `manifestPath` and `repoRoot`. Its legacy `error_path` projection omits those
keys as well. The worker and planner preserve the evidence; only the durable
persistence boundary discards it.

## 架构修复验收

- Preserve exact `manifestPath` and optional `repoRoot` in bounded durable
  `error_details_json` and the public `errorDetails` projection.
- Project `manifestPath` as the primary legacy `errorPath`, falling back to
  `repoRoot` only when no manifest is available.
- Retain existing `sourcePath/resourcePath`, Git-operation, and unexpected-error
  persistence without widening to arbitrary caller-controlled detail keys.
- Add an asynchronous Cargo worker regression covering both status and raw durable
  row evidence.

## 禁止临时方案

- Do not edit Frameworks source, its immutable validation copy, or its durable row.
- Do not enable automatic external-source discovery or weaken descriptor validation.
- Do not persist arbitrary exception dictionaries; keep the structured allowlist.

## 修复结果与回传

- 根因：The bounded validation-copy error allowlist omitted manifestPath and repoRoot, so the async worker discarded otherwise typed external-source evidence.
- 架构修复：Persist manifestPath and repoRoot as bounded structured details and project manifestPath, then repoRoot, through the compatible errorPath field.
- 验证：RED reproduced empty details; focused persistence regressions passed 4/4; failure parser passed 28/28; combined workspace-copy and planner suite passed 70/71 with one unrelated Windows SQLite teardown lock, whose exact test passed 1/1 in isolation.
- 回传：Coordinator validation-copy failures now identify the exact missing external Cargo manifest without weakening descriptor admission or rewriting the failed Frameworks copy.
