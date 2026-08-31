# Runtime362 Full Rebuild Stage Skip

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-360-365-305-311-20260830-v4`

Current validation ticket: `1f220aa2ea7f48169ba15a8a15f0aec0`; source manifest hash:
`2c4796a46cbea9347f102a56a313ed404ee318c55c55d8956a64cde2f355568f`. The batch binds the
compile-time resource `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs`
at `a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.

## Scope

Invalidation classification now records the document full-rebuild condition once and skips redundant
incremental stage insertions when that full stage set is already present. Change-kind reporting is
unchanged; only duplicate BTreeSet insertion work is removed.

## Static Evidence

- Full rebuild detection is cached in `full_rebuild`.
- Incremental stage arrays are guarded by `extend_incremental_stages`.
- Behavior and source-contract tests cover the complete eight-stage set and guarded insertion.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME362_UI_INVALIDATION_FULL_REBUILD_BENCH_V1`. It
compares redundant incremental stage insertion with the guarded full-rebuild path over 2,000,000
rebuilds per sample across 17 interleaved samples and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation,
exact performance timing, record finalization, manifest-only commit, push to `origin/main`, and the
one-shot WeCom result containing exact performance data, test result, commit SHA, and branch.

## Batched validation handoff (2026-08-30)

Runtime362 is included in the six-group Runtime/Editor batch under request
`runtime-editor-360-365-305-311-20260830-v3`, ticket `f858e5d01c3e4da89baef9c1e2a276f9`, with
source manifest hash `8140739cbdd7f104a2d961b52a0f130b521a65a4afdcd5447afde63f030494af`. The
manifest also binds compile-time validation resource
`zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs` at
`a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`. The
full-rebuild flag is opaque during timing so the guarded branch is measured as a runtime decision;
the 30% p95 gate remains. Cargo, performance, independent review, commit, and WeCom remain
coordinator-owned and pending.
