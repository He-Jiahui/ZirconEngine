---
title: Editor asset item-generation streaming construction
category: zircon_editor
report_id: Editor01-item-generation-streaming-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor01 asset item-generation streaming construction

## Scope

`AssetWorkspaceItemGeneration` is built from filtered catalog iterators on the
Asset Browser path. The previous `FromIterator` implementation first collected
every `AssetItemSnapshot` into a temporary `Vec`, then traversed that vector a
second time to build UUID/locator indexes and 64-item chunks. This slice keeps
the existing chunk, index, selection, duplicate-key, and ordering semantics but
constructs all products in one owned pass.

## Implementation

- Added one private `from_items` builder shared by `From<Vec<AssetItemSnapshot>>`
  and `FromIterator<AssetItemSnapshot>`.
- Uses the iterator lower-bound as a safe initial bound for index/chunk pointer
  tables; filtered iterators may still grow normally when their lower bound is
  zero.
- Inserts borrowed payload clones into the existing UUID and locator indexes,
  records selected logical indices, and moves each item directly into the
  current chunk. No intermediate item vector is materialized.
- Keeps chunk boundaries, `get`, `iter`, `selected_index`, `locator_index`,
  replacement behavior, and duplicate assertions unchanged.

## Complexity and deterministic pressure model

For `N` projected rows, construction remains `O(N)` and still owns one payload
copy for each published row plus the existing identity-map keys. The temporary
collector is removed: a 1,000,000-row projection changes modeled owned item
slots from `2,000,000` (temporary vector plus chunk storage) to `1,000,000`
chunk slots. The 64-row chunk table contains 15,625 pointers in either design;
the model intentionally excludes allocator metadata and hash-table growth.

This is structural evidence, not a CPU/RSS/product timing claim. The ignored
Release marker `EDITOR741_ITEM_GENERATION_STREAM_BENCH_V1` compares the legacy
temporary-collector shape with the streaming path and requires the optimized
P95 to remain at or below 80% of the legacy P95 when the managed lane runs it.

## TDD and local evidence

- The new source contract was intentionally RED before the builder existed,
  observing the old `iter.into_iter().collect::<Vec<_>>()` path.
- After implementation, the streaming source/pressure contract passes `3/3`.
- The lower Rust regression covers a non-exact filtered iterator, cross-chunk
  order, UUID/locator indexes, and selected-index parity. The ignored benchmark
  is attached to the same test module for the next owner-attributed Release
  batch.
- The one-process non-tooling Runtime/Editor performance-plus-pressure loader
  covers `554` modules and passes `2062/2062` tests in `16.986s`, including
  this new contract and the preceding Editor740/Runtime206 slices.
- Scoped Rustfmt and Python compilation pass. The existing Runtime/Editor
  performance-plus-pressure loader remains the batched validation mechanism;
  no standalone Cargo process or coordinator query was started.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs` | `79625D746A7B4262147A53A8C1491493A1B45DB8DA773E37AA3B861B66ECAA59` |
| `tools/tests/test_editor_asset_item_generation_streaming_performance_contract.py` | `4EE76CB38F90FF041DEF48192281DFD51DC2D1F8EDCE50001606094489515029` |

## Managed acceptance gate

Keep this slice at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves compilation, iterator/chunk parity,
allocation behavior, and Asset Browser projection p50/p95/p99. Tooling
production work remains intentionally deferred.
