# Runtime320 Linear Random Scope Eviction

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime320-editor265-performance-batch-20260829at-v1`

## Scope

Random stream scope eviction previously scanned matching keys into a temporary vector and removed
each entry through a separate BTree lookup. After the active-lease preflight succeeds, eviction now
uses one ordered `retain` traversal to emit checkpoints and remove entries. A matching active lease
still rejects the complete operation before any registry mutation.

## Static Evidence

- Temporary vectors for scope eviction: `2 -> 1`.
- Per-key BTree remove lookups for 4,096 matching streams: `4096 -> 0`.
- Canonical checkpoint order and all-or-nothing active-lease rejection are unchanged.

## Performance Gate

The ignored Windows release benchmark emits
`RUNTIME320_LINEAR_RANDOM_SCOPE_EVICTION_BENCH_V1`. It evicts 4,096 matching parked streams across
31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
