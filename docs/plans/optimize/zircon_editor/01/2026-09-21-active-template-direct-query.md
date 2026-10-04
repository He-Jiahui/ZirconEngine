---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_active_template_query_tests.rs
  - tools/tests/test_editor859_active_template_direct_query_performance_contract.py
doc_type: milestone-detail
title: Editor859 active activity-window template direct query
category: zircon_editor
report_id: Editor859-active-template-direct-query-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor859 - active activity-window template direct query

## Scope

Retained workbench controls repeatedly check whether the active activity window
owns one template document. The production predicate previously called
`chrome_snapshot()`, which cloned Editor data, layout, view instances, and the
capability-filtered descriptor set before rebuilding the complete Workbench
snapshot for one boolean result.

## Optimization

- Route the predicate through `EditorHostEventController` and `EditorManager` to
  the authoritative `EditorUiHost` workspace owner.
- Borrow the current session layout, resolve only the active Workbench or
  exclusive-page descriptor, and query that descriptor directly from the
  registry.
- Preserve missing-page/window/instance behavior and the registry capability
  filter used by full snapshot construction.
- Leave the test-only owned document-ID accessor and full presentation
  projection unchanged.

## TDD and deterministic performance evidence

The source/model contract was run RED before the direct owner query existed and
then GREEN at `4/4`. Lower Rust regressions cover Workbench and exclusive-page
resolution plus missing authority, and the ignored
`EDITOR859_ACTIVE_TEMPLATE_DIRECT_QUERY_BENCH_V1` marker exercises 65,536
borrowed queries. The deterministic structural target changes full Chrome
snapshot builds per production predicate from `1` to `0`.

## Local validation

- Editor859 and Editor860 focused contracts pass `8/8` in one process.
- The repaired contribution-projection contract passes `3/3`: complete retained
  projections still require real contribution/capability input, while floating
  focus is explicitly a direct layout query.
- The combined Editor859/860, related Workbench projection, and Runtime08d/861/862
  source-contract batch passes `35/35` in `0.018s` with zero failures, errors, or
  skips.
- The current-tree one-process explicit performance-or-contract loader covers
  `974` non-tooling files and passes `4121/4121` tests in `221.095s`, with zero
  load errors, failures, errors, or skips.
- One exact-file Rustfmt check covers the shared host owner, delegates, retained
  callers, and both lower regression owners and exits `0`.
- No local Cargo command or coordinator poll was started. Managed Windows
  Release compilation, lower-test execution, allocator evidence, and retained
  workbench product p50/p95/p99 measurements remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs` | `88EEE559C83B110C5E4C0A6AABF939C8B1F8EEF12C964F5654F7B1E91AC64284` |
| `zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs` | `B6DEA6E3BF9C749BC6FEF93853A535A678B21E08E35C524B53719A1CF6B91831` |
| `zircon_editor/src/ui/host/workspace_state_active_template_query_tests.rs` | `A5C93343538869727B527A5792871B5FE0B6EF123C4AE70C30FC2B57D387C96D` |
| `tools/tests/test_editor859_active_template_direct_query_performance_contract.py` | `EE58D76611AC846820C79047DD532BF727F43E45D79A1A466A46521081E5E044` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Editor tree,
runs the lower semantic regressions and ignored marker, and supplies allocator
plus active-workbench interaction p50/p95/p99 evidence. Tooling production stays
deferred for the later Rust migration.
