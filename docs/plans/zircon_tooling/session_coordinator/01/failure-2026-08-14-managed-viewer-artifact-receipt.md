---
handoff_kind: failure
status: open
created_at: 2026-08-14
summary_slug: managed-viewer-artifact-receipt
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/artifact_receipts.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/tests/test_artifact_receipts.py
  - tools/write_zircon_shader_pbr_build_provenance.ps1
  - tools/zircon_profile_shader_pbr_viewer.ps1
tests:
  - python -m unittest tools.session_coordinator.tests.test_artifact_receipts -v
  - Invoke-Pester -Script .\tools\tests\zircon_profile_shader_pbr_viewer.Tests.ps1 -PassThru
  - Select-String -LiteralPath .\tools\write_zircon_shader_pbr_build_provenance.ps1 -Pattern 'last_write_utc'
---

# Coordinator01: managed viewer artifact receipt is absent

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：M8.4--M8.5 current-source DX12 startup profile provenance.
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns immutable validation-copy identity, Cargo terminal evidence, and the schema/API that can bind a produced viewer artifact to that identity. Shader06 only consumes that evidence and must not invent an equivalent local receipt.

## 失败现象与复现证据

The Shader06 profile writer can fingerprint a supplied executable and compare its
`LastWriteTimeUtc` with current critical sources. This rejects plainly old
executables, and the focused Pester suite passed `8/8` on 2026-08-14, but file
time is mutable metadata: a stale executable can be copied or timestamp-touched
after a source edit, then be newly fingerprinted and recorded by the local
provenance writer. The passed validation ticket binds source files, not that
executable.

Static reproduction boundary:

```powershell
Select-String -LiteralPath .\tools\write_zircon_shader_pbr_build_provenance.ps1 -Pattern 'last_write_utc'
```

The observed implementation accepts based on local binary SHA-256, byte length,
and write time after it queries a terminal passed source-validation ticket. No
Coordinator API returns a receipt containing the viewer artifact path, SHA-256,
byte length, managed input-manifest hash, and producing terminal build identity.

Expected behavior: profiling acceptance must reject every binary without a
Coordinator-owned artifact receipt for the exact managed build. A locally
constructed provenance JSON, file copy, or timestamp change must never satisfy
that acceptance boundary.

## 最低共享层根因

`WorkspaceCopyService` computes and stores `input_manifest_hash` for an immutable
validation copy, and Coordinator validation records source manifests and terminal
process evidence. Neither the workspace-copy nor Cargo terminal contract records
the exact output artifact selected from the managed target directory. Therefore
no source-plan script can prove that an arbitrary local `zircon_shader_pbr_viewer`
binary was produced by the current managed source copy.

## 架构修复验收

- Coordinator01 emits a durable, queryable post-build viewer-artifact receipt
  only after the managed Cargo process exits successfully. It contains the exact
  job/validation identity, managed target-relative artifact path, SHA-256, byte
  length, producing command identity, and immutable input/source-manifest hash.
- The receipt creation verifies the artifact stays under that job's managed target
  root and hashes the file there; it does not accept a caller-supplied output path
  or locally supplied hash.
- Shader06 provenance consumes the receipt by ID, verifies the exact artifact
  fingerprint and manifest/ticket identity, and refuses missing, nonterminal,
  cross-job, cross-target, copied, or mismatched artifacts.
- Coordinator-focused tests cover a successful receipt, nonzero/no-artifact
  rejection, target-root escape rejection, hash mismatch after build, and
  immutable-manifest mismatch. Shader06 profile tests cover rejection of a
  timestamp-advanced stale binary without the matching receipt.
- Re-run the M8.5 managed build, five cold/five warm DX12 capture, WPR/GPU
  timing, screenshot, and RenderDoc acceptance gates using the receipt. Until
  then current-source profile data remains diagnostic only.

## 禁止临时方案

- Do not treat file timestamps, a local JSON record, Git revision, or a
  caller-supplied SHA-256 as proof that an executable came from the managed build.
- Do not add a Shader06-only Coordinator query bypass, compatibility alias,
  silent fallback, or a test-only receipt.
- Do not weaken the M8 current-source screenshot/profile acceptance boundary or
  relabel local diagnostic capture as integrated build evidence.

## 修复结果与回传

Coordinator-owned receipt issuance was implemented by
`eb52a1aadb68458f0ed7d02faf454ef0ac32a500`. A receipt now seals the artifact
under the managed target only after a successful terminal build and durably binds
its path, SHA-256, byte length, producing job/command, validation ticket, and
immutable copy input manifest. Receipt creation rejects target escape, nonzero or
missing output, post-build mutation, foreign Session access, and ticket sources
that are not a hash-identical or tombstoned subset of the materialized copy.

Source hashing occurs before the SQLite write transaction; the copy and ticket
signatures are revalidated inside the short insertion transaction. The complete
`tools.session_coordinator.tests.test_artifact_receipts` suite passed `13/13`, and
the durable fixing-plan evidence is recorded in
`2026-08-15-managed-viewer-artifact-receipt-forward-evidence.md`.

Open state: `Coordinator contract fixed / Shader06 acceptance pending`. The
Shader06 origin owner still owns the managed viewer build, five-cold/five-warm
capture, screenshot/RenderDoc gates, receipt consumption, and ordinary lifecycle
return. This record does not claim those product-side gates or close their origin
failure on Coordinator evidence alone.
