---
title: Editor57 Single-pass Stack Frame Discovery
category: zircon_editor
report_id: Editor57-single-pass-stack-frame-discovery-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Single-pass Stack Frame Discovery

## Scope

This slice consolidates standard Asset Browser stack-layout discovery of the main and utility panel
frames. It preserves first matching control semantics, missing-anchor early return, viewport and
toolbar geometry, stretchable surface classification, frame deltas, and all unrelated nodes. It does
not change Editor57 data, selection, navigation, compact layout, mutation, or retained UI authority.

## Implementation

The retired path scanned the node slice independently for the main panel and utility panel before
performing the existing frame-update pass. The optimized path discovers both first matches in one
scan and stops as soon as both anchors are present. All subsequent geometry and node updates remain
unchanged.

The regression compares every retired and optimized node frame with 128 unrelated nodes and verifies
that a missing utility anchor leaves all frames unchanged. A source contract rejects the repeated
`node_frame` helper and requires the combined early-exit scan.

## Performance Contract

| Evidence per standard stack layout | Retired path | Optimized gate |
| --- | ---: | ---: |
| Full node-slice passes including frame update | 3 | 2 |
| Anchor discovery passes | 2 | 1 |
| Alternating release benchmark | 11 samples x 256 layouts x 517 nodes | optimized P95 <= 85% of retired P95 |

The benchmark emits `EDITOR57_SINGLE_PASS_STACK_FRAME_DISCOVERY_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/node counts, and full node-slice passes.

## Validation

The TDD source probe first observed two repeated lookups and no combined helper, then observed all
five single-pass source contracts. Rust 1.94.1 formatting and scoped static checks passed before
batching. One managed Editor batch covers this slice together with idempotent utility-tab overflow,
including frame equivalence, missing-anchor behavior, source contracts, pointer stability, and both
ignored release benchmarks. Dynamic P95 evidence, integration SHA, automatic commit, and automatic
WeCom performance delivery remain coordinator-owned and pending.
