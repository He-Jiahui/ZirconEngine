---
title: Editor75 timeline tick projection capacity
category: zircon_editor
report_id: Editor75-timeline-tick-projection-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260915
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor75 · timeline tick projection capacity

## Scope

Static timeline content previously materialized a bounded `Vec<f32>` of tick
values and then allocated a second `Vec<TimelineStripTick>` while formatting
those values. The visual budget already proves the exact maximum tick count.
This slice writes formatted tick records directly into one bounded vector and
keeps the existing endpoint, label, generation, and hard-cap semantics.

## Implementation

- Replace the intermediate `timeline_tick_values` float vector with
  `timeline_tick_projection`, which reserves `segment_count + 1` tick records.
- Format each generated value directly through `TimelineStripTick::from_value`
  before converting the single vector to the existing `Arc<[TimelineStripTick]>`.
- Add a lower value/label/endpoint regression and the ignored
  `EDITOR75_TIMELINE_TICK_PROJECTION_CAPACITY_BENCH_V1` marker.

## Deterministic work model

For the 4,096-tick hard-cap path, the retired implementation performs two
intermediate vector allocations (float values plus formatted tick records) and
the optimized path performs one bounded tick-record allocation (`2→1`). This
is a structural allocation model only; it does not claim CPU, RSS, allocator,
or product-latency results.

## TDD and local evidence

- The new source contract was intentionally RED against the two-vector
  materialization and became GREEN after the direct projection helper (`3/3`).
- The lower Rust semantic regression and ignored Release marker are wired in
  `zircon_editor/src/ui/timeline_strip/tests.rs`.
- The six-file timeline-focused source/model batch passes `23/23`; the current
  ordinary Runtime/Editor batch passes `3406/3406` across `838` files and the
  performance/pressure batch passes `2397/2397` across `649` files, all with
  zero failures, errors, load errors, or skips.
- Existing timeline-generation and key-projection contracts remain part of the
  batched Runtime/Editor validation; managed Cargo/Release and product
  p50/p95/p99 evidence remain pending.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/timeline_strip/generation.rs` | `F0378BEB1A96AA197B0B9993C8DCB93F98630F679451A1F8917FD4449B5E70EA` |
| `zircon_editor/src/ui/timeline_strip/tests.rs` | `F63E4262D5E6058FDC7641FD624332CA25CD7CFBED6FEDE1B75DEAA84BE622B1` |
| `tools/tests/test_editor_timeline_tick_projection_capacity_performance_contract.py` | `C15358D337326CFE37676136D6A942033372860F6E1DE9D0381CFA5336FF293C` |

## Managed acceptance gate

Keep this record at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves current-source compilation, tick parity,
allocation behavior, and the declared timeline p50/p95/p99 gates. Tooling
production remains deferred for the later Rust migration.
