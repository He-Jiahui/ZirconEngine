---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/179-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-source-review.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
related_code:
  - zircon_editor/src/core/play/controller/preview_routing.rs
  - zircon_editor/src/core/play/controller/runtime_ownership.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/runtime_event_consumers.rs
tests:
  - tools/tests/test_editor_game_viewport_input_contract.py
  - tools/tests/test_editor_pie_preview_frame_contract.py
  - tools/tests/test_editor_embedded_play_session_contract.py
---

# Play Viewport Source-Owner Contract Repair

Play preview input, simulation-camera routing, preview-frame capture, and
terminal backend retirement moved from aggregate controller/host files into
focused child owners. The static tests now bind to those canonical modules and
also assert their declarations from the aggregate roots, preserving the hard
cut without reintroducing a compatibility facade.

The contracts retain identity-bound gateway dispatch, Play-versus-Simulate
input separation, default-viewport frame capture, and detach-before-retirement
ordering. This is a source-contract repair, not a product timing claim.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| E35 | Rebind play, PIE, and embedded-session static contracts to canonical child owners | implemented_pending_validation | The focused three-module batch passed 15/15; Python compilation and scoped diff checks passed. Managed Rust compilation remains blocked at immutable admission by dirty external worktree `E:\\Git\\zr_vm`; no Release p50/p95/p99 claim is made. |

## Follow-up: Editor04 contract-owner drift (2026-09-12)

Three adjacent Editor04 source contracts still parsed aggregate files after
the Play host/controller split. The hierarchy-order contract now reads
`runtime_event_consumers.rs`, the terminal controller contract reads
`core/play/controller/runtime_ownership.rs`, and the Inspector contract checks
the field writability expression (`field.writable && field.serializable`)
instead of the retired fixed literal. These are test-only path/expectation
updates; production behavior is unchanged.

The pre-change focused run was RED with one source-slice error and one stale
Inspector assertion. After the repair, the three focused modules pass 17/17;
the broader Editor04 history-plus-owner batch passes 35/35. Python bytecode
compilation and scoped `git diff --check` pass. Updated test hashes are:

- `test_editor04_play_hierarchy_domain_contract.py` —
  `2f2fbecce8af2500ce1d1d9f84606d03eda745ee`
- `test_editor04_play_inspector_domain_contract.py` —
  `ea952577e98df84394e78a0ff4fd766087802e47`
- `test_editor04_play_session_controller_contract.py` —
  `a4a09ad8ee76a2e1b6c7efe50f77c2a5f2d693cf`

Managed Cargo, Windows product, and visual gates remain pending because the
external `E:/Git/zr_vm` worktree is dirty; this follow-up remains
`implemented_pending_validation`.
