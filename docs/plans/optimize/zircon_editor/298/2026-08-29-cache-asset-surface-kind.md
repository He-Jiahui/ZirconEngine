# Editor298 Cache Asset Surface Kind

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime353-editor298-performance-batch-20260829bz-v1`

## Scope

Template node surface painting previously repeated the same asset-thumbnail variant and corner
radius predicate before both the base surface and state-layer branches. The result is now computed
once and reused by both branches.

## Static Evidence

- Asset-thumbnail surface predicate evaluations per eligible node: `2 -> 1`.
- Surface and state-layer branch decisions remain identical.
- Paint order, geometry, clipping, colors, opacity, and fallback surface behavior are unchanged.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR298_CACHED_ASSET_SURFACE_KIND_BENCH_V1`. It
compares two predicate calls with one cached result over 1,000,000 branch pairs per sample and 31
interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
