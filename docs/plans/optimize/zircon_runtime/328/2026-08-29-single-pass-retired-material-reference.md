# Runtime328 Single-Pass Retired Material Reference

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime328-editor273-performance-batch-20260829ba-v1`

## Scope

Retired flattened material references previously probed `uuid` and `url` before traversing the same
keys again to remove them. The migration path now removes both keys once and derives reference
presence from the extracted table. Missing references, unrelated material fields, and migration
failure behavior remain unchanged.

## Static Evidence

- Maximum BTreeMap probes for a URL-only retired reference: `4 -> 2`.
- Unrelated slot fields remain in the source table.
- Empty extraction still skips reference migration.

## Performance Gate

The ignored Windows release benchmark emits
`RUNTIME328_SINGLE_PASS_RETIRED_MATERIAL_REFERENCE_BENCH_V1`. It compares the legacy
probe-plus-remove path with single-pass extraction over 8,192 material tables across 31 interleaved
sample pairs and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
