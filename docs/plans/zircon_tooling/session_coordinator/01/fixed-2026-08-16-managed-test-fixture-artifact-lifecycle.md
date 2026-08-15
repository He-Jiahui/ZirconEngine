---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-16
resolved_at: 2026-08-16
summary_slug: managed-test-fixture-artifact-lifecycle
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/artifact_governance.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/tests/test_artifact_governance.py
  - tools/session_coordinator/tests/test_migrations.py
  - tools/session_coordinator/tests/test_server.py
  - tools/mvp/MvpTestFixturePaths.psm1
  - tools/tests/mvp-test-fixture-paths.Tests.ps1
  - tools/tests/build-editor.Tests.ps1
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_artifact_governance
  - python -B -m unittest tools.session_coordinator.tests.test_migrations
  - python -B -m unittest tools.session_coordinator.tests.test_server.ServerTests.test_artifact_fixture_commands_route_process_bound_lifecycle tools.session_coordinator.tests.test_server.ServerTests.test_artifact_fixture_cli_sends_only_prefix_lease_and_owner_pid
  - powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/tests/mvp-test-fixture-paths.Tests.ps1
  - Invoke-Pester tools/tests/build-editor.Tests.ps1
---

# Coordinator01: managed test fixture artifact lifecycle

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：UI12 build-editor finalizer-equivalent Pester validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns governed artifact admission, reservation recovery, and cleanup identity.

## 失败现象与复现证据

- `MvpTestFixturePaths.psm1` created `D:/ZirconBuilds/mvp-test-fixtures-$PID/<guid>` directly. The artifact guardian had no durable record for that live tree and could delete it between Pester `BeforeEach` and the nested PowerShell process that consumed the fixture.
- A finalizer-equivalent NUnit report captured the resulting time-of-check/time-of-use failure: the mock `validate-matrix.ps1` existed when `build-editor.ps1` checked it, then disappeared before `powershell -File` opened it.
- Four cleanup reservations for process parents `11376`, `29760`, `10976`, and `16996` survived after their physical directories disappeared. `artifact audit` returned an empty unmanaged set and ordinary cleanup returned no deletion, because reservation recovery ran only at service startup.

## 最低共享层根因

Artifact governance recognized Cargo jobs, validation copies, workflow artifacts, and cleanup reservations, but exposed no process-bound lifecycle for short-lived test fixtures. The scanner therefore treated an active fixture as unmanaged. Separately, normal cleanup scanned only physical children, so a missing reserved path could not reach the existing restart recovery path.

## 架构修复验收

- Schema 64 persists Coordinator-issued fixture lease ID, immutable target key/path, prefix, owner PID, owner process creation identity, status, and terminal timestamp.
- Acquire accepts only a validated prefix and live owner PID. The service chooses the governed `ZirconBuilds` root and generated target; callers cannot supply or broaden a path.
- Only an active lease whose PID and creation identity still match protects a path. A live lease also protects the interval before the caller creates its directory.
- Release is owner-bound and fails while the fixture directory still exists. Once released, recreating the same tree is unmanaged; there is no permanent prefix exemption.
- A dead owner with an existing tree is deleted through the normal identity-bound cleanup reservation. A dead owner whose tree is already absent is durably recovered without deletion.
- Ordinary cleanup and startup recovery both retire missing artifact reservations. Cleanup reservation overlap rejects a new fixture acquire instead of racing an in-progress tree deletion.

## 禁止临时方案

- Do not exempt `mvp-test-fixtures-*` by prefix, disable the guardian, extend cleanup timeouts, or let callers provide a target path.
- Do not treat PID alone as ownership; PID reuse must fail the process-creation identity check.
- Do not release a lease before deleting its physical tree or delete a live leased tree through a second cleanup path.
- Consumer adoption by `MvpTestFixturePaths.psm1` must occur only after the schema-64 successor is committed and healthy; tests must use the real service lifecycle rather than a mocked prefix exemption.

## 修复结果与回传

- 根因：Active test fixtures and missing cleanup reservations had no continuously reachable managed lifecycle in artifact governance.
- 架构修复：Added process-identity-bound fixture leases, service-issued governed paths, delete-before-release enforcement, and cleanup/startup recovery for both existing and missing stale targets.
- 验证：Managed maintenance commit `54da7a6cc175831f28d47fafc29f3ee8732e7c98` passed the 29/29 service gate. Controlled rollover action `07e66e3a28ef41178de21abc5924e64b` loaded healthy schema-64 successor `c4cc316b608a46b1803e186c8cbf5925` and cleared all four durable stale reservations. The real helper lifecycle contract exits 0, and finalizer-equivalent `build-editor.Tests.ps1` passes 15/15 in 250.88s without Cargo.
- 回传：`MvpTestFixturePaths.psm1` now acquires before creating, deletes before releasing, and proves a released path is unmanaged if recreated. Artifact governance remains strict and no prefix exemption was added.
