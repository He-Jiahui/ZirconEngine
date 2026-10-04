---
title: Editor04 Asset Workspace Search Normalization Cache
category: zircon_editor
report_id: Editor04-search-query-normalization-cache-2026-09-10
date: 2026-09-10
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor04 Asset Workspace Search Normalization Cache

## Scope

This slice removes repeated search-string normalization from the retained Asset Browser snapshot
and catalog-delta paths. It does not change query matching, case-folding policy, catalog
generation invalidation, folder traversal, selection, or the broader Runtime-authoritative asset
projection work owned by Editor04.

## Implementation

- `AssetWorkspaceState` now keeps `normalized_search_query` beside the authored query.
- `set_search_query` computes the ASCII-lowercase form only when the authored query actually
  changes. Idempotent setter calls do not allocate or rewrite either value.
- `build_snapshot` and `patch_catalog_item_generation` borrow the cached `&str`, so stable frame
  snapshots and watcher deltas no longer allocate a temporary lowercase `String`.
- Existing cache-key comparison and bounded replacement-buffer reservations remain unchanged;
  the cached normalized value is an internal projection aid and is not exposed in the snapshot.

## Regression and Local Evidence

- The Rust regression covers mixed-case normalization, idempotent setter calls, and subsequent
  query replacement.
- Source guards require both snapshot consumers to use `as_str()` and contain no per-call
  `to_ascii_lowercase()` operation.
- The combined Editor asset projection/Watcher source-contract batch passed `52/52`.
- The final combined Runtime UI and Editor asset projection/watcher batch passed `123/123`
  across 19 Python contract modules.
- Python syntax, scoped `rustfmt --edition 2021 --check`, and scoped `git diff --check` passed.

These are source and contract checks. Managed Editor Cargo execution and Windows Release
allocation/time measurements with warm-up and p50/p95/p99 remain pending; no product performance
acceptance is inferred from the static batch.

## Remaining Parent Work

Editor04 still owns Runtime registry/Editor catalog authority convergence, asynchronous import and
commit orchestration, reference graph ownership, preview provider/cache policy, and product-scale
asset browser validation.
