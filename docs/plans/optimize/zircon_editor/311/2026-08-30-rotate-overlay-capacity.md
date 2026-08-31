# Editor311 Rotate Overlay Capacity

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-360-365-305-311-20260830-v4`

Current validation ticket: `1f220aa2ea7f48169ba15a8a15f0aec0`; source manifest hash:
`2c4796a46cbea9347f102a56a313ed404ee318c55c55d8956a64cde2f355568f`. The batch binds the
compile-time resource `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs`
at `a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.

## Scope

Rotate-handle overlay construction now reserves the exact four-element capacity used by the three
axis rings and center anchor, avoiding vector growth during every overlay rebuild.

## Static Evidence

- The overlay vector starts with capacity four.
- Existing element order and construction are unchanged.
- Behavior and source-contract tests cover all four emitted elements and the reservation.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR311_ROTATE_OVERLAY_CAPACITY_BENCH_V1`.
It compares zero-capacity growth with exact preallocation over 2,000,000 overlay builds per sample
across 17 interleaved samples and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation,
exact performance timing, record finalization, manifest-only commit, push to `origin/main`, and the
one-shot WeCom result containing exact performance data, test result, commit SHA, and branch.

## Batched validation handoff (2026-08-30)

Editor311 is included in the six-group Runtime/Editor batch under request
`runtime-editor-360-365-305-311-20260830-v2`, ticket `4e5bd6355a5f45eba4cba3e7d3ebe065`, with
source manifest hash `faee7e3ab3f4119b15a685e6a49e985dff861bf013c6078ecb87415f9b0ac57e`. The timed
vector build compares zero-capacity growth with capacity four on identical four-element overlays;
the 30% p95 gate remains. Cargo, performance, independent review, commit, and WeCom remain
coordinator-owned and pending.
