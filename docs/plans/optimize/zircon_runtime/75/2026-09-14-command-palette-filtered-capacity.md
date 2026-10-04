---
title: Runtime75 Command Palette Filtered Capacity
category: zircon_runtime
report_id: Runtime75-command-palette-filtered-capacity-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Command Palette Filtered Capacity

## Scope

`sync_filter_state` parses the command catalog before applying source and query filters. The
result cannot contain more IDs than the parsed entry slice, but the former chained filtered
iterator collected from zero capacity. This Runtime75 leaf preserves the existing source-first,
query-second admission, duplicate-ID retention, and source order while reserving that known upper
bound once.

## Implementation

- Added `filtered_command_ids_for_entries` in
  `zircon_runtime/src/ui/component/state_reducer/command_palette.rs`.
- The helper reserves `entries.len()` before the first append and uses the former source/query
  predicates in the same short-circuit order.
- `sync_filter_state` calls the helper after parsing; disabled-state projection, focus selection,
  publication, and window pagination remain outside the change.
- Added an in-file behavior/capacity regression plus ignored
  `RUNTIME755_COMMAND_PALETTE_FILTERED_CAPACITY_BENCH_V1` Release marker.

## Deterministic work boundary

For `E` parsed entries, filtering still evaluates at most `E` entries and returns the same ordered
subset. The retained ID vector starts with capacity `E`, so the modeled vector-growth count is
`0` instead of geometric growth when the selected subset is large. This is an allocation-shape
improvement only; it neither eliminates catalog parsing nor closes Runtime75's live-surface,
async-query, or product percentile gates.

## Validation

- TDD source contract was observed RED with three missing obligations, then GREEN at `4/4`:
  `tools/tests/test_runtime_command_palette_filtered_capacity_performance_contract.py`.
- One adjacent Runtime/Editor command-palette and recent-capacity invocation passed `34/34` in
  `0.146s` using `python -B -m unittest`; it is local source/model evidence.
- `rustfmt --edition 2021 --check` passes for the changed Runtime source.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
input-to-present p50/p95/p99 evidence remain required before this can be described as product
performance acceptance.
