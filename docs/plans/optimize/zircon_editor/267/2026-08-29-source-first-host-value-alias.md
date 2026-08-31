# Editor267 Source-First Host Value Alias

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime322-editor267-performance-batch-20260829av-v1`

## Scope

Host-value TOML alias projection previously looked for every target before discovering that the
corresponding source was absent. The helper now checks the source first, exits immediately for the
common no-alias case, and checks the target only when an alias value can be projected. Existing
targets still take precedence and source values are cloned only for an actual insertion.

## Static Evidence

- BTree lookups per missing alias source: `2 -> 1`.
- Added heap allocations per missing alias: `0`.
- Source-present insertion and existing-target precedence are unchanged.

## Performance Gate

The ignored Windows release benchmark emits
`EDITOR267_SOURCE_FIRST_HOST_VALUE_ALIAS_BENCH_V1`. It checks three absent aliases against 4,096
96-byte keys over 16,384 passes across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
