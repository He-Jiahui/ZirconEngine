---
title: Editor57 Linear Kind-chip Selection
category: zircon_editor
report_id: Editor57-linear-kind-chip-selection-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Linear Kind-chip Selection

## Scope

This slice removes repeated node searches and temporary candidate vectors from Asset Browser kind-chip
layout. It preserves the mandatory All chip, selected-chip priority, declaration-order admission,
fallback widths, row gaps, width-limit behavior, and the final hide/layout pass. It does not change
Editor57's query, selection, asset type, navigation, or virtualization contracts.

## Implementation

The retired path searched the template node slice once per chip to discover selection. It then cloned
the growing visible-chip vector for each optional chip and recomputed the full candidate stack width,
which searched the node slice once for every currently admitted chip. The final layout searched every
visible chip again.

The optimized path projects each chip's width and selected bit into a fixed array with one node search
per chip. A fixed visibility array then admits mandatory and optional chips in the same declaration
order using a running width. The final layout reuses the projected widths. Missing nodes retain the
same fallback width and unselected state.

The regression compares retired and optimized visibility across five width limits, selected and
unselected states, and a missing-node fallback case. A source contract rejects candidate cloning and
the retired repeated stack-width helper.

## Performance Contract

| Evidence for six chips, none selected, all admitted | Retired path | Optimized gate |
| --- | ---: | ---: |
| Template-node searches per selection/layout | 32 | 6 |
| Candidate `Vec` clones per selection | 5 | 0 |
| Width projection storage | growing `Vec<&str>` candidates | two fixed six-element arrays |
| Alternating release benchmark | 11 samples x 8,192 selections | optimized P95 <= 40% of retired P95 |

The benchmark places 64 unrelated template nodes before the six chip nodes and emits
`EDITOR57_LINEAR_KIND_CHIP_SELECTION_BENCH_V1` with both P95 timings, reduction basis points,
sample/iteration/prefix/chip counts, node searches, and candidate clones.

## Validation

The TDD source probe first observed candidate clones and repeated stack-width scans with no fixed
visibility table, then observed zero candidate clones, zero repeated stack-width scans, and fixed
array projection after implementation. Rust 1.94.1 formatting and scoped diff checks passed before
submission. One managed Editor batch covers behavior equivalence, the source contract, and the
ignored release benchmark. Dynamic P95 evidence, integration SHA, automatic commit, and automatic
WeCom performance delivery remain coordinator-owned and pending.
