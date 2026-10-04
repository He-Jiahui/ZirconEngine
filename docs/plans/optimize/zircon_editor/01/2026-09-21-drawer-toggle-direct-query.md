---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/drawer_toggle.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/drawer_toggle.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs
  - tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py
doc_type: milestone-detail
title: Editor863 drawer-toggle direct state query
category: zircon_editor
report_id: Editor863-drawer-toggle-direct-query-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor863 - drawer-toggle direct state query

## Scope

Every retained drawer toggle cloned the complete `WorkbenchLayout` to read one
drawer mode, active tab, and expanded-region predicate. Large floating-window,
split, and drawer layouts therefore paid unrelated clone cost on an interaction
hot path.

## Optimization

- Route one query through Event Controller → Manager → authoritative
  `EditorUiHost` workspace state.
- Borrow the active drawer map once and return only
  `(mode, is_active, region_was_expanded)`.
- Preserve missing active-window/drawer diagnostics, region sibling semantics,
  active-tab comparison, collapse behavior, and shell-layout reuse decisions.

## TDD and deterministic performance evidence

The combined source contract ran RED with five failures and two missing-query
errors before implementation, then GREEN at `7/7`. The lower regression covers
active/region and missing-authority behavior; ignored marker
`EDITOR863_DRAWER_TOGGLE_DIRECT_QUERY_BENCH_V1` exercises 65,536 queries.
The structural target changes full layout clones per toggle from `1` to `0`.

## Local validation

- Editor859–866 plus retained Workbench contracts pass `26/26` in one process.
- The widened non-tooling contract loader passes `4183/4183` tests across `986`
  files in `224.546s`, with zero load errors, failures, errors, or skips.
- Exact Rustfmt covers the shared owner/delegates, four callers, and lower owner.
- Managed Runtime→Editor→App validation was resubmitted asynchronously as one
  batch; no compiler result, Release marker, allocator, or product percentile
  receipt is claimed here.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs` | `88EEE559C83B110C5E4C0A6AABF939C8B1F8EEF12C964F5654F7B1E91AC64284` |
| `zircon_editor/src/ui/retained_host/callback_dispatch/layout/drawer_toggle.rs` | `57BAAC2758F2C5587FEF9F77AADD4E9B539404D7C9E56E8B526669C1EB882BD7` |
| `zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs` | `548E7A82B75303FD319FD2A07157BC47B0D6721296E3D71C13BCB6F7F736F1EB` |
| `tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py` | `B7CBD7D911F33DE44D92D89A5D65A427D0FA4F7F3838EFA66570DE13CCAD1F40` |

## Acceptance boundary

Keep `implementation_complete` / `managed_validation_pending` until managed
Windows Release compilation runs the lower marker and supplies allocation plus
drawer-interaction p50/p95/p99 evidence. Tooling production remains deferred.
