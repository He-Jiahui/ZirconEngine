---
title: Runtime75 Toast Borrowed Setting
category: zircon_runtime
report_id: Runtime75-toast-borrowed-setting-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Toast Borrowed Setting

## Scope

Toast queue synchronization repeatedly reads the current toast, authored message, and presence
settings. Read-only scans formerly cloned each textual setting even when the value was immediately
compared or used to select a borrowed queue entry. This Runtime75 slice keeps those settings
borrowed through scanning and owns only the identifier written back to state.

## Implementation

- Added a borrowed toast setting lookup layered over the existing state/default resolver.
- Used it for queue scan selection and current-toast presence checks.
- Reworked authored-message fallback so an existing current ID or authored message is cloned only
  once at the state publication boundary.
- Preserved queue precedence, authored message/text fallback, popup behavior, and expiry
  semantics; retained owned IDs where mutation crosses the state borrow boundary.
- Added lower synchronization coverage and the ignored
  RUNTIME772_TOAST_BORROWED_SETTING_BENCH_V1 Release marker.

## Deterministic work boundary

Read-only Toast synchronization removes transient setting clones from queue scanning and presence
checks; an authored message/current ID is owned only when publishing the retained current ID. This
is allocation-shape evidence only, not allocator, CPU/RSS, or product notification latency
p50/p95/p99 evidence.

## Validation

- The TDD source contract was RED before implementation and is GREEN at 5/5:
  tools/tests/test_runtime_toast_borrowed_setting_performance_contract.py.
- The combined Runtime/Editor command-palette, keyboard, menu, TreeView, TextInput, selection,
  collection, table, Toast, and adjacent capacity-contract invocation passed 91/91 in 0.415s
  through python -B -m unittest.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product percentile evidence remain required before performance acceptance.
