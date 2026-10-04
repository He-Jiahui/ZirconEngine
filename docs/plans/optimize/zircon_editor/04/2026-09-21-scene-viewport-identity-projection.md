---
related_code:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs
  - tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py
doc_type: milestone-detail
title: Editor869 scene-viewport identity projection
category: zircon_editor
report_id: Editor869-scene-viewport-identity-projection-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor869 - Scene viewport identity projection

## Scope and optimization

Every dirty render submission cloned all open `ViewInstance` records before
filtering `editor.scene` identities for viewport retirement. The authoritative
workspace owner now projects only matching IDs in BTree order. The no-match
path keeps zero vector capacity, while the first match reserves the bounded
open-view upper bound once. Viewport retention and submission behavior remain
unchanged.

## TDD and local evidence

The combined RED→GREEN contract passes `7/7`; the focused Editor859–871 plus
Workbench batch passes `43/43` in `0.041s`. Lower tests cover filtered order and
zero-capacity misses; the widened explicit performance/contract loader passes
`4441/4441` across `1127` files in `290.084s`, with zero load errors, failures,
errors, or skips; ignored marker
`EDITOR869_SCENE_VIEW_IDENTITY_PROJECTION_BENCH_V1` exercises 16,384 projections
over 128 mixed views. Full `ViewInstance` clones change from all open views per
render pass to zero; only matching IDs are cloned. Exact Rustfmt passes.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/workspace_state.rs` | `608C8E4F1E534911D286F6B2744DFD4597C74D0591792B389B72A26740A2F9A2` |
| `zircon_editor/src/ui/host/editor_manager_workspace.rs` | `7F38A19979F3F16EE5E920EDCA3C316452A4CFA18FFEAD0CD0642196A53C872E` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs` | `88EEE559C83B110C5E4C0A6AABF939C8B1F8EEF12C964F5654F7B1E91AC64284` |
| `zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs` | `E4C23C2848CCE075C71285CB91D41E7AE6C1578994D009363CCCF3A05B51E447` |
| `zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs` | `044621A0019244241388502208DCD06F2BBA36AF75DBB026F7656516460E7E26` |
| `tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py` | `A6E1991F5CE923E7F413AD1E4FC02954074B640681BC4364EE553C2A89049ED2` |

## Acceptance boundary

Keep pending until managed Windows Release, allocator, lower marker, and Scene
viewport retirement/render p50/p95/p99 receipts pass.
