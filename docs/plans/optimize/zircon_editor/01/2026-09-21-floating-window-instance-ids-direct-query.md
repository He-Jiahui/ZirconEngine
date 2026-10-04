---
related_code:
  - zircon_editor/src/ui/workbench/layout/document_node.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs
implementation_files:
  - zircon_editor/src/ui/workbench/layout/document_node.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs
  - tools/tests/test_editor872_floating_window_instance_ids_direct_query_performance_contract.py
doc_type: milestone-detail
title: Editor872 floating-window instance-ID direct query
category: zircon_editor
report_id: Editor872-floating-window-instance-ids-direct-query-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor872 - floating-window instance-ID direct query

## Scope

Native floating-window close cloned the complete Workbench layout, found one
window, then cloned that window's tab identities into the close plan. Large
main-page, drawer, and unrelated floating-window state therefore paid clone
cost on every close request.

## Optimization

- `DocumentNode` now owns reusable depth-first `instance_count` and
  `append_instance_ids` operations.
- The workspace owner borrows the authoritative layout, scans for the exact
  window ID, reserves the target tree's exact identity count, and clones only
  those IDs.
- The Event Controller and Manager delegate the narrow query; the retained
  close caller no longer requests a layout snapshot.
- Missing windows and empty document trees preserve the existing `None`
  behavior and depth-first tab order is unchanged.

## TDD and local evidence

The Editor872 contract was observed RED with two failures and three missing
queries, then GREEN at `5/5`. The focused Editor859–872, Workbench projection,
and floating spatial batch passes `52/52` in `0.040s`; the scoped non-tooling
performance/contract loader passes `4195/4195` tests across `988` files in
`184.584s`, with zero load errors, failures, errors, or skips; exact Rustfmt
passes.
Lower semantics cover depth-first order, empty/missing windows, and both ignored
markers: existing `EDITOR314_FLOATING_WINDOW_INSTANCE_CAPACITY_BENCH_V1` plus
`EDITOR872_FLOATING_WINDOW_INSTANCE_IDS_DIRECT_QUERY_BENCH_V1`. Complete layout
clones per close-ID query change from `1` to `0`.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/layout/document_node.rs` | `9003D0468A22D424BE26877A2165F2422F028563230D6E660807002B68CC25DD` |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs` | `88EEE559C83B110C5E4C0A6AABF939C8B1F8EEF12C964F5654F7B1E91AC64284` |
| `zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs` | `18DE770D9A5E5EA1E79B76B128A3775DC22EF92CBD57658EEB0E4792128E8C58` |
| `zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs` | `044621A0019244241388502208DCD06F2BBA36AF75DBB026F7656516460E7E26` |
| `tools/tests/test_editor872_floating_window_instance_ids_direct_query_performance_contract.py` | `B62EC2EAB59AB1DE9A4D626B4DE078309AB4AA3E4626290ED09AB6C0F57A5DB8` |

## Acceptance boundary

Keep pending until managed current-source Windows compilation, Release markers,
allocator, and native floating-close p50/p95/p99 receipts pass.
