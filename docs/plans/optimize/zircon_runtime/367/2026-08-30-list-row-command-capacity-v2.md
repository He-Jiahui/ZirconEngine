# Runtime367 List Row Command Capacity v2

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-367-313-20260830-v6`

Current validation ticket: `a89c4f5a9afa4d17ad8bd078cb3de74c`; source manifest hash:
`e343a4d8fe5ab0e645dd76587aa8128560180362055d98248962cb006bd62571`. The batch binds the
compile-time resource `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs`
at `a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.

This v2 record carries the current source manifest after Runtime367's production list source was
restored and Editor313's listener projection record was re-established. Runtime list-row capacity
remains unchanged from v1.

The ignored Windows release benchmark emits `RUNTIME367_LIST_ROW_COMMAND_CAPACITY_BENCH_V1` and
requires `candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation,
record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom publication.

## Batched validation handoff (2026-08-30)

Runtime367 is included with Editor313 under request `runtime-editor-367-313-20260830-v5`, ticket
`958895e9dfce4a809a04f798616bff8c`, source manifest hash
`d7f01a84fc63683ee16dda0788f49b530005b6f43e0ebf70e7398168daba654e`. The manifest binds the
current list source/test and listener projection source/test, plus compile-time validation resource
`zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs` at
`a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.
