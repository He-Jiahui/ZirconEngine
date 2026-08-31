# Runtime349 Ray Map Rebuild Capacity

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime349-editor294-performance-batch-20260829bv-v1`

## Scope

RayMap rebuild previously inserted the first frame's compatible pointer-camera pairs into an empty
HashMap and paid repeated growth and rehash costs. Rebuild now reserves the exact active-pair upper
bound before insertion; `clear` still reuses the allocation on subsequent frames.

## Static Evidence

- First-rebuild HashMap growth passes: bounded by one reserve plus inserts.
- Existing ray filtering and active-camera semantics remain unchanged.
- Empty input and inactive-camera inputs retain zero entries.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME349_RAY_MAP_REBUILD_CAPACITY_BENCH_V1`. It
compares an empty-map baseline with active-pair reservation over 256 pointers, 8 cameras, 256
rebuilds per sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
