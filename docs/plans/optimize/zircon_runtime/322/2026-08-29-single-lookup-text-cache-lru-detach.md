# Runtime322 Single-Lookup Text Cache LRU Detach

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime322-editor267-performance-batch-20260829av-v1`

## Scope

Text-cache recency updates and removals previously checked LRU membership before removing the same
slot, then checked each linked neighbor before mutably looking it up. Both callers now attempt the
removal directly, and detach updates each present neighbor through one mutable lookup. Missing-slot,
untracked-entry, and damaged-neighbor recovery behavior is unchanged.

## Static Evidence

- Hash probes for a stable middle-node touch: `9 -> 6`.
- Added heap allocations per touch: `0`.
- Missing previous and next neighbors still repair the surviving LRU boundary links.

## Performance Gate

The ignored Windows release benchmark emits
`RUNTIME322_SINGLE_LOOKUP_TEXT_CACHE_LRU_DETACH_BENCH_V1`. It touches 4,096 tracked entries over 16
passes across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
