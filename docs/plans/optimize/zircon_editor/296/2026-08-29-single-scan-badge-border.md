# Editor296 Single-Scan Badge Border

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime351-editor296-performance-batch-20260829bx-v1`

## Scope

Retained Badge border projection previously tokenized the component variant once for
`overlapCircular` and again for `circular`. A single traversal now recognizes both exact aliases and
preserves the declared border-color fallback.

## Static Evidence

- Unmatched border alias traversals: `2 -> 1`.
- Temporary token collections remain `0`.
- Alias case folding, exact-token matching, background reuse, and declared border fallback remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR296_SINGLE_SCAN_BADGE_BORDER_BENCH_V1`. It
compares two independent token traversals with one combined traversal over a 2,048-byte unmatched
variant, 4,096 checks per sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
