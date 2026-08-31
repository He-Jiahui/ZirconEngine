---
title: Editor57 Single-pass Summary Layout
category: zircon_editor
report_id: Editor57-single-pass-summary-layout-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Single-pass Summary Layout

## Scope

This slice consolidates Asset Browser compact-summary text discovery and frame assignment. It
preserves zero-size collapse behavior, finite coordinate normalization, visual sizing, two-line name
placement, type and revision width measurement, meta-row placement, duplicate-control updates, and
all unrelated nodes. It does not change Editor57 asset data, selection, navigation, mutation, or
retained UI authority.

## Implementation

The retired layout searched the full node slice three times for continuation, type, and revision
text, then scanned the full slice nine more times to assign the nine summary frames. The optimized
layout performs one immutable prefix-classification pass that preserves first-match text semantics,
calculates the same frame values, and performs one mutable prefix-classification pass that applies
all nine frames. Controls outside `AssetBrowserContentPreview` are rejected by one prefix check.

The regression compares every retired and optimized node frame for two-line, one-line, and tiny-card
inputs with 128 unrelated nodes. A source contract rejects the repeated text and frame helpers and
requires the two consolidated passes.

## Performance Contract

| Evidence per non-collapsed summary layout | Retired path | Optimized gate |
| --- | ---: | ---: |
| Full node-slice scans | 12 | 2 |
| Frame update passes | 9 | 1 |
| Text discovery passes | 3 | 1 |
| Alternating release benchmark | 11 samples x 128 layouts x 521 nodes | optimized P95 <= 35% of retired P95 |

The benchmark emits `EDITOR57_SINGLE_PASS_SUMMARY_LAYOUT_BENCH_V1` with both P95 timings, reduction
basis points, sample/iteration/node counts, and full node-slice scans.

## Validation

The TDD source probe first observed all six retired-path indicators, then observed the consolidated
label and frame passes with prefix classification and no repeated helpers. Rust 1.94.1 formatting
and scoped static checks passed before batching. The next managed Editor batch will cover behavior
equivalence, the source contract, and the ignored release benchmark together with at least one more
independent Editor57 optimization. Dynamic P95 evidence, integration SHA, automatic commit, and
automatic WeCom performance delivery remain coordinator-owned and pending.
