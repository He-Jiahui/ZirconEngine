# Editor276 Borrowed Inspector Field IDs

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime331-editor276-performance-batch-20260829bd-v1`

## Scope

Inspector field-ID parsing previously allocated a `String` for every dynamic and built-in control,
then cloned it when constructing the owned runtime binding target. Parsing now returns `Cow<str>`
borrowed from the control ID or static field name; the existing `Into<String>` target boundary
performs only the required ownership conversion. Field values and status text remain unchanged.

## Static Evidence

- Parser allocations per recognized field ID: `1 -> 0`.
- Required binding-target string allocations: `1 -> 1`.
- Dynamic field slices continue to exclude the control prefix.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR276_BORROWED_INSPECTOR_FIELD_IDS_BENCH_V1`. It
compares owned and borrowed parsing over 131,072 dynamic 512-byte field IDs across 31 interleaved
sample pairs and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
