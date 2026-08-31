# Runtime366 Route Stage Names Capacity

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-366-312-20260830-v5`

Current validation ticket: `72668f71e4fc4a4eb285f773b7688822`; source manifest hash:
`e6799e3672bdd574ec1f5a7d04ea70deb4117a53be8589193d0b3bd19b113d57`. The batch binds the
compile-time resource `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs`
at `a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.

## Scope

Route stage name collection now reserves the four entries used by the common route policies,
avoiding the first allocation growth while preserving route order and filtering semantics.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME366_ROUTE_STAGE_NAMES_CAPACITY_BENCH_V1`.
It compares an unreserved vector with the four-entry reservation over 2,000,000 Bubble-policy
four-stage collections per sample across 17 interleaved samples and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation,
record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom publication.

## Batched validation handoff (2026-08-30)

Runtime366 is included with Editor312 in the corrected `bn` batch under request
`runtime-editor-366-312-20260830-v4`, ticket `c48882a3569d49e3b8f3f1fc318ed1b7`, with source
manifest hash `4872c50cd74b3531ae42464bf3c2aa06e1997a639c5c99cbea44601b9f33e4a1`. The manifest binds
the production source, its test, and the compile-time validation resource
`zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs` at
`a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`; the benchmark uses the
production-equivalent four-stage Bubble input. Earlier `bn-v1`/`bn-v2` tickets are not valid results.
Cargo, performance, review, commit, and WeCom remain coordinator-owned and pending.
