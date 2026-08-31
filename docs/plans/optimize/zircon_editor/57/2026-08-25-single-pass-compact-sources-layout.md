---
title: Editor57 Single-pass Compact Sources Layout
category: zircon_editor
report_id: Editor57-single-pass-compact-sources-layout-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Single-pass Compact Sources Layout

## Scope

This slice removes repeated full-node scans from compact Asset Browser sources-panel layout. It
preserves token-derived metrics, finite coordinate normalization, duplicate fixed controls, source
row ordering, shallow-panel clipping, and all public Editor/UI contracts. It does not claim to close
Editor57's remaining asset identity, actions, mutation, preview cache, or product-integration gaps.

## Implementation

`apply_compact_sources_panel_layout` previously called the fixed-control setter six times and then
scanned the node slice again for source rows. The optimized path computes the shared geometry once
and updates fixed controls and ordered rows in one `iter_mut` pass. Unknown controls continue without
mutation, while duplicate fixed controls receive the same frame as before.

The regression covers duplicate title controls, multiple source rows, row order, and an unrelated
node. The ignored release benchmark compares the retired seven-pass implementation with the current
single-pass implementation and verifies frame-for-frame equivalence before timing.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Full node-slice passes per compact sources layout | 7 | 1 |
| Alternating release benchmark | 11 samples x 256 layouts x 4,096 nodes | optimized P95 <= 50% of retired P95 |

The benchmark emits `EDITOR57_SINGLE_PASS_COMPACT_SOURCES_LAYOUT_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/node/row counts, and full-pass counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, and production structure checks are required before
submission. One managed Editor57 Cargo invocation filtered by `editor57_compact_single_pass_`
covers this behavior regression and ignored release benchmark together with compact utility
collapse. Dynamic P95 evidence, integration SHA, and automatic WeCom performance delivery remain
coordinator-owned and pending.
