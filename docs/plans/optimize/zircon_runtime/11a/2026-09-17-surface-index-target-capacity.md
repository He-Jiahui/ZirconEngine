---
title: Runtime11A Surface Index Target Projection Capacity
category: zircon_runtime
report_id: Runtime11A-surface-index-target-capacity-2026-09-17
date: 2026-09-17
session_id: root-astra-optimize-20260917
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime11A · Surface-index target projection capacity

## Finding

`UiAssetSurfaceHotReloadTargets::all_target_surfaces` and
`UiAssetNodeHotReloadTargets::all_target_nodes` merged four ordered target
categories into zero-capacity vectors. Hot-reload publication and dirty-marking
paths therefore paid geometric output growth even though the sum of the four
source-vector lengths is a safe upper bound for the deduplicated result.

## Implementation

- Compute the category-length sum with saturating additions before projection.
- Reserve that bound for both surface and node target outputs.
- Keep borrowed `BTreeSet` deduplication, category order, first-seen order, and
  duplicate semantics unchanged.
- Do not reserve from the unrelated surface/node index cardinality; the bound is
  local to the four inputs and cannot overrun `usize` arithmetic.

## Performance contract

The lower Rust module contains the ignored managed Release marker
`RUNTIME792_SURFACE_INDEX_TARGET_CAPACITY_BENCH_V1`. It alternates legacy
zero-capacity and bounded projections for 17 sample pairs over four 4,096-item
categories, reports raw and p95 timings, and keeps deterministic growth checks
separate from noisy wall-clock acceptance.

The allocation-shape model starts at four entries and projects 16,384 distinct
targets: the legacy vector performs 12 modeled geometric growth events, while
the bounded vector performs 0 (`12 -> 0`). The model is source/allocation
evidence, not allocator, RSS, CPU, or product p50/p95/p99 evidence.

## Local TDD receipt

The source contract was RED before the reservation implementation (the two
projection guards failed against `Vec::new()`), then GREEN at `3/3`. The lower
surface/node order-and-capacity regressions and ignored Release marker are wired
through the existing asset-surface-index test module. Exact-file Rustfmt is
clean; the combined Runtime/Editor contract batch is deferred until this slice
joins the next batched run.

## Acceptance boundary

Keep this slice `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release batch compiles the current Runtime tree,
runs the lower regressions and ignored marker, and supplies allocation plus
hot-reload target projection p50/p95/p99 evidence. No coordinator polling or
retry is part of this slice; tooling production work remains deferred for the
later Rust migration.
