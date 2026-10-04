---
title: Runtime22 Random Selector Compiled Weight Table
category: zircon_runtime
report_id: Runtime799-random-selector-compiled-weight-table-2026-09-18
date: 2026-09-18
session_id: root-runtime-editor-async-optimization-20260918
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime799 · Random-selector compiled weight table

## Scope

The behavior-tree random-selector executor resolved every child weight by
scanning the node parameter slice and then allocated a temporary `Vec<f32>` on
each selection. This slice compiles the existing ID-first / positional-fallback
weight semantics once with the behavior tree and lets the tick path borrow the
stable table. The existing floating-point accumulation and `DefaultHasher`
selection contract are deliberately unchanged; Runtime22's separate random
authority and generation migration remains open.

## Implementation

- Added `CompiledRandomSelectorWeights` with one weight per compiled child and
  the matching precomputed total.
- Populate the table only for `RandomSelector` nodes after child compilation,
  preserving child order, duplicate-parameter first-match behavior, scalar
  clamping, and the default weight of `1.0`.
- Make `weighted_random_child` use the compiled slice without allocation or
  parameter scans. The old borrowed resolver remains only as a defensive
  fallback when a malformed table length is encountered.

## Deterministic performance model

For a 1,024-child selector with one weight parameter per child, the retired
tick path performs one temporary weight-vector allocation and up to 1,048,576
parameter probes per selection. The compiled path performs zero per-selection
allocations and zero parameter probes; the one-time table contains 1,024
entries. This is allocation/operation-shape evidence, not product CPU, RSS,
power, or p50/p95/p99 evidence.

## Local evidence

- TDD source contract is GREEN (`4/4`).
- Lower Rust regression covers child-order, ID-over-position precedence,
  negative-weight clamping, total calculation, and the ignored marker
  `RUNTIME799_RANDOM_SELECTOR_WEIGHT_TABLE_BENCH_V1`.
- Exact-file Rustfmt and Python compilation pass. The combined Runtime/Editor
  source-contract batch passes `2006/2006` in `13.308s` with zero failures,
  errors, or skips; managed Cargo and product percentile evidence remain
  pending.

## Managed compile repair (2026-09-21)

The one-time v8 receipt reached the AI runtime and exposed `E0308` in the shared
selection helper: iterating the borrowed compiled weight slice yields `&f32`,
while the extracted helper compared and subtracted it as an owned `f32`. The
helper now explicitly dereferences the borrowed weight in both operations. The
source contract was extended first, observed RED at `3/4`, and is GREEN at
`4/4`; exact Rustfmt and scoped diff checks pass. Compilation confirmation is
deferred to the next combined current-source lane.

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release batch compiles the AI runtime, runs the
lower selector regression and ignored marker, and supplies representative
selector allocation and latency evidence. Runtime22 random authority,
generation-qualified identity, replay state, and the full AI consumer hard cut
are not claimed here. Tooling production work remains deferred for the later
Rust migration.
