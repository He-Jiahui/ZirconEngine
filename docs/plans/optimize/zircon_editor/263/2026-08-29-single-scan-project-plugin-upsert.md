# Editor263 Single-Scan Project Plugin Upsert

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime318-editor263-performance-batch-20260829ar-v1`

## Scope

Project plugin enablement previously scanned manifest selections to clone the existing selection,
then `set_enabled` scanned the same vector again to replace it. The first scan now records the
position, and the completed selection is replaced directly at that position. Missing selections
still append, and the first duplicate remains authoritative.

## Static Evidence

- Existing-selection linear scans per update: `2 -> 1`.
- Added retained indexes or allocations: `0`.
- Required-plugin rejection and editor capability updates are unchanged.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR263_SINGLE_SCAN_PROJECT_PLUGIN_UPSERT_BENCH_V1`.
It performs 200 terminal updates against 4,096 selections per sample across 31 interleaved sample
pairs and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
