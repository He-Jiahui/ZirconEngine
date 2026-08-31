---
title: Editor01 UI Import Borrowed Hash Cache
category: zircon_editor
report_id: Editor01-ui-import-borrowed-hash-cache-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 UI Import Borrowed Hash Cache

## Scope

This slice replaces the per-generation UI asset physical-parse cache with `HashMap` and adds a
borrowed `Path` hit path before owned insertion. Repeated logical fragment aliases and open
documents now reuse a cached physical parse without allocating a `PathBuf` for every hit.

Only misses allocate the owned key and invoke the loader. Successful parsed documents and parse
failures remain cached once per physical path, `Arc` sharing and error conversion are unchanged,
and the private cache exposes no iteration order.

## Performance Workload

The release workload fills 4,096 long physical UI paths and performs 4,096 stable hits for the
final path.

| Work per workload | Before | After |
|---|---:|---:|
| `PathBuf` allocations on hits | 4,096 | 0 |
| Ordered path lookups | 4,096 | 0 |
| Borrowed hash path lookups | 0 | 4,096 |
| Loader calls on hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_UI_IMPORT_BORROWED_HASH_CACHE_BENCH_V1`. Acceptance requires borrowed hash lookup P95 to
be at least 50% below the legacy owned-key `BTreeMap` path. Exact Windows P50/P95 timings remain
pending the coordinator run.

## Acceptance

- `optimization_batch_20260826cc_ui_import_borrowed_hash_cache_loads_each_path_once` covers cached
  failure replay and one loader call per physical path.
- `optimization_batch_20260826cc_ui_import_borrowed_hash_cache_allocates_only_on_miss` locks the
  borrowed hit before owned-key insertion and absence of ordered traversal.
- `optimization_batch_20260826cc_ui_import_borrowed_hash_cache_p95` reports paired release P50/P95
  samples and enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns full retained UI generation, invalidation, projection, native presentation,
asset dependency refresh, and end-to-end profile qualification. This slice only converges the
physical UI import parse cache.
