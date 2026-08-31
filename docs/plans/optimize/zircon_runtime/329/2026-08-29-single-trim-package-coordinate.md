# Runtime329 Single-Trim Package Coordinate

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime329-editor274-performance-batch-20260829bb-v1`

## Scope

Runtime plugin package coordinate validation previously trimmed the input twice and compared the
second trimmed slice with the complete segment before scanning its lowercase token characters.
Validation now trims once and detects boundary whitespace from the subslice length. Diagnostics and
accepted coordinate characters remain unchanged.

## Static Evidence

- `str::trim` calls per coordinate segment: `2 -> 1`.
- Full valid-segment equality comparisons: `1 -> 0`.
- Lowercase ASCII token validation remains unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME329_SINGLE_TRIM_PACKAGE_COORDINATE_BENCH_V1`.
It compares the legacy two-trim/full-equality path with one trim and a length check over 8,192
valid 4-KiB coordinate checks across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
