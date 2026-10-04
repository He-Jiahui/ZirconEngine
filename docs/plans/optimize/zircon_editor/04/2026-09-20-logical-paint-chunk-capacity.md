---
title: Editor849 Logical Paint Chunk Capacity
category: zircon_editor
report_id: Editor849-logical-paint-chunk-capacity-2026-09-20
date: 2026-09-20
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor849 - logical paint chunk capacity

## Scope

Asset Browser logical-paint projection already shares immutable 64-item source
chunks and reuses unchanged projected chunks. When a chunk is rebuilt, its
source length is known, but the projected-item vector previously grew from zero
capacity through the map operation.

## Optimization

- Allocate each newly projected paint chunk at `source_chunk.len()` before
  extending projected items.
- Keep the reuse check and shared-chunk cache path ahead of projection, so
  unchanged chunks still perform no item projection or allocation.
- Preserve view-mode projection, item order, counters, cache generations, and
  `Arc<[AssetBrowserPaintItem]>` ownership semantics.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the explicit
chunk reservation existed, then GREEN after production and lower wiring were
added (`4/4`). The folder-backed lower module covers exact source-chunk bounds,
empty input, and the ignored
`EDITOR849_LOGICAL_PAINT_CHUNK_CAPACITY_BENCH_V1` Release marker. A dense
64-item chunk model removes geometric growth (`5 -> 0`) for each rebuilt chunk.

## Local validation

- `tools/tests/test_editor_logical_paint_chunk_capacity_performance_contract.py`:
  `4/4`.
- Exact-file Rustfmt and Python compilation pass for the production, lower, and
  contract files.
- The focused eight-contract Runtime/Editor loader passes `32/32` tests in
  `0.028s`; the broad non-tooling performance/pressure loader passes
  `2390/2390` tests across `653` modules in `19.949s`, with zero load errors,
  failures, errors, or skips. Managed Windows Cargo/Release, allocator, and
  Asset Browser paint product p50/p95/p99 evidence remain pending behind the
  external worktree admission gate.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/layouts/views/asset_browser/logical_paint_source.rs` | `8EC41716A1BC410CAD4BF1F093B770F3C940C61ACFFEA694F4FFF878CB54130D` |
| `zircon_editor/src/ui/layouts/views/asset_browser/logical_paint_source/capacity_tests.rs` | `E0EDD0BFA5BFF133D4378C05BD2DF14157C2FFFC5ADB82540B91B1AC6B965494` |
| `tools/tests/test_editor_logical_paint_chunk_capacity_performance_contract.py` | `914203E72B19392946450D229D80375F375AB0D1F548FABA2B68E352C56F4CAA` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed managed Windows Release lane compiles the current
Runtime/Editor tree, executes the lower regression and ignored marker, and
supplies allocator plus Asset Browser paint product p50/p95/p99 evidence.
Tooling production remains deferred for the later Rust migration.
