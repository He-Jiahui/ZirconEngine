# Runtime342 Single-Scan Material Name Sort

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime342-editor287-performance-batch-20260829bo-v1`

## Scope

Material management sorting previously traversed both names once for ASCII-folded ordering and a
second time for the case-sensitive tie-break. The comparator now records the first raw-byte
difference during the folded comparison and resolves both orderings in one traversal.

## Static Evidence

- Equal-under-ASCII-case name traversals: `2 -> 1`.
- Temporary lowercase buffers remain `0`.
- Folded primary ordering, raw-byte tie-break, length ordering, and non-ASCII bytes remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME342_FAST_MATERIAL_NAME_CASE_BENCH_V1`. It
compares the two-pass iterator comparator with the one-pass folded/tie-break comparator over
4,096-byte names, 8,192 checks per sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
