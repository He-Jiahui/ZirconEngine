# Editor309 Scene Mode Entry Registration

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-360-365-305-311-20260830-v4`

Current validation ticket: `1f220aa2ea7f48169ba15a8a15f0aec0`; source manifest hash:
`2c4796a46cbea9347f102a56a313ed404ee318c55c55d8956a64cde2f355568f`. The batch binds the
compile-time resource `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs`
at `a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.

## Scope

Scene-mode registration now uses one `HashMap::entry` lookup to detect duplicates and insert new
registrations, while preserving sorted mode identifiers and duplicate errors.

## Static Evidence

- `Entry::Occupied` returns the existing duplicate error.
- `Entry::Vacant` performs ordered insertion and map insertion once.
- Behavior and source-contract tests cover ordering and duplicate rejection.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR309_SCENE_MODE_ENTRY_REGISTRATION_BENCH_V1`.
It compares contains-then-insert registration with the entry-based path over 2,000,000 modes across
17 interleaved samples and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation,
exact performance timing, record finalization, manifest-only commit, push to `origin/main`, and the
one-shot WeCom result containing exact performance data, test result, commit SHA, and branch.

## Batched validation handoff (2026-08-30)

Editor309 is included in the six-group Runtime/Editor batch under request
`runtime-editor-360-365-305-311-20260830-v2`, ticket `4e5bd6355a5f45eba4cba3e7d3ebe065`, with
source manifest hash `faee7e3ab3f4119b15a685e6a49e985dff861bf013c6078ecb87415f9b0ac57e`. The
registration benchmark compares the duplicate-check sequence and entry API on identical map
insertions; the 30% p95 gate remains. Cargo, performance, independent review, commit, and WeCom
remain coordinator-owned and pending.
