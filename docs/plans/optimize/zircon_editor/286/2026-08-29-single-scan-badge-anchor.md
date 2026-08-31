# Editor286 Single-Scan Badge Anchor Variant

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime341-editor286-performance-batch-20260829bn-v1`

## Scope

Retained Badge anchor projection previously tokenized the component variant separately for two
circular aliases, three left aliases, and three bottom aliases. One traversal now records all three
independent flags while preserving case-insensitive token matching, aliases, and combined anchors.

## Static Evidence

- Variant token traversals on an unmatched value: `8 -> 1`.
- Temporary token collections remain `0`.
- Circular offsets and left/bottom coordinate selection remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR286_SINGLE_SCAN_BADGE_ANCHOR_BENCH_V1`. It
compares eight independent token traversals with one combined traversal over 2,048-byte unmatched
variants, 4,096 checks per sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
