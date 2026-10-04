---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs
  - tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py
doc_type: milestone-detail
title: Editor871 extension-retirement identity projection
category: zircon_editor
report_id: Editor871-extension-retirement-identity-projection-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor871 - extension-retirement identity projection

## Scope and optimization

Extension retirement cloned every open view record before filtering instances
owned by retiring descriptors. It now asks the workspace owner for matching
IDs only, preserving BTree order, complete close preflight, all-or-error close
behavior, and descriptor unregister order. Empty/no-match projections retain
zero vector capacity.

## TDD and local evidence

The combined RED→GREEN contract passes `7/7`; the focused Editor859–871 plus
Workbench batch passes `43/43` in `0.041s`. Lower tests cover descriptor-set
filtering, order, and zero-capacity empty input; the widened explicit
performance/contract loader passes `4441/4441` across `1127` files in
`290.084s`, with zero load errors, failures, errors, or skips; ignored marker
`EDITOR871_EXTENSION_RETIRE_IDENTITY_PROJECTION_BENCH_V1` exercises 16,384
projections over 128 mixed views. Full-record clones change from every open
view to zero; only matching identities are cloned. Exact Rustfmt passes.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_extension_views.rs` | `175333BF38F7F9BD404A73CE01BDD15261EDAD84C4FABB87670F3302C2825F29` |
| `zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs` | `044621A0019244241388502208DCD06F2BBA36AF75DBB026F7656516460E7E26` |
| `tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py` | `A6E1991F5CE923E7F413AD1E4FC02954074B640681BC4364EE553C2A89049ED2` |

## Acceptance boundary

Keep pending until managed Windows Release, allocator, lower marker, and
extension retirement p50/p95/p99 receipts pass. Tooling production is deferred.
