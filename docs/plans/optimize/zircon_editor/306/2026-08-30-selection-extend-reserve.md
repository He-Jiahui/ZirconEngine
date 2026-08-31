# Editor306 Selection Extend Reserve

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-360-365-305-311-20260830-v4`

Current validation ticket: `1f220aa2ea7f48169ba15a8a15f0aec0`; source manifest hash:
`2c4796a46cbea9347f102a56a313ed404ee318c55c55d8956a64cde2f355568f`. The batch binds the
compile-time resource `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs`
at `a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.

## Scope

Selection extension now reserves the lower-bound capacity advertised by the incoming iterator before
inserting entities. Ordering, duplicate handling, primary selection, and generation semantics are
unchanged.

## Static Evidence

- The iterator size hint is consumed once before the existing insertion loop.
- Behavior and source-contract tests cover order, primary selection, and duplicate no-op behavior.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR306_SELECTION_EXTEND_RESERVE_BENCH_V1`. It
compares the previous incremental IndexSet growth with the reserved batch insertion over 512 items
and 2,000 repetitions per sample across 17 interleaved samples and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation,
exact performance timing, record finalization, manifest-only commit, push to `origin/main`, and the
one-shot WeCom result containing exact performance data, test result, commit SHA, and branch.

## Batched validation handoff (2026-08-30)

Editor306 is included in the six-group Runtime/Editor batch under request
`runtime-editor-360-365-305-311-20260830-v3`, ticket `f858e5d01c3e4da89baef9c1e2a276f9`, with
source manifest hash `8140739cbdd7f104a2d961b52a0f130b521a65a4afdcd5447afde63f030494af`. The
manifest also binds compile-time validation resource
`zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs` at
`a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`. Its
baseline now reproduces the pre-reserve extend behavior, including primary and generation updates,
so only allocation reservation differs; the 30% p95 gate remains. Cargo, performance, independent
review, commit, and WeCom remain coordinator-owned and pending.
