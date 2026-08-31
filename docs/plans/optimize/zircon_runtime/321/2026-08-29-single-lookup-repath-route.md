# Runtime321 Single-Lookup Repath Route

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime321-editor266-performance-batch-20260829au-v1`

## Scope

Cached navigation repath lookup previously probed the route map once to validate the request and a
second time to obtain the mutable route. It now holds one occupied entry through validation,
waypoint advancement, and exhaustion removal. Missing, mismatched, successful, and exhausted route
behavior is unchanged.

## Static Evidence

- Hash lookups for a matching cached route: `2 -> 1`.
- Added heap allocations per lookup: `0`.
- Mismatched and exhausted routes are still removed before returning `None`.

## Performance Gate

The ignored Windows release benchmark emits
`RUNTIME321_SINGLE_LOOKUP_REPATH_ROUTE_BENCH_V1`. It resolves 4,096 cached routes over 16 lookup
passes across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
