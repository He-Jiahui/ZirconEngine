---
title: Editor57 Exact Logical Paint Chunk Capacity
category: zircon_editor
report_id: Editor57-exact-logical-paint-chunk-capacity-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Exact Logical Paint Chunk Capacity

## Scope

This slice reserves the logical-paint chunk vector from the source generation's exact-size chunk
iterator. It preserves cache-hit behavior, chunk reuse identity, projection order, chunk build/reuse
counters, item projection counts, list/thumbnail variants, and published logical item contents. It does
not change chunk size, visible-asset generation ownership, selection, virtualization, or cache keys.

## Implementation

The retired cache-miss path created an empty chunk vector and allowed geometric growth while pushing
each reused or newly projected chunk. `AssetWorkspaceItemGeneration::item_chunks` is already an
`ExactSizeIterator`, so the optimized path keeps that iterator, allocates capacity for its exact
length, and consumes it through the unchanged projection loop.

The regression verifies projected item count and variant, while the source contract fixes iterator,
reserve, and projection-loop order. The focused capacity benchmark isolates chunk-vector assembly so
its P95 measures the changed operation rather than asset text projection.

## Performance Contract

| Evidence per 16,384 items / 256 chunks | Retired path | Optimized gate |
| --- | ---: | ---: |
| Dynamic chunk-vector capacity growths | multiple | 1 exact reserve |
| Chunk pushes | 256 | 256 |
| Alternating release benchmark | 11 samples x 1,024 assemblies | optimized P95 <= 80% of retired P95 |

The benchmark emits `EDITOR57_EXACT_LOGICAL_PAINT_CHUNK_CAPACITY_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/item/chunk counts, and measured retired/optimized capacity
growths.

## Validation

The scoped TDD source probe first observed the missing exact iterator and reserve, then observed one
exact reservation before the unchanged chunk projection loop. Rust 1.94.1 formatting and scoped
static checks passed before batching. One managed Editor57 batch covers this slice together with exact
table-row capacity, including equivalence, source contracts, and both ignored release benchmarks.
Dynamic P95 evidence, integration SHA, automatic commit, and automatic WeCom performance delivery
remain coordinator-owned and pending.
