---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs
  - tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py
doc_type: milestone-detail
title: Editor865 floating-window existence direct query
category: zircon_editor
report_id: Editor865-floating-window-exists-direct-query-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor865 - floating-window existence direct query

## Scope

After closing all requested tabs and recomputing, native-window close handling
cloned the entire Workbench layout to test whether one floating-window ID still
existed.

## Optimization

- Query the authoritative session layout under one borrowed lock and scan only
  floating-window identities.
- Preserve exact case-sensitive `MainPageId` equality and the existing
  keep-shown versus hide response.
- Leave tab close dispatch, error reporting, and recompute ordering unchanged.

## TDD and deterministic performance evidence

The combined contract ran RED then GREEN at `7/7`. Lower semantics cover exact
and case-mismatched IDs; ignored marker
`EDITOR865_FLOATING_WINDOW_EXISTS_DIRECT_QUERY_BENCH_V1` exercises 65,536
queries over 128 windows. Full layout clones per close confirmation change
from `1` to `0`.

## Local validation

The Editor859–866 focused batch passes `26/26`; the widened non-tooling loader
passes `4183/4183` tests across `986` files in `224.546s`, with zero load
errors, failures, errors, or skips; exact Rustfmt passes. The managed
three-package batch remains asynchronous and has not supplied compiler,
Release, allocator, or product percentile acceptance.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs` | `88EEE559C83B110C5E4C0A6AABF939C8B1F8EEF12C964F5654F7B1E91AC64284` |
| `zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs` | `18DE770D9A5E5EA1E79B76B128A3775DC22EF92CBD57658EEB0E4792128E8C58` |
| `zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs` | `548E7A82B75303FD319FD2A07157BC47B0D6721296E3D71C13BCB6F7F736F1EB` |
| `tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py` | `B7CBD7D911F33DE44D92D89A5D65A427D0FA4F7F3838EFA66570DE13CCAD1F40` |

## Acceptance boundary

Keep pending until managed Windows Release executes the lower marker and
provides allocation plus native-window close p50/p95/p99 evidence.
