---
title: Runtime09C Shading Model Token Hash Index
category: zircon_runtime
report_id: Runtime09C-shading-model-token-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime09c-three-task-material-index-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09C Shading Model Token Hash Index

## Scope

This slice replaces the ordered shading-model token index with a `HashMap`. Material lighting-model
resolution now uses expected constant-time normalized-token lookup, while the descriptor table
remains a `BTreeMap` so deterministic descriptor iteration and ID ordering are unchanged.

Token trimming/lowercasing, borrowed common-token lookup, custom-token fallback normalization,
duplicate ID/token errors, plugin ID range validation, supported GBuffer channel validation, and
descriptor ownership are unchanged.

## Performance Workload

The release workload fills 256 normalized tokens sharing a long prefix and performs 4,096 stable
hits.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered token-index lookups | 4,096 | 0 |
| Hash token-index lookups | 0 | 4,096 |
| Descriptor ordered-table policy changes | 0 | 0 |
| Token allocations on direct hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME09C_SHADING_TOKEN_HASH_INDEX_BENCH_V1`. Acceptance requires HashMap token lookup P95 to be
at least 30% below the legacy BTreeMap path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `runtime09c_batch_shading_token_hash_index_preserves_normalized_lookup` covers
  HashMap ownership and case-normalized custom lighting-model resolution.
- `runtime09c_batch_shading_token_hash_index_preserves_order_and_duplicates` covers
  duplicate-token rejection and unchanged descriptor ID order.
- `runtime09c_batch_shading_token_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.
- The managed `runtime09c_batch_` release gate covers this task, material-option value hashing,
  and material-property schema rescan elision in one Cargo invocation: 3 source contracts, 9 Rust
  tests, and 3 performance rows. Dynamic marker values, integration commit, and WeCom delivery
  remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime09C still owns material shader compilation, pipeline variants, PSO lifetime, persistence,
and product GPU evidence. This slice only converges shading-model token lookup.
