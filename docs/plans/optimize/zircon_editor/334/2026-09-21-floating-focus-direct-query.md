---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/floating_window/dispatch.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/floating_window/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/floating_window/dispatch.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
  - docs/plans/optimize/zircon_editor/334/2026-08-30-floating-focus-single-scan.md
tests:
  - zircon_editor/src/ui/host/workspace_state_floating_focus_query_tests.rs
  - tools/tests/test_editor860_floating_focus_direct_query_performance_contract.py
  - tools/tests/test_editor_retained_workbench_contribution_projection_contract.py
doc_type: milestone-detail
title: Editor860 floating-window focus direct query
category: zircon_editor
report_id: Editor860-floating-focus-direct-query-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor860 - floating-window focus direct query

## Scope

Focusing a callback's floating source window previously built a complete Chrome
snapshot, projected command context, and constructed the complete Workbench view
model before selecting one target tab. Editor334 had already reduced the final
model scan to one pass, but the much larger projection remained on every focus
transition.

## Optimization

- Query the authoritative `WorkbenchLayout` directly through the Host → Manager
  → Event Controller ownership chain.
- Find only the requested floating window and traverse its `DocumentNode` tree
  once, preserving the exact `focused → first active → first tab` priority.
- Return one owned `ViewInstanceId` only at the command boundary; the layout and
  all intermediate tab identities remain borrowed.
- Compile the legacy model-resolution helper only for its retained regression
  tests; full Workbench projections still use real contribution and capability
  state where presentation actually requires them.

## TDD and deterministic performance evidence

The Editor860 source/model contract was run RED before the direct layout query
and then GREEN at `4/4`. Lower Rust regressions cover nested focused, active,
first-tab, missing-window, and empty-window behavior. The ignored
`EDITOR860_FLOATING_FOCUS_DIRECT_QUERY_BENCH_V1` marker exercises 65,536 direct
queries. The deterministic structural target changes Chrome plus Workbench-model
builds per focus dispatch from `1` to `0`; the remaining work is one requested-
window lookup and one traversal of that window's tabs.

## Local validation

- Editor859 and Editor860 focused contracts pass `8/8` in one process.
- The initially failing retained contribution-projection contract was traced to
  a stale requirement that every floating-focus dispatch build a full model.
  Its owner contract now distinguishes complete presentation projection from
  direct authoritative layout lookup and passes `3/3`.
- The combined Editor859/860, related Workbench projection, and Runtime08d/861/862
  source-contract batch passes `35/35` in `0.018s` with zero failures, errors, or
  skips.
- The current-tree one-process explicit performance-or-contract loader covers
  `974` non-tooling files and passes `4121/4121` tests in `221.095s`, with zero
  load errors, failures, errors, or skips.
- One exact-file Rustfmt check covers all touched Rust owners and exits `0`.
- No local Cargo command or coordinator poll was started. Managed Windows
  Release compilation, lower-test execution, allocator evidence, and floating-
  focus product p50/p95/p99 measurements remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs` | `88EEE559C83B110C5E4C0A6AABF939C8B1F8EEF12C964F5654F7B1E91AC64284` |
| `zircon_editor/src/ui/retained_host/callback_dispatch/layout/floating_window/mod.rs` | `1B9653720640EF4D57FEFF7FAE00085666CEDE4A131E2243771F19AED402E594` |
| `zircon_editor/src/ui/retained_host/callback_dispatch/layout/floating_window/dispatch.rs` | `B12095169147887310C7AB557991B01D8C667B74FDE57345F6A79BD9CE9EDF36` |
| `zircon_editor/src/ui/host/workspace_state_floating_focus_query_tests.rs` | `18396C16832D6DF581C15590116456995E37633D2FFE771CA78FFD6EB2C8E091` |
| `tools/tests/test_editor860_floating_focus_direct_query_performance_contract.py` | `2FCED26E2E80FC58750833168B9FBA4716E0F94DE28A474F203367866BA8563F` |
| `tools/tests/test_editor_retained_workbench_contribution_projection_contract.py` | `38B435693AF650CCD0E879D948D13A62BEAF4D61BC69548ABBAD2C5C0391208E` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Editor tree,
runs the lower priority regressions and ignored marker, and supplies allocator
plus floating-window focus p50/p95/p99 evidence. Tooling production stays
deferred for the later Rust migration.
