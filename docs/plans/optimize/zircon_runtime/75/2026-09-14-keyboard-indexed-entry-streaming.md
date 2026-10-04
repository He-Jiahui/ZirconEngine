---
title: Runtime75 Keyboard Indexed Entry Streaming
category: zircon_runtime
report_id: Runtime75-keyboard-indexed-entry-streaming-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Keyboard Indexed Entry Streaming

## Scope

Indexed keyboard controls collect IDs from one or more candidate properties. The former pipeline
first materialized all IDs into one vector, then consumed that vector to filter empty IDs into a
second vector. This Runtime75 slice streams parsed IDs directly into the retained result while
keeping the candidate-property and option declaration ordering intact.

## Implementation

- `indexed_keyboard_entries` resolves the static candidate-property slice once and creates one
  result vector.
- It parses each present property, reserves the direct parsed-option bound, and moves nonempty IDs
  directly into the final vector.
- Candidate-property order, nested option parsing, empty-ID filtering, and all downstream keyboard
  navigation behavior remain unchanged.
- Added a lower property/order/capacity regression and ignored
  `RUNTIME766_INDEXED_KEYBOARD_ENTRY_STREAM_BENCH_V1` Release marker.

## Deterministic work boundary

For `N` parsed option entries, the old intermediate full ID vector is eliminated. The final result
still owns the same ordered nonempty IDs and retains the same single parsing pass per candidate
property. This is allocation-shape evidence only, not allocator, CPU/RSS, or product keyboard
p50/p95/p99 evidence.

## Validation

- TDD source contract was RED before implementation and is GREEN at `3/3`:
  `tools/tests/test_runtime_keyboard_indexed_entry_streaming_performance_contract.py`.
- The combined Runtime/Editor menu, command-palette, keyboard, TreeView, input, and capacity
  contract invocation passed `65/65` in `0.380s` through `python -B -m unittest`.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product percentile evidence remain required before performance acceptance.
