---
title: Editor25 Single-Pass Timeline Selection
category: zircon_editor
report_id: Editor25-single-pass-timeline-selection-2026-08-25
date: 2026-08-25
session_id: root-editor25-overlay-capacity-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor25 Single-Pass Timeline Selection

## Scope

This slice removes repeated timeline-summary scans while the Editor debug reflector resolves its
selected frame. It preserves invalid-selection fallback, missing-frame behavior, previous/next
neighbors, latest labels, row order, and public diagnostics contracts. It does not claim to close
Editor25's larger timeline virtualization, capture, telemetry, or product-authority gaps.

## Implementation

The timeline model previously searched the summary slice independently to validate the selected
handle, fetch its summary, fetch the matching snapshot, find neighbors, and fetch latest summary.
The common selected-equals-latest case therefore traversed the same tail-positioned handle five
times.

The optimized resolver finds the selected index once, derives selected summary, snapshot, and
neighbors by index, and reuses the selected summary when it is also latest. A distinct latest handle
still receives its required lookup, while invalid selected/latest fallback semantics remain intact.

The regression compares retired and optimized projection signatures for valid selection, invalid
selection with valid fallback, invalid latest, and an empty timeline. A source contract requires the
single resolver and rejects the three retired scanning helpers.

## Performance Contract

| Evidence for 4,096 frames, selected=latest at tail | Retired path | Optimized gate |
| --- | ---: | ---: |
| Handle comparisons per projection | 20,480 | 4,096 |
| Alternating release benchmark | 11 samples x 256 projections | optimized P95 <= 35% of retired P95 |

The benchmark emits `EDITOR25_SINGLE_PASS_TIMELINE_SELECTION_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/frame counts, and retired/optimized handle comparisons.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped `git diff --check`, and production source guards passed before
submission (apart from the repository's existing CRLF notice). One managed Editor batch covers
retired/optimized resolution equivalence, the single-resolver source contract, and the ignored
release benchmark. Dynamic P95 evidence, integration SHA, and automatic WeCom performance delivery
remain coordinator-owned and pending.
