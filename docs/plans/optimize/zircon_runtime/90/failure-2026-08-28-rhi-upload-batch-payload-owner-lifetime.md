---
handoff_kind: failure
status: open
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
---

# Runtime90: upload batch payload-owner lifetime assertion

## Source executor

- Origin plan: `docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md`
- Fix owner: `docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- Handoff reason: the failure was discovered while clearing the lower-layer `zr_rhi` compile chain and is isolated to the neutral upload-batch regression.

## RED evidence

Managed Cargo job `b917170b89484ba6a7b55e8369563840` compiled `zr_rhi` and ran all 78 library tests. It finished with exit code 1: 77 tests passed and only `upload::tests::batches_share_payload_owners_and_count_only_selected_ranges` failed at `upload.rs:271`, where `Arc::strong_count(&payload)` was 1 instead of 3.

The production batch accounting was correct. The test constructed each batch as a temporary inside `assert_eq!`; each temporary was dropped at the end of that statement, releasing its payload owner before the final strong-count assertion.

## Repair

Keep the buffer and texture batches in named local variables through the ownership assertion. This preserves the intended two batch-held `Arc` owners, continues to verify that byte accounting uses only the selected source ranges, and changes no production upload behavior.

## Acceptance

- The focused managed test passes with three live owners and selected byte totals 6 and 4.
- The complete managed `zr_rhi --lib` suite passes.
- No raw Cargo invocation or unrelated Runtime90 path is used.

## Result

Managed job `c95a25120f4d4c75a7a206946aaa898d` passed the exact upload-batch
regression and released with exit code 0. Managed job
`b5522b23945e4c70837b8dacb18b145c` then passed all 78 `zr_rhi --lib` tests and
released with exit code 0. The selected byte totals remain 6 and 4 while the original
payload plus both live batches hold exactly three `Arc` owners.

Open state: `repair and managed validation green / Runtime90 atomic integration pending`.
