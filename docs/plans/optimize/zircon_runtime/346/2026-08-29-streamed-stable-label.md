# Runtime346 Streamed Stable Asset Label

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime346-editor291-performance-batch-20260829bs-v1`

## Scope

Shader prewarm asset labels previously collected component strings into a temporary vector, joined
them into a normalized string, and formatted a second final string. The label now reserves once and
streams path components directly behind the stable scheme prefix.

## Static Evidence

- Owned container/string allocations per label: `3 -> 1`.
- Path component traversals remain `1`.
- Relative, outside-root, empty-relative, separator, and lossy component semantics remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME346_STREAMED_STABLE_LABEL_BENCH_V1`. It compares
the collect/join/format baseline with direct streaming over 256 path components, 1,024 checks per
sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
