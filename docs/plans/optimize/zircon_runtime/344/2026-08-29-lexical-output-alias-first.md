# Runtime344 Lexical Output Alias First

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime344-editor289-performance-batch-20260829bq-v1`

## Scope

Export validation previously attempted physical file identity before checking whether report and
contents-artifact paths were already lexical aliases. The equivalent OR predicate now checks the
pure lexical relation first and only performs filesystem identity probing for unresolved paths.

## Static Evidence

- Filesystem identity probes for lexical aliases: `1 -> 0`.
- Path allocations remain unchanged.
- Lexical aliases, hard links, distinct paths, and error fallback retain the same boolean result.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME344_LEXICAL_OUTPUT_ALIAS_FIRST_BENCH_V1`. It
compares identity-first and lexical-first predicates over 2,048 checks per sample and 31 interleaved
sample pairs using nonexistent lexical aliases and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
