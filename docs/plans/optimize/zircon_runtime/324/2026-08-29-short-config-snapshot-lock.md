# Runtime324 Short Config Snapshot Lock

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime324-editor269-performance-batch-20260829ax-v1`

## Scope

Config snapshots previously cloned every JSON value while holding the store mutex. Snapshot capture
now clones keys and shared value handles under the lock, releases the mutex, and deep-clones the
publicly owned values afterward. Snapshot contents and the public `HashMap<String, Value>` contract
are unchanged.

## Static Evidence

- Deep JSON value clones performed while holding the config mutex: `N -> 0`.
- Shared handle clones performed while holding the mutex: `0 -> N`.
- Public snapshot ownership remains independent from subsequent store updates.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME324_SHORT_CONFIG_SNAPSHOT_LOCK_BENCH_V1`. It
compares the legacy deep-clone critical section with shared-entry capture for 1,024 8-KiB JSON
values across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.

## Validation Attempt

The managed debug ticket `92d650129cd049709bf9368ae1884694` and release ticket
`24d4642a4ffa4ed09803df17133f141d` both stopped during validation-copy artifact governance before
Cargo started. The coordinator reported the generated path
`D:\\ZirconBuilds\\mvp-test-fixtures-36724`; its fixture contains the intentional junction
`summary-log-reparse\\logs`, which governance refuses to traverse. No compile/test result or
performance sample was produced, so this record remains pending and makes no product-performance
claim.
