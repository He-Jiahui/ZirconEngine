---
title: Editor75 timeline key projection capacity
category: zircon_editor
report_id: Editor75-timeline-key-projection-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260915
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor75 · timeline key projection capacity

## Scope

`TimelineStripGeneration::new` already receives the complete authored key
vector. Its finite-time filter and duration clamp therefore have the input
length as a safe upper bound, but the old iterator collector began at zero
capacity. This slice removes only that avoidable geometric growth; filtering,
order, time clamping, labels, selection state, and generation hashes remain
unchanged.

## Implementation

- Reserve `input.keys.len()` before the normalization loop.
- Append only finite keys, preserving the existing source order and duration
  clamp, then convert the owned vector to the existing `Arc<[TimelineStripKey]>`.
- Add a lower semantic regression for NaN rejection, order, clamping, labels,
  and selection, plus the ignored marker
  `EDITOR75_TIMELINE_KEY_PROJECTION_CAPACITY_BENCH_V1`.

## Deterministic work model

For 4,096 retained keys, the legacy zero-capacity collector models 11
geometric growth events; the bounded path models zero growth events. The input
bound is conservative when non-finite keys are filtered and does not claim a
CPU, RSS, allocator, or product-latency result.

## TDD and local evidence

- The new source contract was intentionally RED against the iterator collector
  and became GREEN after the explicit bounded loop (`3/3`).
- The existing Editor07 Timeline generation contract and the new capacity
  contract pass together (`8/8`).
- The lower Rust semantic regression and ignored Release marker are wired in
  `zircon_editor/src/ui/timeline_strip/tests.rs`; scoped Rustfmt, Python
  compilation, and `git diff --check` pass.
- The current one-process non-tooling Runtime/Editor source/model batch remains
  the batched validation mechanism; managed Cargo/Release and product
  p50/p95/p99 evidence remain pending.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/timeline_strip/generation.rs` | `ED7A994A1CF020224834D112B5E28EE8A9B0A976B32F47F8090B15980BC3CD67` |
| `zircon_editor/src/ui/timeline_strip/tests.rs` | `F2E6BCBBF146D259BB0F1EFC1C02D77BB14AC9A0E230A551C1E45D20FB831E85` |
| `tools/tests/test_editor_timeline_key_projection_capacity_performance_contract.py` | `01C3EA860E06FCD0AEDC77A3214A85B66EC0484F73098460F6ACB4622BDA5045` |

## Managed acceptance gate

Keep this record at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves current-source compilation, key parity,
allocation behavior, and the declared timeline p50/p95/p99 gates. Tooling
production remains deferred for the later Rust migration.
