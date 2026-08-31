---
title: Runtime96 Intrusive Reflection Probe Slot LRU
category: zircon_runtime
report_id: Runtime96-intrusive-probe-slot-lru-2026-08-26
date: 2026-08-26
session_id: root-runtime96-intrusive-probe-slot-lru-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime96 Intrusive Reflection Probe Slot LRU

## Scope

This slice optimizes the Runtime09F1 reflection-probe cubemap slot allocator under its current
Runtime96 source owner. It replaces the full-entry LRU scan performed for every pressure admission.
It does not change the fixed slot budget, probe resolution, capture scheduling, spatial assignment,
shader-side probe selection, residency ownership, or the public allocation receipt.

## Change

- Each live cubemap entry carries previous/next `ResourceId` links, while the allocator retains the
  oldest and newest identities.
- Existing-cubemap acquisition detaches and appends one entry without allocation. Pressure
  acquisition removes the oldest entry directly and reuses its exact slot.
- The redundant slot-owner vector and wrapping usage clock are removed. Before capacity is reached,
  the live entry count remains the next deterministic slot because the allocator has no release
  operation; after capacity, every removal and insertion is paired.
- The per-entry link fields are a bounded memory tradeoff at the current 64-slot default. They avoid
  unbounded scan work but do not claim to solve the parent plan's roughly 64 MiB probe texture budget.
- A Rust regression covers repeated touches, two successive evictions, exact victim order, and slot
  reuse, in addition to the existing capacity, LRU, and revision-upload tests.

## Deterministic Performance Evidence

The independent release model uses the product's 64-slot capacity and 131,072 consecutive pressure
admissions, with 21 alternating legacy/intrusive sample pairs per run.

| Evidence | Full-scan LRU | Intrusive LRU | Result |
|---|---:|---:|---:|
| LRU inspections/link steps | 8,388,608 | 393,279 | 95.312% fewer |
| Run 1 P50 | 103.044 ms | 12.072 ms | 88.284% faster |
| Run 1 P95 | 176.995 ms | 16.531 ms | 90.660% faster |
| Run 2 P50 | 85.808 ms | 12.010 ms | 86.003% faster |
| Run 2 P95 | 92.896 ms | 13.608 ms | 85.351% faster |
| Run 3 P50 | 99.096 ms | 12.500 ms | 87.386% faster |
| Run 3 P95 | 121.329 ms | 14.821 ms | 87.785% faster |

The managed gate requires the exact deterministic step counts above, at least 90% step reduction,
at least 75% P50 improvement, and at least 50% P95 improvement.

## Acceptance

- `tools.tests.test_runtime09f1_intrusive_probe_slot_lru_performance_contract` passes 3/3 locally.
- The four `render_probe_slot_allocator_*` Rust tests are submitted together with the source
  contracts, exact-file formatting, performance model, and scoped diff checks in one coordinator
  validation batch.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Runtime96/09F1 still needs a shared residency/budget owner, product quality tiers, generation-aware
capture scheduling, GPU-driven spatial assignment, bounded shader probe lists, and game/Editor
authoring evidence.
