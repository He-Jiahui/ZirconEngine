---
title: Editor25 Exact-Capacity Lazy Overlay Collection
category: zircon_editor
report_id: Editor25-overlay-exact-capacity-2026-08-25
date: 2026-08-25
session_id: root-editor25-overlay-capacity-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor25 Exact-Capacity Lazy Overlay Collection

## Scope

This slice reduces observer overhead while the Editor UI debug reflector projects Runtime overlay
snapshots. It preserves shared-before-visualizer ordering, all overlay toggles, existing damage
primitives, synthesized damage fallback, and the public diagnostics contracts. It does not claim to
close Editor25's observation-session, capture, timeline, telemetry, or product-authority gaps.

## Implementation

`primitives_from_snapshot` previously collected allowed shared primitives into a growable `Vec`,
then extended it with visualizer primitives. Every visualizer primitive was fully constructed before
its toggle was checked, so disabled overlays still cloned or formatted labels and discarded them.

The optimized path first counts allowed shared and visualizer items without constructing output
objects, includes the optional damage fallback in the count, and allocates the exact output capacity
once. Visualizer primitives use lazy `bool::then` construction, so disabled entries do not allocate
labels. An empty accepted set retains zero capacity and therefore the prior zero-allocation result.

The regression compares retired and optimized results across filtered overlays and both synthesized
and pre-existing damage cases. A source contract rejects the old collected growable vector and eager
construction while checking exact output capacity and the empty-capacity case.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Label clones per 4,096 visualizer overlays with 1,024 accepted | 4,096 | 1,024 |
| Output vector reservation | incremental growth from no accepted-size hint | exact accepted count |
| Alternating release benchmark | 11 samples x 64 collections | optimized P95 <= 65% of retired P95 |

The benchmark uses 128-byte-class labels and emits `EDITOR25_OVERLAY_EXACT_CAPACITY_BENCH_V1` with
both P95 timings, reduction basis points, sample/iteration/overlay/accepted counts, and retired versus
optimized label-clone counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped `git diff --check`, and production source guards passed before
submission (apart from the repository's existing CRLF notice). One managed Editor batch covers
retired/optimized behavioral equivalence, exact-capacity and lazy-construction contracts, and the
ignored release benchmark. Dynamic P95 evidence, integration SHA, and automatic WeCom performance
delivery remain coordinator-owned and pending.
