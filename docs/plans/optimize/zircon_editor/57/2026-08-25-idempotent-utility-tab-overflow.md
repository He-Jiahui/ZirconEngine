---
title: Editor57 Idempotent Utility-tab Overflow
category: zircon_editor
report_id: Editor57-idempotent-utility-tab-overflow-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Idempotent Utility-tab Overflow

## Scope

This slice removes redundant overflow-string reconstruction from repeated Asset Browser utility-tab
typography layout. It preserves utility control filtering, selected and idle font weights, font size,
the `elide` overflow value, node order, and all unrelated nodes. It does not change Editor57 utility
content, interaction, selection, navigation, mutation, or retained UI authority.

## Implementation

The retired typography pass assigned a new `elide` string to every utility tab on every layout call.
The current retained-host `SharedString` is a `String` alias, so a steady four-tab layout rebuilt four
owned strings. The optimized path compares the borrowed current value and assigns only on initial
layout or a real overflow-mode change. Font values are still refreshed on every call.

The regression preserves existing typography coverage and adds pointer stability across an unchanged
second layout. A source contract requires the overflow field to route through the conditional helper.

## Performance Contract

| Evidence per repeated four-tab typography layout | Retired path | Optimized gate |
| --- | ---: | ---: |
| Overflow string writes and reconstructions | 4 | 0 |
| Font size and weight writes | 8 | 8 |
| Alternating release benchmark | 11 samples x 16,384 layouts x 4 tabs | optimized P95 <= 45% of retired P95 |

The benchmark emits `EDITOR57_IDEMPOTENT_UTILITY_TAB_OVERFLOW_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/tab counts, and repeated overflow writes.

## Validation

The TDD source probe first observed the unconditional overflow assignment and no conditional helper,
then observed all three optimized source contracts. Rust 1.94.1 formatting and scoped static checks
passed before batching. One managed Editor batch covers this slice together with single-pass stack
frame discovery, including typography values, pointer stability, frame equivalence, source contracts,
and both ignored release benchmarks. Dynamic P95 evidence, integration SHA, automatic commit, and
automatic WeCom performance delivery remain coordinator-owned and pending.
