# Editor266 Single-Lookup View Registration

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime321-editor266-performance-batch-20260829au-v1`

## Scope

View registration previously checked descriptor uniqueness with `contains_key` and then performed a
second hash lookup for insertion. It now uses one hash-map entry lookup to reject an occupied ID or
insert a vacant descriptor. Duplicate error text and the existing descriptor are unchanged.

## Static Evidence

- Hash lookups for a unique descriptor registration: `2 -> 1`.
- Added heap allocations beyond the existing descriptor-ID clone: `0`.
- Duplicate registration still returns the same error without replacing the registered descriptor.

## Performance Gate

The ignored Windows release benchmark emits
`EDITOR266_SINGLE_LOOKUP_VIEW_REGISTRATION_BENCH_V1`. It registers 4,096 descriptors with 96-byte
IDs across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
