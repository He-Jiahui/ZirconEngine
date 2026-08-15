---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-15
summary_slug: build-editor-approved-root-separator
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/performance/01
related_code:
  - tools/build-editor.ps1
  - tools/WindowsPathResolver.psm1
  - tools/tests/build-editor.Tests.ps1
tests:
  - Invoke-Pester -Script tools/tests/build-editor.Tests.ps1 -PassThru
  - powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-editor.ps1 -OutputDirectory E:\ZirconBuilds\editor-debug-performance-20260815
---

# Performance01: editor bundle approved-root separator failure

## Failure evidence

The current product profiling entry rejects both its generated `D:\ZirconBuilds` output and an explicit `E:\ZirconBuilds` output before Cargo starts. `tools/build-editor.ps1:130` appends the PowerShell literal `'\\'` to an already resolved root. PowerShell does not use backslash escaping, so this creates two separators; a valid `...\ZirconBuilds\child` can never start with `...\ZirconBuilds\\`.

The current Pester suite reproduces the shared cause: **15 total, 9 passed, 6 failed** in 80.54 seconds. Success publication, runtime-build cleanup, reparse rejection, existing-output preservation, missing-parent reporting, and relative-output resolution all fail before their intended branch. The two direct product invocations fail with the same `OutputDirectory must resolve below...` error and create no product bundle.

## Ownership and minimum fix

`tools/build-editor.ps1`, `tools/WindowsPathResolver.psm1`, and `tools/tests/build-editor.Tests.ps1` contain substantial foreign uncommitted work. Performance01 acquired then released a lease on the builder and did not overwrite that work. The current Session's maintenance authorization is limited to `docs/plans`, so the source edit remains for a source-authorized continuation.

The minimum code change is to append one separator (`'\'`) at `tools/build-editor.ps1:130`; do not weaken the resolved-path, lease, reparse-point, or no-overwrite checks. Existing Pester behavior tests already cover the regression, so no source-shape-only receipt is required.

## Acceptance

- All 15 current `tools/tests/build-editor.Tests.ps1` tests pass under Windows PowerShell.
- An explicit unique E-drive bundle reaches the managed validator, produces `zircon_editor.exe`, `zircon_runtime.dll`, and assets, and passes `--help` smoke.
- No artifact is written to C:; failure cleanup leaves no staging directory.
- Performance01 then runs the current MVP product with WPR/xperf and RenderDoc. Until that dynamic evidence exists, this handoff stays `open` and no graphics module moves to `review.md`.

## Prohibited shortcuts

- Do not switch the output to C:, use the stale 2026-08-10 executable, bypass the managed validator, or relax root containment.
- Do not report the 9 passing path-safety tests as a successful bundle build.
- Do not close this record on static inspection alone.
