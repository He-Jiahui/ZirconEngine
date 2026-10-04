---
title: Editor04 Asset Workspace Borrowed Filter Matching
category: zircon_editor
report_id: Editor04-asset-filter-borrowed-match-2026-09-10
date: 2026-09-10
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor04 Asset Workspace Borrowed Filter Matching

## Scope

This slice removes per-candidate lowercase string materialization from the Asset Browser folder
and item filters and reserves the bounded folder-tree projection buffers up front. It preserves
ASCII case-folding, empty-query behavior, field search order, kind filtering, folder ordering,
selection flags, and the existing full-tree projection contract. It does not claim the indexed
query, lazy branch, or cursor work still owned by the broader Editor04 plan.

## Implementation

`folder_matches_search` and `asset_matches_filters` now use a byte-window matcher backed by
`eq_ignore_ascii_case`. The matcher compares the original UTF-8 byte stream against the already
normalized query and performs no temporary lowercase allocation; non-ASCII bytes retain the same
exact-match behavior as `to_ascii_lowercase().contains(...)`.

`build_folder_tree` now gives its parent-group map and flattened output vector the catalog folder
count as an initial capacity. The recursive branch and published row contents remain unchanged;
the reservation only removes geometric growth during a large-catalog materialization.

## Deterministic work model

For a non-empty query, the retired filter can allocate up to three lowercase buffers per asset
candidate (display name, file name, and locator) plus one buffer per folder candidate. The new
path performs zero matcher-owned heap allocations and keeps the same bounded linear comparisons.
The folder projection reserves one output buffer and one parent-map bucket budget from the known
folder count. This is structural allocation evidence, not a product CPU/RSS or frame-latency
measurement.

## Validation

- Rust regressions cover mixed-case, empty-query, non-ASCII, and negative matching behavior.
- Source regressions require the borrowed matcher in both filter paths and capacity reservations
  in the folder-tree builder; they reject per-candidate `to_ascii_lowercase()` calls.
- Managed Cargo and Windows Release p50/p95/p99 CPU, allocation, and RSS evidence remain pending.

## Remaining Parent Work

Editor04 still owns indexed token/kind/tag queries, lazy expanded-branch generations, independent
browser navigation state, provider capabilities, import/reimport orchestration, preview policy,
and product-scale catalog qualification.
