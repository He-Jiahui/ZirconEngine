---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads/editor_panes.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads/editor_panes.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads/editor_panes.rs
  - tools/tests/test_editor873_editor_pane_identity_projection_performance_contract.py
doc_type: milestone-detail
title: Editor873 editor-pane identity projection
category: zircon_editor
report_id: Editor873-editor-pane-identity-projection-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor873 - editor-pane identity projection

## Scope

Full retained-host lifecycle recompute cloned every open `ViewInstance` whenever
either UI Asset or Animation pane payloads were visible. The collector then
needed only each matching instance ID, so unrelated descriptors and the title,
payload, dirty, and host fields of matching instances were copied without being
consumed.

## Optimization

- The authoritative workspace owner now performs one borrowed session scan and
  projects UI Asset and Animation instance IDs into separate vectors.
- Each vector reserves the open-instance upper bound only after its first
  matching descriptor; disabled or no-match families retain zero capacity.
- Manager and Event Controller expose the narrow query without exposing session
  storage.
- Retained pane collection consumes the two targeted ID vectors and no longer
  branches on or owns complete `ViewInstance` values.
- BTree identity order and the existing visibility gates remain unchanged.

## TDD and local evidence

The Editor873 source contract was observed RED with three failures and two
missing-query errors, then GREEN at `5/5`. The focused Editor860–873,
Workbench, payload-cache batch passes `47/47` in `0.107s`; the adjacent native
projection batch also passes `38/38` in `0.032s`. The current non-Tooling
performance/contract loader passes `4204/4204` tests across `990` files in
`277.584s`, with zero load errors, failures, errors, or skips. Fixture-emitted
Cargo command lines are not managed compile evidence. Exact Rustfmt passes.

Lower semantics cover both pane families, BTree order, independent visibility
gates, and disabled-family zero capacity. The ignored
`EDITOR873_EDITOR_PANE_IDENTITY_PROJECTION_BENCH_V1` marker records complete
view-record clones changing from `queries × open_views` to `0`; only matching
identities are cloned.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs` | `88EEE559C83B110C5E4C0A6AABF939C8B1F8EEF12C964F5654F7B1E91AC64284` |
| `zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs` | `E3E8A6715EF648B2EF2C560FA299C60A9B8B39C3D1021D7D06C614F544DF8DFE` |
| `zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads/editor_panes.rs` | `393C40A653EFAF5DCD2D8887EA449055A2EE0FC595224F195AE692781EF2FFF2` |
| `zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs` | `044621A0019244241388502208DCD06F2BBA36AF75DBB026F7656516460E7E26` |
| `tools/tests/test_editor873_editor_pane_identity_projection_performance_contract.py` | `DE79A2F226F0033A33639A2F479580EA9FCAFCE74AF6E9A621D9A3D32392A43F` |

## Acceptance boundary

Keep pending until managed current-source Windows compilation, Release marker,
allocator, and full lifecycle pane-recompute p50/p95/p99 receipts pass.
