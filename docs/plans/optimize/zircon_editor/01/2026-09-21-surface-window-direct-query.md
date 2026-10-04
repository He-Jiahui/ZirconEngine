---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window/focus.rs
  - zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window/focus.rs
  - zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_surface_window_query_tests.rs
  - tools/tests/test_editor861_surface_window_direct_query_performance_contract.py
doc_type: milestone-detail
title: Editor861 focused surface-window direct query
category: zircon_editor
report_id: Editor861-surface-window-direct-query-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor861 - focused surface-window direct query

## Scope

Native focus notifications provide one surface key. The retained host previously
built the full Chrome/Workbench snapshot, projected every floating window, and
then scanned that projection only to recover the matching `MainPageId`.

## Optimization

- Route the lookup through `EditorHostEventController` and `EditorManager` to the
  authoritative `EditorUiHost` workspace owner.
- Borrow the current session layout, scan only `floating_windows`, and clone the
  one matched window ID at the retained-host boundary.
- Preserve exact case-sensitive surface-key equality, the special `main`
  handling in the caller, and missing-window `None` behavior.
- Delete the now-unreferenced snapshot lookup helper and its orphaned import.

## TDD and deterministic performance evidence

The source/model contract was run RED before the direct owner query existed and
then GREEN at `4/4`. The lower regression covers exact, case-mismatched, and
missing keys. The ignored `EDITOR861_SURFACE_WINDOW_DIRECT_QUERY_BENCH_V1`
marker exercises 65,536 last-window lookups across 128 windows. The structural
target changes full Chrome snapshot builds per focus notification from `1` to
`0`.

## Local validation

- The combined Editor859-862, related Workbench projection, and Runtime08d/861/862
  source-contract batch passes `43/43` in `0.036s` with zero failures, errors, or
  skips.
- The one-process current-tree explicit performance-or-contract loader covers
  `974` non-tooling files and passes `4121/4121` tests in `221.095s`, with zero
  load errors, failures, errors, or skips.
- Rustfmt is checked over the changed owner chain, retained caller, and lower
  regression. The initial write-mode formatter encountered a transient Windows
  mapped-file lock only on unrelated Editor862 files; read-only exact-file
  checks distinguish that environment event from source validity.
- No local Cargo command or coordinator poll was started. Managed Windows
  Release compilation, lower-test execution, allocator evidence, and native
  focus p50/p95/p99 measurements remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs` | `88EEE559C83B110C5E4C0A6AABF939C8B1F8EEF12C964F5654F7B1E91AC64284` |
| `zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window/focus.rs` | `C7A46523F90EFBE588CF6A97BE21DFEC1F86AF418DF6051528F6AA85A59774F2` |
| `zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs` | `B6DEA6E3BF9C749BC6FEF93853A535A678B21E08E35C524B53719A1CF6B91831` |
| `zircon_editor/src/ui/host/workspace_state_surface_window_query_tests.rs` | `825BB0691E42C38B6A8CEFFFD731636EB352DE3CE3E3236E7769FFD52E2573F1` |
| `tools/tests/test_editor861_surface_window_direct_query_performance_contract.py` | `D35F4F07C9E0D2593F61E81C361A3F0EB78EC2534B328BA620D055B3587D5FEC` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Editor tree,
runs the lower regression and ignored marker, and supplies allocator plus native
focus p50/p95/p99 evidence. Tooling production stays deferred for the later Rust
migration.
