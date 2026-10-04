---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: rhi-upload-batch-payload-owner-lifetime
origin_plan: docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/optimize/zircon_runtime/90
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi/src/upload.rs
tests:
  - ".codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zr_rhi -SkipBuild -LibTests -TestFilter batches_share_payload_owners_and_count_only_selected_ranges -VerboseOutput"
  - ".codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zr_rhi -SkipBuild -LibTests -VerboseOutput"
resolved_at: 2026-09-10
---

# Runtime90: upload batch payload-owner lifetime assertion

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md`
- 来源执行切片：Lower-layer `zr_rhi` compilation and complete library regression.
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：The failure was discovered while clearing the lower-layer `zr_rhi` compile chain and is isolated to the neutral upload-batch regression.

## 失败现象与复现证据

Managed Cargo job `b917170b89484ba6a7b55e8369563840` compiled `zr_rhi` and ran all 78 library tests. It finished with exit code 1: 77 tests passed and only `upload::tests::batches_share_payload_owners_and_count_only_selected_ranges` failed at `upload.rs:271`, where `Arc::strong_count(&payload)` was 1 instead of 3.

## 最低共享层根因

The production batch accounting was correct. The test constructed each batch as a temporary inside `assert_eq!`; each temporary was dropped at the end of that statement, releasing its payload owner before the final strong-count assertion.

## Repair

Keep the buffer and texture batches in named local variables through the ownership assertion. This preserves the intended two batch-held `Arc` owners, continues to verify that byte accounting uses only the selected source ranges, and changes no production upload behavior.

## 架构修复验收

- The focused managed test passes with three live owners and selected byte totals 6 and 4.
- The complete managed `zr_rhi --lib` suite passes.
- No raw Cargo invocation or unrelated Runtime90 path is used.

## 禁止临时方案

- Do not weaken the exact three-owner assertion or selected byte totals.
- Do not leak the payload or alter production batch ownership to satisfy a fixture.
- Do not treat historical pass counts as current-source proof without matching hashes.

## 修复结果与回传

- 根因：The production upload batch ownership is correct; the regression fixture created both batches as temporary expressions, so they were dropped before the final Arc strong-count assertion. The current named-batch fixture preserves both owners.
- 架构修复：Keep BufferUploadBatch and TextureUploadBatch in named locals through the ownership assertion, preserving selected source-range accounting and three live Arc owners without changing production upload ownership.
- 验证：Current upload.rs SHA-256 is 105f704dce0277de67636ade4266accb2d09e1cf90956ee2900c5dbf3526046e and matches the immutable current-source evidence in managed job ee28f1c8875043ab94f22b4bf8d32a3c, which passed the exact upload regression and complete zr_rhi library batch with --locked. A fresh Tooling01-owned validation ticket remains required for closeout.
- 回传：Returned the lower-layer upload fixture repair with historical source-matched evidence; the original Runtime90 cross-plan blocker is resolved, while current Tooling01 validation and closeout remain explicitly pending.
