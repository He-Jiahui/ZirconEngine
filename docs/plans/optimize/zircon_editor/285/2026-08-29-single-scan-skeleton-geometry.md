# Editor285 Single-Scan Skeleton Geometry

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime340-editor285-performance-batch-20260829bm-v1`

## Scope

Retained skeleton geometry previously tokenized the component variant twice for frame classification
and twice for corner-radius classification. Each classifier now records all relevant variant flags in
one token traversal while preserving circular/text and rectangular/circular priority. Geometry,
clamping, configured-radius fallback, and host metrics behavior remain unchanged.

## Static Evidence

- Variant token traversals: `4 -> 2` across frame and radius decisions.
- Temporary token collections remain `0`.
- Classification priority remains explicit and stable.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR285_SINGLE_SCAN_SKELETON_GEOMETRY_BENCH_V1`. It
compares the baseline four token traversals with two combined traversals over 8,192 unmatched 4-KiB
variants across 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
