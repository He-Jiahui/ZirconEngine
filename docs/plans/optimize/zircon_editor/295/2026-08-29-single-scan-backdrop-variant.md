# Editor295 Single-Scan Backdrop Variant

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime350-editor295-performance-batch-20260829bw-v1`

## Scope

Retained material backdrop projection previously tokenized the component variant independently for
`open` and `invisible`. One traversal now captures both flags before admission and preserves the
invisible suppression rule, exact-token matching, and popup/surface fallbacks.

## Static Evidence

- Backdrop variant traversals: `2 -> 1`.
- Temporary token collections remain `0`.
- Open admission, invisible suppression, clip rejection, and paint output remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR295_SINGLE_SCAN_BACKDROP_VARIANT_BENCH_V1`. It
compares two independent token traversals with one combined traversal over a 2,048-byte variant,
4,096 checks per sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
