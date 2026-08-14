---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-14
summary_slug: validation-copy-compiletime-resource-error-details
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/tests/test_database.py
  - tools/session_coordinator/tests/test_validation_tickets.py
  - tools/session_coordinator/tests/test_workspace_copy.py
  - tools/session_coordinator/validation_ticket_worker.py
  - tools/session_coordinator/workspace_copy.py
resolved_at: 2026-08-14
---


# validation-copy-compiletime-resource-error-details: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Runtime09/UI12 exact-7-path managed validation materialization
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Runtime09/UI12 exact-7-path managed validation materialization` — Tickets 69750716903a4254bea2401665adb421 and 0f9c7e3efa1348a58710d58280a9360c failed in closure_planning with validation_copy_compile_time_resource_missing; jobs 4e6b2d53fa8d4276bf295198ec2a7256 and bc4600f60d5446cb932a883a7bec57ab persist error_path NULL although CoordinatorError.details contains sourcePath zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/host_adapter.rs and resourcePath zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/tests.rs.

## 最低共享层根因

WorkspaceCopyService._fail_materialization only maps CoordinatorError details.path or details.paths into a single error_path column; validation_ticket_worker then projects only that lossy field.

## 架构修复验收

- Persist exact sourcePath/resourcePath as structured validation-copy error details while retaining a compatible primary errorPath.
- Project the structured details into durable validation-ticket failure evidence and status responses.
- Add migration and regressions for compile-time missing resources plus existing generic path failures.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：WorkspaceCopyService persisted only details.path/details.paths into error_path, discarding compile-time sourcePath/resourcePath; ValidationTicketWorker consequently exposed only the lossy legacy field.
- 架构修复：Schema 62 adds durable error_details_json; WorkspaceCopyService persists a bounded structured path whitelist while keeping resourcePath as legacy errorPath, and ValidationTicketWorker projects errorDetails into terminal evidence.
- 验证：Local suites: test_database 22/22, test_validation_tickets 23/23, test_workspace_copy 59/59. Hot-loaded daemon schema 62 instance f6f000287095467284637bd6722480e6. Managed copy e9d49ed23dda44c89be5ae42dd314f53 manifest 3e8f161188bb51c91ae22433ef6d5a8c9d07af2d1baa98a7a901f2f87c31effb, run 6f0522bcef75406d9d2652bdac2050ea, 4/4 exit 0. No raw Cargo.
- 回传：Future validation-copy and ticket terminal evidence preserve exact sourcePath/resourcePath, so owners can identify compile-time resources without blind retries; Runtime15 retains ownership of the stale include itself.
