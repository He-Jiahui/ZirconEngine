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
plan_link_mode: child_record_only
related_code:
  - tools/build-editor.ps1
  - tools/common/WindowsPathResolver.psm1
  - tools/tests/build-editor.Tests.ps1
tests:
  - Invoke-Pester -Script tools/tests/build-editor.Tests.ps1 -PassThru
  - powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-editor.ps1 -OutputDirectory E:\cargo-targets\editor-debug-performance-20260929-a1b0df1a
---

# Performance01: editor bundle approved-root separator failure

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：build-editor approved artifact-root preflight
- 修复责任计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 交接原因：Performance01 owns the product build and profiling acceptance that first exposed the approved-root containment defect.

## 失败现象与复现证据

The current product profiling entry rejects both its generated `D:\ZirconBuilds` output and an explicit `E:\ZirconBuilds` output before Cargo starts. `tools/build-editor.ps1:130` appends the PowerShell literal `'\\'` to an already resolved root. PowerShell does not use backslash escaping, so this creates two separators; a valid `...\ZirconBuilds\child` can never start with `...\ZirconBuilds\\`.

The current Pester suite reproduces the shared cause: **15 total, 9 passed, 6 failed** in 80.54 seconds. Success publication, runtime-build cleanup, reparse rejection, existing-output preservation, missing-parent reporting, and relative-output resolution all fail before their intended branch. The two direct product invocations fail with the same `OutputDirectory must resolve below...` error and create no product bundle.

## 最低共享层根因

`tools/build-editor.ps1`, `tools/common/WindowsPathResolver.psm1`, and `tools/tests/build-editor.Tests.ps1` contain substantial foreign uncommitted work. Performance01 acquired then released a lease on the builder and did not overwrite that work. The current Session's maintenance authorization is limited to `docs/plans`, so the source edit remains for a source-authorized continuation.

The minimum code change is to append one separator (`'\'`) at `tools/build-editor.ps1:130`; do not weaken the resolved-path, lease, reparse-point, or no-overwrite checks. Existing Pester behavior tests already cover the regression, so no source-shape-only receipt is required.

## 架构修复验收

- All 17 current `tools/tests/build-editor.Tests.ps1` tests pass under Windows PowerShell.
- An explicit unique E-drive bundle reaches the managed validator, produces `zircon_editor.exe`, `zircon_runtime.dll`, and assets, and passes `--help` smoke.
- No artifact is written to C:; failure cleanup leaves no staging directory.
- Performance01 then runs the current MVP product with WPR/xperf and RenderDoc. Until that dynamic evidence exists, this handoff stays `open` and no graphics module moves to `review.md`.

## 禁止临时方案

- Do not switch the output to C:, use the stale 2026-08-10 executable, bypass the managed validator, or relax root containment.
- Do not report the 9 passing path-safety tests as a successful bundle build.
- Do not close this record on static inspection alone.

## 修复结果与回传

Open state at the 2026-08-15 repair checkpoint: source accepted a unique
`D:\ZirconBuilds` child through the product-staging acquire/release preflight,
and that checkpoint's artifact audit reported no unmanaged path. Its Windows
PowerShell Pester suite passed 17/17. The current build script instead requires
a physical `D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets` root.
No product Cargo build, bundle publication, smoke launch, WPR/xperf
capture, or RenderDoc evidence is claimed here, so the failure remains open for
Performance01's dynamic acceptance.

## 2026-09-19 successor intake (failure-roll-01a084c8-perf01-build-editor-root-r2)

The archived Performance01 successor scope was re-admitted after the current
coordinator index was checked. Ownership transfer preview fingerprint
`1fa93a8b88793f5a23ace4bba79c52096280324afa7b8055940be0ce888b5b15` was applied
for this failure record, `tools/build-editor.ps1`,
`tools/common/WindowsPathResolver.psm1`, and `tools/tests/build-editor.Tests.ps1`;
the session lease and baseline attribution were then acquired for exactly
those paths. The current source snapshots were unchanged at intake:

- `tools/build-editor.ps1`
  `f24a6013706e7aee1e6e2ec17a425dc6e5a1c5b5b3cb4e0cad301dd797eef89d`
- `tools/common/WindowsPathResolver.psm1`
  `27d315830c00ef91e8c5241a454f7036a96a2c85fa5a04077b0815f8a64e5a08`
- `tools/tests/build-editor.Tests.ps1`
  `bd895ece58ed411bcd76246850ee8b2b0090b86e5327a71e8d9fcc8c65a6bf95`

The existing repair remains the one-separator/approved-root implementation;
no source edit is being inferred from this intake. A coordinator-managed
PowerShell parse/contract check will verify the current resolver, approved
`D:\`, `E:\`, and `F:\` root guards, and the focused Pester suite remains
required. This is preparatory evidence only: the explicit E-drive product
bundle, validator/smoke, WPR/RenderDoc, independent review, fixed/return
artifacts, and failure closeout are still pending.

### Coordinator validation receipt

- Request: `failure-roll-01a084c8-perf01-build-editor-root-20260919-r1`
- Ticket: `95e908d8533740928ded3628ded0a04f`
- Source-manifest hash: `7bb65a9e2e3b607783afbcb5839279a46b7fda20fd464c209548ff8a2a607f7e`
- Admission: `queued`; execution kind `pending`; no command output or dynamic
  pass is claimed.
- The coordinator reported `validation_dependency_failed` for the open
  lower-chain records `plan-status-receipt-test-compile-debt`,
  `input-manager-bound-text-owner-budget`, `camera-table-render-extract-stale-map`,
  `render-graph-rust-2021-let-chain`, `runtime-package-asset-root-projection`,
  `postprocess-plugin-legacy-pass-order-drift`, and
  `kira-send-frame-capture-routing` (with their Runtime15/Runtime09/Render01/
  Render07/Plugins13/Plugins02 dependency paths). The ticket remains queued
  until those dependencies are resolved; this receipt is not reusable as a
  Pester, bundle, smoke, or performance acceptance.

### 2026-09-29 approved-root acceptance command correction

The original failing product command remains historical reproduction evidence:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/build-editor.ps1 -OutputDirectory E:\ZirconBuilds\editor-debug-performance-20260815
```

The tests frontmatter now names a distinct `E:\cargo-targets` child, preserving
the E-drive product acceptance while meeting the current physical artifact-root
policy. Its destination was absent at this metadata check. A rerun must use a
new absent child and seal that exact command in its managed ticket. No product
build, smoke, profiler capture, or visual acceptance is claimed here.
