# Editor292 Single-Scan Avatar Shape

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime347-editor292-performance-batch-20260829bt-v1`

## Scope

Retained Avatar radius projection previously tokenized the component variant once for `square` and
again for `rounded`. A single traversal now resolves the shape while preserving square priority,
case-insensitive exact-token matching, and the circular fallback.

## Static Evidence

- Unmatched variant token traversals: `2 -> 1`.
- Temporary token collections remain `0`.
- Square priority, rounded metrics, bounds clamping, and circular fallback remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR292_SINGLE_SCAN_AVATAR_SHAPE_BENCH_V1`. It
compares two independent token traversals with one combined traversal over 2,048-byte unmatched
variants, 4,096 checks per sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
