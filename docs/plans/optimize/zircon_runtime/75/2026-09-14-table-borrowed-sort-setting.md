---
title: Runtime75 Table Borrowed Sort Setting
category: zircon_runtime
report_id: Runtime75-table-borrowed-sort-setting-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Table Borrowed Sort Setting

## Scope

Table sort and column-width events frequently inspect textual UiValue payloads and retained sort
settings. The reducer formerly cloned those strings even when the value was only compared or passed
to a function that already accepts a borrowed string. This Runtime75 slice propagates borrowed
text through the event, sort-setting, and column-width payload paths.

## Implementation

- Added borrowed String/Enum and retained-setting helpers for read-only table paths.
- Passed borrowed event values, current-sort comparison values, mode settings, and column-width
  fields directly into normalization and comparison.
- Changed column-width payload projection to return a borrowed field and delayed ownership until
  the state write requires it.
- Retained one current-column clone in the sort-direction transition because that state-derived
  string must remain valid while the same state map is mutably updated and rows are sorted.
- Preserved all aliases, server/client mode checks, row ordering, and sort-model publication.
- Added lower semantic coverage and the ignored
  RUNTIME771_TABLE_BORROWED_SORT_SETTING_BENCH_V1 Release marker.

## Deterministic work boundary

Read-only event payload, comparison, mode, and column-width handling no longer clones textual
values. The sort-direction transition retains its necessary current-column clone before mutating
the same state map; ownership is also created at retained-state and sort-model write boundaries.
This is allocation-shape evidence only, not allocator, CPU/RSS, or product table latency
p50/p95/p99 evidence.

## Validation

- The TDD source contract was RED before implementation and is GREEN at 4/4:
  tools/tests/test_runtime_table_borrowed_sort_setting_performance_contract.py.
- The combined Runtime/Editor command-palette, keyboard, menu, TreeView, TextInput, selection,
  collection, table, Toast, and adjacent capacity-contract invocation passed 91/91 in 0.415s through
  python -B -m unittest.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product percentile evidence remain required before performance acceptance.
