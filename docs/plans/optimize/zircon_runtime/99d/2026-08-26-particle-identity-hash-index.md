---
title: Runtime99D Particle Identity Hash Index
category: zircon_runtime
report_id: Runtime99D-particle-identity-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime99d-two-task-particle-performance-batch-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99D Particle Identity Hash Index

## Scope

This slice replaces particle previous-state matching's private ordered identity/count maps and
anonymous-entity set with `HashMap` and `HashSet`. The frame path only increments counts, performs
identity `get_mut`, tests ambiguity membership, and sums counts; it never iterates these containers
to define sprite output order.

Stable sprite identity multiplicity, anonymous key-zero ambiguity, missing previous-state counts,
and current sprite traversal order remain unchanged. All downstream consumers use the ambiguity
set only through membership checks.

## Performance Workload

The release workload builds a 16,384-entry particle identity count index and performs 4,096
mutable identity lookups per iteration.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered identity insertions | 16,384 | 0 |
| Ordered mutable lookups | 4,096 | 0 |
| Hash identity entries | 0 | 16,384 |
| Hash mutable lookups | 0 | 4,096 |
| Sprite order projections | 0 | 0 |

Both private hash indexes reserve their source upper bound before insertion. The ignored release
gate runs 17 alternating sample pairs and emits
`RUNTIME99D_PARTICLE_IDENTITY_HASH_INDEX_BENCH_V1`. Acceptance requires hash build-and-lookup P95
to be at least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending
the coordinator run.

## Acceptance

- `runtime99d_batch_particle_identity_hash_index_preserves_match_policy` covers
  anonymous ambiguity, stable-key matching, excess previous sprites, and missing-state counts.
- `runtime99d_batch_particle_identity_hash_index_has_no_order_projection` locks the
  hash map/set owners and prevents hidden sorting from returning.
- `runtime99d_batch_particle_identity_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

This task shares one managed Runtime99d validation ticket with exact particle vertex capacity. The
batch runs five `runtime99d_batch_` Rust tests, three Python contracts, one Windows release model,
and emits two exact performance rows; no local Cargo lane is launched.

## Remaining Parent-plan Work

Runtime99D still owns CPU/GPU simulation budgets, deterministic spawning, renderer submission,
sorting, collision, LOD, streaming, and product-scale qualification. This slice only converges
the frame-local previous-state identity lookup.
