---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-15
summary_slug: validate-matrix-nonzero-cleanup-outcome
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
tests:
  - Invoke-Pester validate-matrix.Tests.ps1 -TestName 'Coordinator pre-start failure cleanup'
  - Invoke-Pester validate-matrix.Tests.ps1 -TestName 'Validate matrix managed Cargo environment policy'
  - Invoke-Pester validate-matrix.Tests.ps1 -TestName 'Validate matrix Windows PowerShell compatibility'
resolved_at: 2026-08-15
---


# Coordinator01: managed validation cleanup masks a nonzero Cargo stage

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：UI12 reported current-source Editor managed validation cleanup failure
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the managed validation wrapper's finish, release, and result-projection boundary.

## 失败现象与复现证据

- Managed Job `82cc57e88efa45c591ab83e6c685d206` reached the real `cargo test -p zircon_editor --lib requested_host_window_size_stays_in_physical_pixels` process and returned a nonzero compile result with current Editor diagnostics.
- The Cargo step was recorded in `$Results` instead of thrown as `$primaryFailure`. When the coordinator became temporarily unavailable during finish/release, the `finally` block threw the cleanup exception and replaced the already-known nonzero validation outcome.
- The schema-63 successor reconciled the job to terminal `failed`, retained its managed pool/cache, and reported no live process tree. The 263 Editor diagnostics remain product current-source debt and are not part of this Tooling fix.

## 最低共享层根因

`Invoke-ValidateMatrixMain` distinguished only thrown primary failures from successful execution when arbitrating cleanup exceptions. `Invoke-Step` intentionally records nonzero Cargo exits without throwing, so a failed validation stage left `$primaryFailure` null and was incorrectly treated like a successful run during cleanup.

## 架构修复验收

- Compute an explicit `hasFailedStep` identity from the recorded validation results and use it for both the durable Cargo exit code and cleanup arbitration.
- If a validation stage is already nonzero, coordinator or managed-environment cleanup failure is emitted as an additional warning; the wrapper continues to its summary and returns the stage failure code.
- A thrown primary validation error remains primary and cleanup failure remains secondary diagnostic evidence.
- If all validation stages succeeded and no primary error exists, cleanup failure is still thrown; the repair must not hide a genuinely incomplete finish/release.
- Pre-start jobs remain unreleased until `cargo start`, and ordinary successful started jobs still finish then release in order.

## 禁止临时方案

- Do not convert nonzero Cargo exits into generic exceptions, return success after a failed stage, or suppress cleanup diagnostics.
- Do not delete the retained target pool/cache, manually rewrite Job `82cc57e88efa45c591ab83e6c685d206`, or absorb Editor compile diagnostics into Tooling scope.
- Do not run raw or replacement product Cargo as part of this fix.

## 修复结果与回传

- 根因：Recorded nonzero validation stages were absent from cleanup-error precedence, so a transient coordinator cleanup exception replaced the real Cargo outcome.
- 架构修复：A single cleanup-outcome resolver now preserves thrown primary errors and recorded failed stages while keeping otherwise-successful cleanup failures fatal.
- 验证：The focused cleanup lifecycle Describe passes 5/5, including nonzero-stage/offline cleanup and successful-stage/offline cleanup boundaries; managed Cargo environment policy passes 3/3; Windows PowerShell/ASCII compatibility passes 1/1. The full wrapper suite was attempted once but exceeded its 300-second Windows I/O limit without a terminal result, so it is not claimed as green.
- 回传：UI12 may retry through the committed and hot-loaded managed wrapper after the shared Cargo lane and Coordinator finalizer are explicitly released; this repair does not authorize a raw Cargo replay.
