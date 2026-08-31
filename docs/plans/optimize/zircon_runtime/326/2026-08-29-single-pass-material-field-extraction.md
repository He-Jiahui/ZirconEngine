# Runtime326 Single-Pass Material Field Extraction

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime326-editor271-performance-batch-20260829ay-v1`

## Scope

Material texture reference extraction previously probed the fixed reference-field set with
`contains_key` and then traversed the same set again to remove values. Extraction now removes each
named field once and derives presence from the resulting table. Missing references, unrelated
fields, and serialized reference values retain their previous semantics.

## Static Evidence

- Fixed-name table probes per extraction: `2N -> N`.
- Reference and unrelated field ownership remain unchanged.
- Empty reference tables still deserialize as absent references.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME326_SINGLE_PASS_MATERIAL_FIELD_BENCH_V1`. It
compares the legacy probe-plus-remove path with single-pass removal over 4,096 material tables
across 31 interleaved sample pairs and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
