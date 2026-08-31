---
title: Editor03 Scene Mode Hash Registry
category: zircon_editor
report_id: Editor03-scene-mode-hash-registry-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor03 Scene Mode Hash Registry

## Scope

This slice replaces the scene-mode registration owner with `HashMap` for factory creation and
descriptor lookup. Registration is a cold plugin lifecycle path and now inserts the unique mode ID
into a sorted vector by binary partition.

`registrations()` projects registrations through that sorted ID vector, preserving deterministic
component registration and diagnostics. Duplicate rejection, plugin boundary isolation, factory
ID validation, and error behavior are unchanged.

## Performance Workload

The release workload fills 16,384 realistic plugin mode IDs and performs 4,096 stable registry
lookups for the final ID.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered registry lookups | 4,096 | 0 |
| Hash registry lookups | 0 | 4,096 |
| Ordered ID insertions per registration | implicit tree maintenance | 1 binary-position insert |
| Registration-order policy changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR03_SCENE_MODE_HASH_REGISTRY_BENCH_V1`. Acceptance requires hash lookup P95 to be at least
30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826cf_scene_mode_hash_registry_preserves_registration_order` covers
  shuffled registration, stable iteration, and descriptor lookup.
- `optimization_batch_20260826cf_scene_mode_hash_registry_keeps_order_index` locks the hash owner,
  binary insertion, and ordered projection.
- `optimization_batch_20260826cf_scene_mode_hash_registry_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor03 still owns scene/prefab lifecycle, selection, modes, gizmos, picking, world replacement,
and product-scale qualification. This slice only converges scene-mode registration lookup while
preserving deterministic plugin order.
