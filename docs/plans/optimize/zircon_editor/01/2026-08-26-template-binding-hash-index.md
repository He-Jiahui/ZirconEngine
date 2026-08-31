---
title: Editor01 Template Binding Hash Index
category: zircon_editor
report_id: Editor01-template-binding-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Template Binding Hash Index

## Scope

This slice replaces the editor template adapter's binding owner with `HashMap`. Retained template
materialization now resolves stable binding IDs through expected constant-time lookup.
Registration uses the entry API, combining duplicate detection and insertion into one hash probe.

The adapter exposes no binding-order iterator. Binding cloning, missing-binding errors,
event-kind validation, instance binding order, payload ownership, and native event routing are
unchanged.

## Performance Workload

The release workload fills 1,024 binding IDs with long shared prefixes and performs 4,096 stable
hits for the final ID.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered binding-ID lookups | 4,096 | 0 |
| Hash binding-ID lookups | 0 | 4,096 |
| Registration membership probes | 1 + insert | 1 entry probe |
| Allocations on binding hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_TEMPLATE_BINDING_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at least
30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826bw_template_binding_hash_index_preserves_resolution` covers typed
  resolution, duplicate rejection, and event-kind mismatch errors.
- `optimization_batch_20260826bw_template_binding_hash_index_uses_single_probe_registration` locks
  hash ownership and entry-based registration.
- `optimization_batch_20260826bw_template_binding_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization,
accessibility, and product-scale interaction evidence. This slice only converges editor template
binding registration and resolution.
