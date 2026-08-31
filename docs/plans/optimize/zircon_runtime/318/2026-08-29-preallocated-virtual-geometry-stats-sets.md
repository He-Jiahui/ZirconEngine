# Runtime318 Preallocated Virtual Geometry Stats Sets

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime318-editor263-performance-batch-20260829ar-v1`

## Scope

Virtual-geometry execution statistics previously created the unique-segment and unique-page
HashSets at zero capacity even when the draw iterator reported an exact lower size bound. The
collector now converts the input once, reads its conservative lower `size_hint`, and preallocates
both sets before hashing draws. Iterators without a lower bound retain zero-capacity behavior.

## Static Evidence

- Initial segment/page capacity for an exact 4,096-draw iterator: `0 -> 4096` each.
- Statistics, duplicate suppression, and state classification are unchanged.
- Untrusted iterator upper bounds are not used for allocation.

## Performance Gate

The ignored Windows release benchmark emits
`RUNTIME318_PREALLOCATED_VIRTUAL_GEOMETRY_STATS_SETS_BENCH_V1`. It builds statistics for 4,096
unique draws 20 times per sample across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
