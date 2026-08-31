# Editor290 Single-Scan Badge Color Variant

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime345-editor290-performance-batch-20260829br-v1`

## Scope

Retained Badge color projection previously tokenized the component variant independently for eight
color tokens. One traversal now records the highest-priority match while preserving the original
token order and validation-level/text-tone fallback.

## Static Evidence

- Unmatched variant token traversals: `8 -> 1`.
- Temporary token collections remain `0`.
- Cross-token priority and no-match fallback remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR290_SINGLE_SCAN_BADGE_COLOR_BENCH_V1`. It
compares eight independent token traversals with one combined traversal over 2,048-byte unmatched
variants, 4,096 checks per sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
