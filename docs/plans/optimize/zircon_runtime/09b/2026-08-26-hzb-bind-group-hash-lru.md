---
title: Runtime09B HZB Bind Group Hash LRU
category: zircon_runtime
report_id: Runtime09B-hzb-bind-group-hash-lru-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B HZB Bind Group Hash LRU

## Scope

This slice removes the linear lookup and deque relocation from stable HZB occlusion bind-group
cache hits. Bind groups are indexed by sampled HZB identity plus indirect-resource identity in a
`HashMap`, while a monotonic access generation retains the existing least-recently-used policy.

The 64-entry bound, bind-group creation inputs, sampled-resource revision identity, indirect
workspace revision identity, and miss-only GPU resource creation are unchanged. Access-generation
overflow rebases live entries in oldest-first order before continuing.

## Deterministic Work Model

The release workload fills all 64 entries and performs 4,096 stable hits against the legacy tail
entry.

| Work per stable frame | Legacy | Optimized |
|---|---:|---:|
| Key comparisons / hash lookups | 262,144 comparisons | 4,096 lookups |
| Deque remove-and-push relocations | 4,096 | 0 |
| Bind groups created on hits | 0 | 0 |
| Capacity or eviction-policy changes | 0 | 0 |

Deterministic lookup work falls by 98.4375%. The ignored release gate runs 17 alternating sample
pairs and emits `RUNTIME09B_HZB_BIND_GROUP_HASH_LRU_BENCH_V1`. Acceptance requires HashMap
generation-LRU P95 to be at least 50% below the legacy `VecDeque` path. Exact Windows P50/P95
timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bn_hzb_bind_group_hash_lru_preserves_key_identity` covers equivalent
  key lookup and both sampled-resource and indirect-resource revision separation.
- `optimization_batch_20260826bn_hzb_bind_group_hash_lru_eliminates_linear_hit_scan` locks the
  HashMap/generation implementation, overflow rebase, and deterministic lookup model.
- `optimization_batch_20260826bn_hzb_bind_group_hash_lru_p95` reports paired release P50/P95
  samples and enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Runtime09B still owns persistent render-scene authority, correct CPU/GPU bounds, view visibility,
GPU compaction, HZB correctness, and product-scale capture evidence. This slice only converges the
stable HZB bind-group cache hit path.
