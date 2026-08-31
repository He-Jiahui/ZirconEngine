# Editor312 Scale Overlay Capacity

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-366-312-20260830-v5`

Current validation ticket: `72668f71e4fc4a4eb285f773b7688822`; source manifest hash:
`e6799e3672bdd574ec1f5a7d04ea70deb4117a53be8589193d0b3bd19b113d57`. The batch binds the
compile-time resource `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs`
at `a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.

## Scope

Scale-handle overlay construction now reserves the exact four-element capacity used by the three
axis handles and center anchor, avoiding vector growth during each overlay rebuild.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR312_SCALE_OVERLAY_CAPACITY_BENCH_V1`.
It compares zero-capacity growth with exact preallocation over 2,000,000 overlay builds per sample
across 17 interleaved samples and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation,
record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom publication.

## Batched validation handoff (2026-08-30)

Editor312 is included in the corrected `bn` Runtime/Editor batch under request
`runtime-editor-366-312-20260830-v2`, ticket `2da65704d2424ea6863ca28383a6a48e`, with source
manifest hash `084859d4afc3c5d7be6956d3d50c802e772b6da590cafd48e8ada963a8e45dba`. The batch keeps
the exact four-element capacity comparison and 30% p95 gate. Cargo, performance, review, commit,
and WeCom remain coordinator-owned and pending.
