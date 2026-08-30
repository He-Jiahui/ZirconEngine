---
handoff_kind: failure
status: open
created_at: 2026-08-29
summary_slug: validation-copy-explicit-overlay-resource
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/validation_copies.py
  - tools/session_coordinator/validation_ticket_worker.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/tests/test_validation_copies.py
  - tools/session_coordinator/tests/test_workspace_copy.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_validation_copies.ValidationCopySourceTests.test_compile_time_resource_accepts_declared_overlay_file -v
  - python -B -m unittest tools.session_coordinator.tests.test_validation_copies.ValidationCopySourceTests.test_compile_time_resource_accepts_any_declared_including_source -v
  - python -B -m unittest tools.session_coordinator.tests.test_validation_copies.ValidationCopySourceTests.test_compile_time_resource_rejects_live_untracked_file -v
  - python -B -m unittest tools.session_coordinator.tests.test_validation_copies.ValidationCopySourceTests.test_compile_time_resource_discovery_uses_bounded_git_arguments -v
  - python -B -m unittest tools.session_coordinator.tests.test_workspace_copy.WorkspaceCopyTests.test_cargo_materialization_passes_overlays_to_planner -v
  - python -B -m unittest tools.session_coordinator.tests.test_workspace_copy.WorkspaceCopyTests.test_cargo_worker_persists_stale_owned_overlay_path -v
---

# Validation copy rejects declared untracked compile-time resources

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：validation-copy Cargo closure planning
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns validation-copy closure planning and worker
  propagation; product plans own only the overlay content and final Cargo gate.

## 失败现象与复现证据

Cargo validation-copy closure planning enumerated compile-time `include_str!` and
`include_bytes!` resources through `git ls-files` only. A dirty or untracked source that was
already present in the ticket's explicit overlay could therefore reference another explicit
overlay resource and still fail with
`validation_copy_compile_time_resource_missing` before Cargo started.

This blocked the batched Runtime07, Runtime90, and RuntimeInterface01 validations on resources
whose exact content hashes were already present in their source manifests.

## 最低共享层根因

The planner queried tracked Git paths without considering the request's
explicit overlay manifest, so an untracked resource already covered by the
validated overlay was treated as missing.

## 架构修复验收

- `CargoInputClosurePlanner.plan` now accepts the request's normalized `overlay_paths`.
- Explicit live overlay sources participate in compile-time include discovery.
- A discovered resource is admitted when the including source is explicit, or when the resource
  itself (or a descendant for a dynamic resource root) is explicit.
- Shared resources record whether any including source is explicit, so admission is independent
  of source ordering when tracked and overlay sources include the same resource.
- Resources that are neither tracked nor declared remain rejected.
- Synchronous materialization and the durable Cargo worker both pass normalized `overlayPaths`
  into the planner before ownership and materialization checks.

## 禁止临时方案

- Do not whitelist untracked resources by prefix or bypass overlay ownership.
- Do not claim a managed Cargo validation pass from planner-only evidence.

## 修复结果与回传

- 根因：planner queried tracked Git paths without considering the request's explicit overlay manifest.
- 架构修复：`CargoInputClosurePlanner.plan` accepts normalized `overlay_paths`; explicit sources and
  resources participate in include discovery, while undeclared live resources remain rejected. The
  durable worker passes `overlayPaths` before ownership/materialization checks.
- 验证：declared source/resource overlay planner coverage, shared-resource ordering,
  undeclared-resource rejection, bounded Git pathspec batching, synchronous propagation, and
  durable-worker propagation passed fresh `6/6`; `py_compile` and scoped `git diff --check` remain
  required on the final exact snapshot.
- 回传：proof-bound successor replay and Runtime07/Runtime90/RuntimeInterface01 managed Cargo gates
  remain pending; this open artifact must not be returned until those gates pass.
