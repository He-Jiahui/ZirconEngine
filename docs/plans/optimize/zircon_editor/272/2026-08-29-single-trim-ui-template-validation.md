# Editor272 Single-Trim UI Template Validation

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime327-editor272-performance-batch-20260829az-v1`

## Scope

UI template document validation previously trimmed the same string twice and compared the second
trimmed slice with the entire input. Validation now trims once and detects boundary whitespace from
the subslice length. Empty, whitespace-padded, Unicode-whitespace, suffix, and owned-error behavior
remain unchanged.

## Static Evidence

- `str::trim` calls per document: `2 -> 1`.
- Full valid-document equality comparisons per document: `1 -> 0`.
- `.zui` suffix validation remains unchanged.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR272_SINGLE_TRIM_UI_TEMPLATE_VALIDATION_BENCH_V1`.
It compares the legacy two-trim/full-equality path with one trim and a length check over 4,096
4-KiB documents across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
