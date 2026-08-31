# Editor313 Listener Projection Capacity v2

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-367-313-20260830-v6`

Current validation ticket: `a89c4f5a9afa4d17ad8bd078cb3de74c`; source manifest hash:
`e343a4d8fe5ab0e645dd76587aa8128560180362055d98248962cb006bd62571`. The batch binds the
compile-time resource `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs`
at `a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.

## Scope

Listener descriptor projection now reserves the exact listener count before mapping descriptors.
The JSON field order, descriptor values, and empty-list behavior remain unchanged while large
listener lists avoid growth reallocations.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR313_LISTENER_PROJECTION_CAPACITY_BENCH_V1`.
It compares an unreserved projection with exact listener-count reservation over 2,000,000
listeners per sample across 17 interleaved samples and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

## Batched validation handoff (2026-08-30)

Editor313 is included with Runtime367 under request `runtime-editor-367-313-20260830-v5`, ticket
`958895e9dfce4a809a04f798616bff8c`, source manifest hash
`d7f01a84fc63683ee16dda0788f49b530005b6f43e0ebf70e7398168daba654e`. The manifest binds the
Listener Projection source and test, Runtime367 source and test, and the compile-time validation
resource `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs` at
`a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.
