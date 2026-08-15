---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-15
resolved_at: 2026-08-15
summary_slug: build-editor-approved-root-separator
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/WindowsPathResolver.psm1
  - tools/build-editor.ps1
  - tools/mvp/MvpTestFixturePaths.psm1
  - tools/tests/build-editor.Tests.ps1
  - tools/tests/windows-path-resolver.Tests.ps1
  - tools/tests/mvp-test-fixture-paths.Tests.ps1
tests:
  - Invoke-Pester tools/tests/build-editor.Tests.ps1
  - Invoke-Pester tools/tests/windows-path-resolver.Tests.ps1
  - Invoke-Pester tools/tests/mvp-test-fixture-paths.Tests.ps1
---

# Coordinator01: build-editor approved-root separator repair

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：UI12 managed product-build preflight
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the managed Windows product-build wrapper and approved artifact-root boundary.

## 失败现象与复现证据

UI12 twice invoked the managed product-build preflight with an output below
`D:\ZirconBuilds`. Both attempts failed closed before Cargo and the wrapper removed its
temporary directories. The reported output for one attempt was
`D:\ZirconBuilds\editor-ui12-aa-20260815-232251`; OS inspection found no Cargo or
rustc process.

## 最低共享层根因

The containment expression appended the PowerShell single-quoted string `'\\'` to
the approved operational root. PowerShell does not use backslash as a string escape,
so that value contains two separators while a normal operational child path contains
one.

## 架构修复验收

`Resolve-BundleOutputDirectory` now appends
`[System.IO.Path]::DirectorySeparatorChar` when establishing the root-bound child
prefix. This produces exactly one separator while retaining the junction-aware
operational path, ordinal case-insensitive comparison, and the D/E/F approved-root
boundary.

The complete six-path Tooling stack is sealed together because the current
`build-editor` script depends on the Windows path resolver and the managed fixture
root helper. No Editor or Runtime product source is part of the repair.

## 禁止临时方案

- Do not weaken containment to textual prefix matching without a separator.
- Do not permit C-drive or arbitrary caller-provided roots.
- Do not edit Editor or Runtime product sources or use raw Cargo for this repair.

## 修复结果与回传

- 根因：The approved-root prefix contained two backslashes and rejected every normal child path.
- 架构修复：Use the platform directory-separator character while preserving the junction-aware operational identity and D/E/F root boundary.
- 验证：RED passed 10/15 with five containment failures. GREEN
  `build-editor.Tests.ps1` passes 15/15 and
  `windows-path-resolver.Tests.ps1` passes 6/6. The manual-assertion
  `mvp-test-fixture-paths.Tests.ps1` contract exits 0. The earlier 14/15 result was
  reproduced as artifact governance deleting its unregistered live fixture. After
  schema 64 was committed and hot-loaded, the helper acquired and released formal
  fixture leases and the finalizer-equivalent complete run passed 15/15 in 250.88s.
- 回传：After the scoped maintenance finalize and daemon rollover, UI12 may use the managed `tools/build-editor.ps1` path; this record does not authorize raw Cargo.
