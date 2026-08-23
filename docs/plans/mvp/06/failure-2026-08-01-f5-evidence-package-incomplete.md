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
