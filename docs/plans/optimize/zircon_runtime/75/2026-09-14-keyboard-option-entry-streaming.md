---
title: Runtime75 Keyboard Option Entry Streaming
category: zircon_runtime
report_id: Runtime75-keyboard-option-entry-streaming-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Keyboard Option Entry Streaming

## Scope

Keyboard navigation flattens option declarations from arrays, strings, enums, and maps into
`OptionEntry` values. The former array path recursively constructed a temporary vector at every
level and then flattened those vectors. This Runtime75 slice gives the root one output vector and
streams recursive array leaves into it, while retaining the existing map ID and label rules.

## Implementation

- `option_entry_list` now creates one output vector with a conservative root shape hint.
- `collect_option_entries` appends string/enum leaves and map-derived entries directly to that
  vector, reserving each direct array bound before recursion.
- Empty string filtering, map identity aliases, label fallback, declaration order, duplicate IDs,
  and unknown-value rejection remain unchanged.
- Added a lower nested-order/capacity regression and ignored
  `RUNTIME758_KEYBOARD_OPTION_ENTRY_STREAM_BENCH_V1` Release marker.

## Deterministic work boundary

For a flat array of `N` option declarations, the final vector starts with the direct input bound
and no longer creates one recursive `Vec<OptionEntry>` per leaf/container. Nested arrays keep one
shared output vector and reserve their direct bounds as encountered. Map ID expansion intentionally
continues through the existing `option_id_list` contract. This is an allocation-shape improvement,
not allocator, CPU/RSS, or product keyboard-latency p50/p95/p99 evidence.

## Validation

- The TDD source contract was introduced before implementation and is GREEN at `4/4`:
  `tools/tests/test_runtime_keyboard_option_entry_streaming_performance_contract.py`.
- A combined Runtime/Editor command-palette, keyboard, TreeView, and adjacent capacity-contract
  invocation passes `46/46` in `0.266s` through `python -B -m unittest`.
- `rustfmt --edition 2021 --check` passes for the changed Runtime reducer sources.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product keyboard percentile evidence remain required before performance acceptance.
