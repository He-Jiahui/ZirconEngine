---
title: Editor57 Single-pass Compact Details Preview
category: zircon_editor
report_id: Editor57-single-pass-compact-details-preview-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Single-pass Compact Details Preview

## Scope

This slice removes repeated full-node scans from compact Asset Browser Details preview projection. It
preserves the panel, visual, seven text rows, duplicate controls, finite coordinate normalization,
line clipping, unrelated nodes, and all public Editor/UI contracts. It does not alter Details field
layout outside the preview block.

## Implementation

`apply_compact_details_preview_layout` previously invoked the generic frame setter for the panel and
visual, then once for each of seven text controls. The optimized path computes shared geometry once
and projects every recognized preview node in one `match` traversal. Duplicate controls receive the
same frame as before and unknown controls remain untouched.

The regression compares all frame outputs against a local retired implementation, including a
duplicate text node and unrelated node. The ignored release benchmark repeats the same equivalence
check over a 4,096-node preview-heavy fixture before timing.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Full node-slice passes per Details preview projection | 9 | 1 |
| Alternating release benchmark | 11 samples x 256 projections x 4,096 nodes | optimized P95 <= 35% of retired P95 |

The benchmark emits `EDITOR57_SINGLE_PASS_COMPACT_DETAILS_PREVIEW_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/node counts, and full-pass counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, module-size checks, and production structure checks
are required before submission. One managed Editor57 Cargo invocation filtered by
`editor57_compact_parent_single_pass_` covers this regression and benchmark together with compact
anchor discovery. Dynamic P95 evidence, integration SHA, and automatic WeCom performance delivery
remain coordinator-owned and pending.
