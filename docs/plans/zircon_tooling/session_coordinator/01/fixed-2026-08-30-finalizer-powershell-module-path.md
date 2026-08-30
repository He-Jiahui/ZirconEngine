---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: finalizer-powershell-module-path
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_git_finalize.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_git_finalize.GitFinalizeTests.test_powershell_validation_receives_module_path_without_caller_environment -v
  - powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/tests/codex-session-hook.Tests.ps1
resolved_at: 2026-08-30
---

# Coordinator01: finalizer PowerShell validation loses module discovery

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：2026-08-30 maintenance finalizer PowerShell validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the maintenance finalizer child-environment contract.

## 失败现象与复现证据

The Codex Hook installer acceptance passes when launched from the interactive shell,
but the same validation command fails inside the Coordinator maintenance finalizer.
The finalizer process inherits a service environment without `PSModulePath`, so
PowerShell cannot resolve the built-in `Get-FileHash` cmdlet used by the acceptance
test. The finalizer reports only `Validation command failed with exit code 1`.

## 最低共享层根因

Finalizer validation normalizes temporary paths but assumes every child shell has the
interactive PowerShell module search path. A service-launched Coordinator does not
make that caller-only environment contract available to validation children.

## 架构修复验收

- Detect PowerShell validation commands and add the standard Windows PowerShell module
  roots to a child-only environment when `PSModulePath` is absent or empty.
- Preserve any caller-provided module roots and de-duplicate the resulting path list.
- Leave non-PowerShell validation environments unchanged.
- Prove the installer acceptance runs from the managed finalizer environment.

## 禁止临时方案

- Do not change the installer test to avoid `Get-FileHash`.
- Do not require an interactive profile or caller-specific shell startup.
- Do not alter the global process environment or weaken finalizer validation.

## 修复结果与回传

- 根因：Coordinator maintenance finalizer launches PowerShell validation children without PSModulePath, so the service-launched shell cannot resolve the built-in Get-FileHash command used by the managed acceptance test.
- 架构修复：Finalizer now builds a per-child validation environment for PowerShell commands, preserving compatible caller module roots and adding standard Windows PowerShell/Core roots with deterministic de-duplication; non-PowerShell validation remains unchanged.
- 验证：Focused GitFinalizeTests PowerShell environment regression and bounded validation-diagnostic regression pass; Windows PowerShell 5.1 excludes the incompatible PowerShell 7 module root; direct codex-session-hook acceptance passes; the managed finalizer validation is replayed after this fix.
- 回传：Returned to Coordinator01 for a scoped maintenance fix covering finalizer validation environment construction and its regression test.
