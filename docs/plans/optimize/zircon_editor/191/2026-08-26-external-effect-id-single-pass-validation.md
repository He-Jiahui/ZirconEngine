# Editor191 External Effect Id Single Pass Validation

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime245-editor191-performance-batch-20260826gy-v1`

## Problem

Dirty external-effect id parsing scanned valid identifiers once with `split` to detect empty
segments and then again with `char_indices` to detect invalid characters. Journal and save flows
therefore traversed every valid identifier twice before accepting it.

## Optimization

- Track leading, repeated, and trailing separators during the character-validation scan.
- Record the first invalid character without returning early so empty-segment errors retain their
  established higher precedence.
- Return the existing public error variants and byte indices without allocating validation state.

## Regression Contract

The `optimization_batch_20260826gy_` Editor tests preserve valid ids, public error mapping, byte
indices, and empty-segment precedence, enforce the single-pass source shape, and provide an ignored
paired release benchmark emitting `EDITOR191_EXTERNAL_EFFECT_ID_SINGLE_PASS_BENCH_V1`. It performs
50,000 validations per sample on a valid multi-segment id and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
