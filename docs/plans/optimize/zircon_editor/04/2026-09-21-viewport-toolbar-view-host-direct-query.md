---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/size.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/size.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs
  - tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py
doc_type: milestone-detail
title: Editor866 viewport toolbar view-host direct query
category: zircon_editor
report_id: Editor866-viewport-toolbar-view-host-direct-query-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor866 - viewport toolbar view-host direct query

## Scope

Viewport toolbar size resolution cloned every open `ViewInstance`, then kept
only the instance matching one surface key to inspect its `ViewHost`.

## Optimization

- Borrow current session instances, scan exact keys, and clone only the matched
  small `ViewHost` enum.
- Preserve floating-window early return, document/drawer/exclusive sizing,
  absent-instance fallback, and the minimum one-pixel width.
- Keep frame authority in the existing Workbench/native-window bridges.

## TDD and deterministic performance evidence

The combined contract ran RED then GREEN at `7/7`. Lower semantics cover exact
and case-mismatched keys; ignored marker `EDITOR866_VIEW_HOST_DIRECT_QUERY_BENCH_V1`
exercises 65,536 queries over 128 views. Modeled view-instance clones change
from `queries × open_views` to `0`; the matched host is cloned once.

## Local validation

The Editor859–866 focused batch passes `26/26`; the widened non-tooling loader
passes `4183/4183` tests across `986` files in `224.546s`, with zero load
errors, failures, errors, or skips; exact Rustfmt passes. Managed Windows
compile/Release, allocator, and viewport product p50/p95/p99 evidence remain
pending in the asynchronous combined batch.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs` | `88EEE559C83B110C5E4C0A6AABF939C8B1F8EEF12C964F5654F7B1E91AC64284` |
| `zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/size.rs` | `951C5EEE7B079A8761DF0729CA1CDEC14A8D90B64410F860981C212253DBF163` |
| `zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs` | `548E7A82B75303FD319FD2A07157BC47B0D6721296E3D71C13BCB6F7F736F1EB` |
| `tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py` | `B7CBD7D911F33DE44D92D89A5D65A427D0FA4F7F3838EFA66570DE13CCAD1F40` |

## Acceptance boundary

Keep pending until managed Windows Release executes the lower marker and
supplies allocator plus viewport-toolbar p50/p95/p99 evidence. Tooling stays
deferred.
