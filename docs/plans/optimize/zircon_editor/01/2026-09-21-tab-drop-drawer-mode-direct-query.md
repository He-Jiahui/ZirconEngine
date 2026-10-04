---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/tab_drop.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/tab_drop.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs
  - tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py
doc_type: milestone-detail
title: Editor864 tab-drop drawer-mode direct query
category: zircon_editor
report_id: Editor864-tab-drop-drawer-mode-direct-query-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor864 - tab-drop drawer-mode direct query

## Scope

Attaching a dragged tab to a drawer cloned the complete Workbench layout only
to determine whether the target drawer had to reopen after the attach command.

## Optimization

- Borrow the active activity window and requested drawer directly.
- Return only its copyable `ActivityDrawerMode`; missing activity windows or
  drawers retain the previous `None` behavior.
- Preserve attach ordering, collapsed-to-pinned reopening, effect merging, and
  all non-drawer drop routes.

## TDD and deterministic performance evidence

The combined contract ran RED before the direct query and GREEN at `7/7`.
Lower semantics cover exact requested-mode projection; ignored marker
`EDITOR864_TAB_DROP_DRAWER_MODE_DIRECT_QUERY_BENCH_V1` exercises 65,536
queries. Full layout clones per drawer attach change from `1` to `0`.

## Local validation

The Editor859–866 focused batch passes `26/26`; the widened non-tooling loader
passes `4183/4183` tests across `986` files in `224.546s`, with zero load
errors, failures, errors, or skips; exact Rustfmt passes. Managed
Runtime→Editor→App validation is asynchronous and supplies no accepted
compiler, Release, allocator, or product percentile result yet.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs` | `88EEE559C83B110C5E4C0A6AABF939C8B1F8EEF12C964F5654F7B1E91AC64284` |
| `zircon_editor/src/ui/retained_host/callback_dispatch/layout/tab_drop.rs` | `0F4C67B9A00846B65FEC01AEA04F40FCF7C0FE484766C852D1F117DFBED53FA5` |
| `zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs` | `548E7A82B75303FD319FD2A07157BC47B0D6721296E3D71C13BCB6F7F736F1EB` |
| `tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py` | `B7CBD7D911F33DE44D92D89A5D65A427D0FA4F7F3838EFA66570DE13CCAD1F40` |

## Acceptance boundary

Keep pending until managed Windows Release executes the lower marker and
provides allocator plus tab-drop p50/p95/p99 evidence. Tooling stays deferred.
