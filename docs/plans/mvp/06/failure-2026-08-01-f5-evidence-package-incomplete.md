---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: f5-evidence-package-incomplete
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/mvp/06-f5-acceptance-wave.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/mvp/06
plan_link_mode: child_record_only
related_code:
  - tools/mvp/Stage-MvpProducts.ps1
  - tools/mvp/Invoke-MvpAcceptance.ps1
  - .github/workflows/mvp-editor-windows.yml
tests:
  - tools/tests/mvp-staging.Tests.ps1
  - tools/tests/mvp-acceptance.Tests.ps1
  - tools/tests/mvp_editor_windows_workflow.Tests.ps1
---

# MVP06: RequireF5Evidence accepts an incomplete evidence package

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：2026-08-01 current-source plan/code review convergence
- 修复责任计划：`docs/plans/mvp/06-f5-acceptance-wave.md`
- 交接原因：MVP06 owns the acceptance schema, Stage/Invoke boundary and F5 completion claim; Performance01 can identify the false-positive gate but must not redefine acceptance evidence.

## 失败现象与复现证据

- The F5 plan requires `build/profile-contract-summary.json`, `build/workspace-summary.json` and absolute start/end time for every product process.
- `Stage-MvpProducts.ps1` executes product processes and records elapsed duration, but its published evidence does not retain each process's absolute start/end pair.
- `Invoke-MvpAcceptance.ps1` archives staging/startup/project/log/capture/automation evidence, but does not require or copy the two build summaries into the final evidence root.
- The Windows workflow invokes `-RequireF5Evidence` and uploads the resulting package, so that switch can succeed while the package still violates the plan's own F5 schema.

## 最低共享层根因

Execution and evidence validation have separate owners, but the acceptance schema does not require all upstream build/profile and process-timing evidence at the final publication boundary. A successful product smoke is therefore being conflated with a complete F5 acceptance package.

## 架构修复验收

- Keep `Stage-MvpProducts.ps1` as the sole product/process executor and `Invoke-MvpAcceptance.ps1` as the immutable evidence validator/archiver.
- Give Invoke explicit build/profile summary inputs, verify their source fingerprint and hashes, and copy them under the canonical `build/` paths.
- Record absolute start/end time plus exit code for every Stage-owned child process; elapsed duration may remain as derived telemetry.
- Make `RequireF5Evidence` fail if either build summary or any required process timing field is absent, malformed or bound to another source/project identity.
- Add focused Pester failures for missing/mismatched summaries and missing process time; keep fake child/timeout/order coverage under the Stage test owner.
- Run the corrected workflow in a clean coordinator validation copy and inspect the uploaded bounded artifact before fixed return.

## 禁止临时方案

- 不得把开关重命名或修改计划以降低 F5 证据定义，除非用户明确作出产品验收降级决定。
- 不得由 Invoke 重跑产品或从 Cargo target 猜 build summaries。
- 不得用 elapsed milliseconds 代替绝对开始/结束时间，或用 workflow step 时间代替每个 process 时间。
- 不得仅让 Pester 字符串检查通过而不检查真实上传包结构。

## 修复结果与回传

2026-08-01 current source 已完成实现收敛：Stage 为 project creation、baseline/authoring/reopen automation 与五个 product run 记录 `started_at_utc`/`ended_at_utc`/`exit_code`；Invoke 在 `RequireF5Evidence` 下强制显式 profile/workspace summary，校验 kind/source fingerprint 并把 summaries 与 process timing 写入 schema v2 evidence package。旧负向测试仍匹配 `F5 product process 1`，而 canonical validator 标签已是 `F5 runtime product attempt 1`；已只修测试标签，不放宽 validator。

本地当前源验证：`pwsh -NoProfile -File tools/tests/mvp-staging.Tests.ps1` 输出 `MVP staging contract passed`；`pwsh -NoProfile -File tools/tests/mvp-acceptance.Tests.ps1` 输出 `MVP acceptance manifest contract passed`；`pwsh -NoProfile -File tools/tests/mvp_editor_windows_workflow.Tests.ps1` 输出 `MVP Windows workflow contract passed`。这些门证明 schema、负向篡改与原子 evidence package 行为，但尚未替代 clean coordinator validation copy 中的 corrected workflow 和真实上传 artifact 检查。

2026-08-02 current-source review 发现上述 acceptance 通过记录已失效：PNG decoded-pixel SHA 支撑改为嵌入 C# 后，`Add-Type -ReferencedAssemblies` 只显式传入 Drawing assemblies，导致 `SHA256` 即使已 import namespace 仍无法解析。当前实现把 `[Security.Cryptography.SHA256].Assembly.Location` 纳入同一显式引用集合；fresh 串行 `pwsh -NoProfile -File tools/tests/mvp-acceptance.Tests.ps1` 在 148 秒内通过并输出 `MVP acceptance manifest contract passed`，覆盖真实 Add-Type 编译、decoded-pixel hash、runtime diagnostics counters、authoring/reopen identity 与负向篡改矩阵。该修复恢复 focused gate，但不替代 clean validation copy 和真实 workflow artifact 检查。

2026-08-02 current-source hardening further makes the immutable evidence boundary explicit. `Invoke-MvpAcceptance.ps1` now publishes a no-follow staging snapshot before validation and holds its identity through a snapshot lease. The lease pins the root and staged entries, creates exclusive delete-on-close markers for the root and each visited directory, revalidates root and child identities during traversal, and excludes those held markers from the archived evidence tree. `tools/tests/mvp-acceptance-staging-snapshot.Tests.ps1` covers publication, replacement/reparse rejection, held-marker cleanup protection, and marker exclusion; it remains a focused source contract rather than workflow evidence.

Open state: `MVP06 implementation and focused PowerShell gates are green; fixed return still requires the corrected workflow in a clean coordinator validation copy and inspection of its uploaded bounded artifact.`

## 2026-08-03 product-validation continuation

The requested Windows product run was attempted before acceptance publication. The only
permitted build path was:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_app -NoDefaultFeatures -Features target-client -SkipTest
```

The current source tree did not produce `zircon_runtime.exe` or `zircon_editor.exe`. A targeted
search of the repository and the managed target directory found no reusable executable. After
forward fixes for three immediate compile blockers in `zircon_runtime_interface` and `zr_rhi_wgpu`,
the same controlled client build reached `zircon_runtime` and failed with 164 source errors across
independent rendering, resource, capture, and borrow/const-contract modules. Representative
remaining errors include missing mesh-pipeline submodule re-exports, invalid render capture imports,
non-Send WGPU error-scope futures retained by `MeshPipelineCache`, and Rust 2021 const/borrow
violations. Product staging, desktop launch, and PNG capture were therefore not executed; no
product screenshot is claimed by this continuation.

Forward direction: restore the missing module boundaries and resolve the independent runtime
compile errors until the controlled client build produces fresh runtime and editor-host binaries,
then rerun `Stage-MvpProducts.ps1` from a durable evidence root and publish with
`Invoke-MvpAcceptance.ps1`. This is a current-source product-build blocker, not an acceptance
waiver or rollback request.

The focused evidence-boundary regression
`tools/tests/mvp-acceptance-staging-snapshot.Tests.ps1` completed with exit code 0 under the
default warning policy after this continuation. Its expected warning capture confirms that
identity-mismatched cleanup skips a replacement root; this source-level result does not claim a
product run or screenshot.

2026-08-03 forward repair update: the same repository-controlled Windows client build completed
after the first two runtime-support repair batches and reduced the compiler error count from 164
to 110. The completed repairs cover missing facade re-exports, retained WGPU capture visibility,
temporary resource-manager borrows, stable `const fn` clamps, text-cache/module visibility, and
several local ownership defects. The build still fails in current source, so it produced no fresh
runtime/editor executable; product staging, desktop launch, and PNG capture remain unexecuted.
The next forward slice is the remaining restricted re-exports, script host lifetime contracts,
readback ownership, and the gameplay-host world-operation inference root cause.

2026-08-03 forward repair update: a third runtime batch reduced the same controlled build's
observed source-error count from 110 to 68 before the next compiler rerun. It repairs additional
facade boundaries, IBL dispatch context ownership, GPU timestamp-safe graph recording, pipeline
manifest adaptation, and targeted type/borrow failures. The graph recorder now excludes timestamp
and IBL-owner stages from its parallel path instead of capturing non-Send WGPU state in a worker
closure. The remaining known direct compile root in the renderer's capture admission is a missing
boolean argument, but its source file is currently held by another writer and was deliberately not
overwritten.

The controlled validator also exposed a Windows PowerShell compatibility defect before Cargo
started: `validate-matrix.ps1` required `System.Text.Json.JsonDocument`, unavailable in the active
host although the coordinator response was a single JSON object. The strict parser now uses
`ConvertFrom-Json` and retains the object-only contract. A subsequent `-DryRun` exceeded the
non-blocking command window, so this parser repair and the 68-error source state still require one
completed managed build result. No fresh runtime/editor executable exists from this continuation;
staging, desktop launch, and PNG screenshot evidence remain explicitly unexecuted.

2026-08-03 product-validation repair update: consecutive completed managed client builds exposed
three later compile roots. The runtime fallback presenter retained a captured frame while it called
back into `RuntimeEntryApp` (`E0502`); it now releases that frame before the first-frame completion
path re-borrows the app. The UI image-resource call site was concurrently advanced to include
`payload.resource_generation`, matching the generation-aware resource-table lookup. Finally, the
WGPU image cache required the resource table to discard transferred CPU payloads at the end of a
present; `UiSurfaceImageResourceTable::clear` now owns that operation, with a focused
multiple-generation regression test. The next controlled build request was rejected before Cargo
started because its coordinator admission checkpoint was stale. Per the coordination rule, this
continuation did not poll, recover, or bypass the coordinator with a direct Cargo command.

The self-executing snapshot contract
`tools/tests/mvp-acceptance-staging-snapshot.Tests.ps1` passed with exit code 0. An attempt to run
the broader `tools/tests/mvp-staging.Tests.ps1` exceeded the local 60-second command window, so it
is deliberately not recorded as passing or failing. There is still no fresh product executable,
desktop launch, staged project, or PNG screenshot from this source state.

## 2026-09-18 failure rolling repair successor r1

Session `failure-roll-01a084c8-mvp06-f5-evidence-r1` re-claimed the archived failure record and
the Stage/Invoke PNG evidence owners. A fresh local run exposed a current-runtime compile gap
that the earlier SHA-256-only repair did not cover: `mvp-acceptance.Tests.ps1` failed in
`Invoke-MvpAcceptanceTestDriver.ps1` with CS0012 because `IImage` requires
`System.Private.Windows.GdiPlus` (and its `System.Private.Windows.Core` dependency) in the
`Add-Type -ReferencedAssemblies` set. The lowest shared fix now resolves those optional split
assemblies through `[Reflection.Assembly]::Load` in both `Stage-MvpProducts.ps1` and
`Invoke-MvpAcceptance.ps1`, while retaining compatibility with hosts that do not expose them.
The two source-contract tests assert the assembly names and loader call.

Current source hashes after that repair are:

- `tools/mvp/Stage-MvpProducts.ps1`: `cc0d452fe4190e6ea654ab7083ef9122fda92680f5a67318c49e7576c3071b9e`
- `tools/mvp/Invoke-MvpAcceptance.ps1`: `2354e3071424de4c44437e9d1dcfdccd2c9de29d6d856e7e30a7c6c91d6a1bc3`
- `tools/tests/mvp-staging.Tests.ps1`: `63b674afc6e54a33668f1dbffd8eaa478b38154241ddcf1bc1753a0e280a2989`
- `tools/tests/mvp-acceptance.Tests.ps1`: `7c98ed8657839acbaaafba93149bc8100348b589691097b78bbb5bf71dd690f7`

Local evidence matching those hashes: the staging contract passed; the Windows drawing
reference probe compiled the same `System.Drawing` helper successfully; scoped `git diff --check`
passed. The full acceptance script was bounded locally and interrupted after the first
environmental run exceeded the interactive window, so no local acceptance pass is claimed.

Two immutable Windows PowerShell tickets are queued against this exact manifest and session:

- `e137638de01449f298b1fe56edbdbf37` / request `failure-roll-01a084c8-mvp06-contracts-20260918-r2`
  runs the Stage, workflow, and staging-snapshot contracts.
- `258b2002ccf24ae5bedb4cd1d1bf52b2` / request `failure-roll-01a084c8-mvp06-acceptance-20260918-r1`
  runs the corrected acceptance-manifest contract.

These tickets are pending terminal receipts. Product staging, corrected workflow execution,
uploaded artifact inspection, independent review, fixed return, and closeout SHA remain open;
this failure is not returned or marked fixed.

## 2026-09-18 managed ticket terminal evidence and forward scope correction

The two tickets reached terminal `failed` states on the immutable copies; neither is a
source-contract GREEN:

- `e137638de01449f298b1fe56edbdbf37` (job `adc5cb75965c4cb6bb0237515b3bfe5b`, run with
  exit code `1`) stopped before the three scripts ran because the copied
  `MvpTestFixturePaths.psm1` could not acquire a fixture: the copied repository had no
  coordinator runtime descriptor (`offline` / `descriptor_absent`). Its coordinator-category
  stderr is retained as an environment blocker, not test evidence.
- `258b2002ccf24ae5bedb4cd1d1bf52b2` (job `88a9029b48424aafb3581552d63cb000`, run with exit
  code `1`) reached `mvp-acceptance.Tests.ps1` but its immutable source copy omitted
  `.github/workflows/mvp-editor-windows.yml`, which the test reads directly at line 74. The
  failure is a validation-manifest closure defect, not an acceptance assertion result.

The workflow file is clean at the current HEAD and had no conflicting source owner. It was
therefore transferred into this Session's attribution with coordinator fingerprint
`0f20f58469c1ea75a4efb58a86d06bb77c0606a40041c5cc73f848742a172e53` (no bytes changed), and
its current SHA-256 is
`fb19d46eb65010ae7496c78fc3f5debbbe83db6a4607c5dc563cddf1554cb577`. A forward ticket must
include this path in the sealed manifest. The focused staging contracts still require a
coordinator-aware validation execution environment; no ticket is reused or counted as passed.

Product staging, a corrected immutable contract run, full acceptance, uploaded artifact
inspection, independent review, fixed return, and closeout SHA remain open.

The forward acceptance ticket `b9a8b9ac44df461781fe6178600ed2e0` (job
`271bb70ed8c14a47a707b481edc6c805`) sealed the expanded eight-path manifest, including the
workflow at the hash above, but also failed before acceptance assertions: the staging-snapshot
contract's `New-MvpTestFixtureRoot` could not reach the coordinator from the immutable copy
(`offline` / `descriptor_absent`). Its terminal stderr and coordinator-category classification
are retained; this ticket is not reusable as a pass. The validation environment must provide a
repository-bound coordinator endpoint to copy-executed fixture helpers, or the contract batch
must remain explicitly blocked rather than bypassing fixture leases.

## 2026-09-18 validation-environment handoff

The expanded ticket is reconciled as a terminal coordinator failure, not a source-test result.
The immutable-copy runner does not expose the live coordinator descriptor required by
`MvpTestFixturePaths.psm1`; adding a descriptor to the copy or running the fixture helper from the
mutable checkout would invalidate the managed-validation boundary. The three terminal ticket
identities (`e137638de01449f298b1fe56edbdbf37`, `258b2002ccf24ae5bedb4cd1d1bf52b2`, and
`b9a8b9ac44df461781fe6178600ed2e0`) and their coordinator-category stderr remain the complete
evidence for this handoff.

The successor session is suspended in `waiting_validation` after recording the current failure
document hash and attribution. Its path leases are released so an unrelated owner is not blocked;
the lifecycle remains open and must be reactivated only after the coordinator supplies a
repository-bound fixture endpoint and a newly sealed manifest. No fixed document, return record,
review, product staging, or closeout commit is claimed.

## 2026-09-25 current-source rolling reconciliation

- Current hashes remain aligned with the 2026-09-18 source repair:
  `tools/mvp/Stage-MvpProducts.ps1` =
  `cc0d452fe4190e6ea654ab7083ef9122fda92680f5a67318c49e7576c3071b9e`,
  `tools/mvp/Invoke-MvpAcceptance.ps1` =
  `2354e3071424de4c44437e9d1dcfdccd2c9de29d6d856e7e30a7c6c91d6a1bc3`,
  `tools/tests/mvp-staging.Tests.ps1` =
  `63b674afc6e54a33668f1dbffd8eaa478b38154241ddcf1bc1753a0e280a2989`,
  `tools/tests/mvp-acceptance.Tests.ps1` =
  `7c98ed8657839acbaaafba93149bc8100348b589691097b78bbb5bf71dd690f7`,
  `tools/tests/mvp_editor_windows_workflow.Tests.ps1` =
  `1c00da4397aebee89c7417662498e314477573247f73aa89a4c1234d172708af`, and
  `.github/workflows/mvp-editor-windows.yml` =
  `fb19d46eb65010ae7496c78fc3f5debbbe83db6a4607c5dc563cddf1554cb577`.
- The four `tools/mvp`/Pester paths are dirty foreign overlays in the shared
  checkout; the workflow is clean. This doc-only successor does not claim or
  absorb those source owners. Existing local Pester receipts and ticket failures
  remain historical evidence, not a fresh product/workflow acceptance result.
- The corrected workflow must still run in a coordinator-bound immutable copy,
  publish a bounded evidence package, and pass artifact inspection. Product
  staging, external clean-worktree admission, independent C/I/M review, fixed
  return, and closeout remain pending; the failure stays `open`.

## 2026-09-25 independent static review receipt

- Reviewer `/root/review_editor03_gizmo_private` re-read snapshot 3800 at
  document SHA-256 `733b53bfa3b20fab8ae3f2c410340c0c470469fd6267b76d78ce6965a9f95ee6`.
  All six current hashes match. The four Stage/Invoke/Pester files are dirty
  foreign overlays, while the workflow is clean, matching the recorded
  provenance.
- Review result: Critical/Important/Moderate = `0/0/0`. Stage records
  `started_at_utc`/`ended_at_utc`/`exit_code`; Invoke's `RequireF5Evidence`
  enforces explicit summaries, source fingerprint, timing, and schema v2
  publication. Historical Pester/ticket receipts remain non-passes; no current
  clean coordinator workflow/artifact or product acceptance was claimed and no
  validation command was run.
