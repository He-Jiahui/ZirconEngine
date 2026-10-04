---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs
  - tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py
doc_type: milestone-detail
title: Editor870 main-window close identity projection
category: zircon_editor
report_id: Editor870-main-window-close-identity-projection-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor870 - main-window close identity projection

## Scope and optimization

Main-window close cloned all open view records before discovering whether a
dirty prompt was needed. The clean close path now performs no view projection;
only a required prompt asks the workspace owner for ordered IDs, cloning no
titles, payloads, descriptors, or hosts. Save-block, dirty-project, prompt, and
hide/keep semantics remain unchanged.

## TDD and local evidence

The combined RED→GREEN contract passes `7/7`; the focused Editor859–871 plus
Workbench batch passes `43/43` in `0.041s`. Lower tests cover ordered identity
projection; the widened explicit performance/contract loader passes `4441/4441`
across `1127` files in `290.084s`, with zero load errors, failures, errors, or
skips; ignored marker
`EDITOR870_MAIN_CLOSE_IDENTITY_PROJECTION_BENCH_V1` exercises 16,384 projections
over 128 views. Clean close changes all-view cloning to zero, and dirty close
changes full-record clones to identity-only clones. Exact Rustfmt passes.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs` | `88EEE559C83B110C5E4C0A6AABF939C8B1F8EEF12C964F5654F7B1E91AC64284` |
| `zircon_editor/src/ui/retained_host/app/native_window_close.rs` | `E11232CFC5DDF761E2EB309AA0788C22DEF08A7EBF09C80E1776137D7D8907BA` |
| `zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs` | `044621A0019244241388502208DCD06F2BBA36AF75DBB026F7656516460E7E26` |
| `tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py` | `A6E1991F5CE923E7F413AD1E4FC02954074B640681BC4364EE553C2A89049ED2` |

## Acceptance boundary

Keep pending until managed Windows Release, allocator, lower marker, and clean
and dirty close p50/p95/p99 receipts pass.
