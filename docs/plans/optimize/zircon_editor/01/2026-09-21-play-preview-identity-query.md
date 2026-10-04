---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs
  - tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py
doc_type: milestone-detail
title: Editor868 play-preview identity query
category: zircon_editor
report_id: Editor868-play-preview-identity-query-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor868 - Play Preview identity query

## Scope and optimization

Focusing Play Preview previously cloned every open `ViewInstance`, scanned the
owned copies for the game descriptor, then retained only one ID. The workspace
owner now scans the borrowed ordered map and clones only the first matching
identity. Existing first-match order, open-on-miss, focus, and restore behavior
remain unchanged.

## TDD and local evidence

The combined source contract ran RED then passes `7/7`; the focused
Editor859–871/Workbench batch passes `43/43` in `0.041s`; the widened explicit
performance/contract loader passes `4441/4441` across `1127` files in
`290.084s`, with zero load errors, failures, errors, or skips. Lower tests cover
ordered first match and missing descriptors, and ignored marker
`EDITOR868_PLAY_PREVIEW_IDENTITY_QUERY_BENCH_V1` exercises 65,536 queries over
128 views. Modeled full-instance clones change from `queries × open_views` to
zero, with at most one matching ID clone per query. Exact Rustfmt passes.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/workbench/shell_state.rs` | `AB185EEE7254D9966A8385C1CD5219F9EB4AC0E1B4BB631E5CE4752EE1D927E8` |
| `zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs` | `044621A0019244241388502208DCD06F2BBA36AF75DBB026F7656516460E7E26` |
| `tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py` | `A6E1991F5CE923E7F413AD1E4FC02954074B640681BC4364EE553C2A89049ED2` |

## Acceptance boundary

Keep pending until managed Windows Release runs the lower marker and supplies
allocator plus Play Preview interaction p50/p95/p99 evidence.
