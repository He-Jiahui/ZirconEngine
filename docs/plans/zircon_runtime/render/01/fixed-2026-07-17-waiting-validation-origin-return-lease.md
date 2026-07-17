---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: waiting-validation-origin-return-lease
origin_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/render/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/server.py
tests:
  - tools/session_coordinator/tests/test_server.py::ServerTests::test_scoped_failure_return_allows_waiting_validation_origin_destination_lease
resolved_at: 2026-07-17
---


# Coordinator01: FIFO Waiting Origin Cannot Receive Child Return

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 来源执行者：`render01-f2-basic-scene-render-20260717`
- 来源执行切片：Render01 current-source validation waiting behind an existing FIFO reservation.
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：generated `fixed-*.md` records are coordinator-owned lifecycle output and must not require the fixing Session to take the origin directory lease.

## 失败现象与复现证据

- Coordinator01 completed the source-manifest capacity repair and attempted to return it through the typed `child_record_only` lifecycle.
- The active Render01 origin Session held the matching plan path and directory lease, but its correct FIFO state was `waiting_validation`.
- `_require_origin_destination_lease` only searched `active` and `resolving_failure`, so it rejected the return with `failure_return_lease_missing` before writing or deleting any artifact.

## 最低共享层根因

The cross-plan return selector treated `waiting_validation` as terminal for origin-lease authorization even though it is a live non-terminal Session state with a valid retained directory lease and a pending FIFO reservation.

## 架构修复验收

- Allow only `waiting_validation` in addition to the existing `active` and `resolving_failure` origin owner states for the exact generated fixed destination.
- Continue to require matching origin `plan_path`, a live overlapping origin lease, and separately owned fixing failure plus return receipt leases.
- Do not transfer, release, or broaden the origin directory lease; stale, cancelled, completed, unrelated-plan, and missing-lease Sessions remain rejected.
- Cover a child-only return whose origin owner transitions `registered -> active -> waiting_validation` before the selector runs.

## 禁止临时方案

- Do not change the Render01 Session to `active` merely to make a return succeed.
- Do not claim or release the Render01 directory lease from the fixing Session.
- Do not manually create a fixed file or alter global parent plans.

## 修复结果与回传

- 根因：The child-record return selector excluded waiting_validation even though that Session is non-terminal, owns the matching origin plan and directory lease, and is legitimately waiting on FIFO validation.
- 架构修复：The selector now accepts waiting_validation only alongside active and resolving_failure after it verifies the exact origin plan path and live overlapping destination lease. The fixing Session still owns only its failure and receipt; no origin lease is transferred or released.
- 验证：Focused server regression first reproduced rejection for registered-to-active-to-waiting_validation origin owner, then passed after the selector change; full test_server passed 34/34; handoff validation passed 237 artifacts with 0 errors; live retry successfully returned fixed-2026-07-17-source-manifest-build-config-cap.md under the waiting Render01 directory lease.
- 回传：Coordinator01 returned the waiting-validation origin lease fix after proving it through the blocked source-manifest capacity return. Existing Render01 FIFO reservation and all foreign leases remain unchanged.
