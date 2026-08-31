# Runtime336 Borrowed Shader Pass Token

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime336-editor281-performance-batch-20260829bi-v1`

## Scope

Shader prewarm manifest pass-token parsing previously allocated a lowercase copy before scanning the
fixed pass table. Lookup now trims once and compares the borrowed token with each canonical pass by
ASCII case-insensitive equality. Whitespace handling, accepted pass names, unknown-token behavior,
and material pass order remain unchanged.

## Static Evidence

- Lowercase token allocations per lookup: `1 -> 0`.
- Temporary lookup collections remain `0`.
- Lookup remains ASCII case-insensitive.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME336_BORROWED_SHADER_PASS_TOKEN_BENCH_V1`. It
compares the baseline lowercase-allocation lookup with borrowed lookup over 8,192 unknown 4-KiB
tokens across 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
