# Runtime335 Single-Pass Primitive Descriptor

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime335-editor280-performance-batch-20260829bh-v1`

## Scope

Model primitive descriptor construction previously traversed every vertex once for bounds and a
second time for planar classification. Descriptor construction now accumulates min/max coordinates
and planar state in one vertex loop. Empty meshes, floating-point min/max order, derived bounds,
primitive kind, and descriptor counts remain unchanged.

## Static Evidence

- Full vertex traversals per planar descriptor: `2 -> 1`.
- Intermediate position collections remain `0`.
- Descriptor allocations remain `0`.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME335_SINGLE_PASS_PRIMITIVE_DESCRIPTOR_BENCH_V1`.
It compares the baseline two-pass bounds/planarity path with the combined pass over 8,192 planar
vertices for 64 checks across 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
