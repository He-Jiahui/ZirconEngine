---
title: Runtime75 Keyboard Option ID Streaming
category: zircon_runtime
report_id: Runtime75-keyboard-option-id-streaming-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Keyboard Option ID Streaming

## Scope

Map-valued keyboard options use `option_id_list` to flatten `id`/`value` declarations. Its former
recursive array and map branches returned a temporary `Vec<String>` at each level. This Runtime75
slice keeps the same alias and empty-value behavior while collecting into one capacity-hinted
output vector.

## Implementation

- `option_id_list` now allocates one output vector from a conservative root shape hint and streams
  recursive array/map leaves through `collect_option_ids`.
- Each direct array reserves its child-count bound before recursion.
- Array-level empty IDs are removed after collection, preserving the former scalar-root behavior
  where an empty `String`/`Enum` remains representable until its caller applies its own filter.
- `id` precedence over `value`, declaration order, duplicate retention, and unknown-value
  rejection remain unchanged.
- Added a lower nested-order/empty-rule regression and ignored
  `RUNTIME759_KEYBOARD_OPTION_ID_STREAM_BENCH_V1` Release marker.

## Deterministic work boundary

For a flat array of `N` IDs, the output begins with the direct input bound and avoids recursive
temporary ID vectors. Nested arrays reserve their local direct bounds as traversed. This does not
alter map label projection or eliminate the caller's intentional map-entry materialization, and
is not allocator, CPU/RSS, or product keyboard-latency p50/p95/p99 evidence.

## Validation

- The TDD source contract was introduced before implementation and is GREEN at `4/4`:
  `tools/tests/test_runtime_keyboard_option_id_streaming_performance_contract.py`.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product keyboard percentile evidence remain required before performance acceptance.
