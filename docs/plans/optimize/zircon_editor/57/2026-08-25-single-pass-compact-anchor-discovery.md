---
title: Editor57 Single-pass Compact Anchor Discovery
category: zircon_editor
report_id: Editor57-single-pass-compact-anchor-discovery-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Single-pass Compact Anchor Discovery

## Scope

This slice consolidates repeated control lookup at the start of compact Asset Browser layout. It
preserves first-duplicate selection, required root/main/utility admission, optional sources/content/
details presence, source/detail width inputs, and all public Editor/UI contracts. It does not alter
column budget policy or downstream panel layout.

## Implementation

The compact root path previously called `node_frame` five times, then the main-panel path repeated
three presence lookups for sources, content, and details. The optimized `CompactLayoutAnchors` scan
captures the first frame for all six control IDs in one pass and stops as soon as the complete set is
found. Optional panel presence is passed into the main-panel function instead of rediscovered.

The regression proves that the first duplicate root remains authoritative and that all six anchor
frames are preserved. The ignored release benchmark places anchors at the end of a 4,096-node table,
forces all retired searches to execute, and compares the retired eight-search summary with the new
single-pass summary before timing.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Linear node searches/passes per complete compact anchor discovery | 8 | 1 |
| Alternating release benchmark | 11 samples x 256 discoveries x 4,096 nodes | optimized P95 <= 30% of retired P95 |

The benchmark emits `EDITOR57_SINGLE_PASS_COMPACT_ANCHOR_DISCOVERY_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/node counts, and retired/optimized search counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, module-size checks, and production structure checks
are required before submission. One managed Editor57 Cargo invocation filtered by
`editor57_compact_parent_single_pass_` covers this regression and benchmark together with Details
preview projection. Dynamic P95 evidence, integration SHA, and automatic WeCom performance delivery
remain coordinator-owned and pending.
