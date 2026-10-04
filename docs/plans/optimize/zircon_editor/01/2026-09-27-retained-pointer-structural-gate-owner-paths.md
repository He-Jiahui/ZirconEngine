---
related_code:
  - zircon_editor/src/tests/ui/boundary/template_assets/invalidation.rs
  - zircon_editor/src/tests/host/retained_drawer_resize/surface_contract.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
tests:
  - zircon_editor/src/tests/ui/boundary/template_assets/invalidation.rs
  - zircon_editor/src/tests/host/retained_drawer_resize/surface_contract.rs
  - zircon_editor/src/ui/retained_host/app/tests/projection_geometry.rs
  - zircon_editor/src/tests/host/retained_drawer_resize/pointer_bridge.rs
doc_type: milestone-detail
title: Editor1023 retained pointer structural gate owner paths
category: zircon_editor
report_id: Editor1023-retained-pointer-structural-gate-owner-paths-2026-09-27
date: 2026-09-27
session_id: astra-optimize-20260926-batch-a
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: product_gate_pending
---

# Editor1023 retained pointer structural gate owner paths

## Failure evidence

The existing pointer gate read ten `app/*.rs` parent modules as though each
contained its pointer handler. Eight now contain module bindings while the
actual `use_committed_pointer_layout` calls live in child modules. The gate
also read obsolete parent paths for lifecycle dirty marking, UI Asset Editor
actions, and dispatch effects. Its dirty-write scan treated only
`host_lifecycle.rs` as the lifecycle owner, so it would reject the legitimate
`host_lifecycle/invalidation_bridge/dirty_flags.rs` writes. The drawer resize
gate likewise looked for capture and callback bodies in parent modules and for
the retired `Arc::new(DragTargetFrames` and `dispatch_input_event(` source
shapes. These are source-confirmed failures of old structural assertions;
no pre-repair Rust test execution is claimed.

The exact preimages were preserved before editing. Their SHA-256 values are
`fec3a1bc39674f617c7511fb7897d230973e3c807650f88b0c9c7a5cfa3008d5`
for `invalidation.rs` and
`b5549c94a3bbddc5b973248211f936dd30b4843bf136699f20601bd4d6866ddd`
for `surface_contract.rs`. The first file was an existing untracked test leaf;
its original bytes remain in the Batch S operational preimage.

## Repair

- Resolve each of the ten pointer groups from its parent and recursive child
  Rust owners. Require a production call to the committed pointer layout and
  forbid `recompute_if_dirty(` and `chrome_snapshot(` in every group owner.
  Check the cached entrypoint body in `host_lifecycle/tick.rs`.
- Check dirty marking in `invalidation_bridge/dirty_marking.rs`, actual flag
  writes in `dirty_flags.rs`, UI Asset Editor actions in `actions.rs`, and both
  scoped and unscoped effect invalidation in `dispatch_effects.rs`. The two
  recursive dirty-write scans exempt only the exact legitimate `dirty_flags.rs`
  owner (and `event_bridge.rs` where it owns its own effect state), retaining
  the negative constraints everywhere else.
- Check resize callback wiring in `globals/ui_context.rs` and
  `callback_wiring/host_shell/drag_resize.rs`; check capture geometry in
  `workspace_docking/drawer_resize/capture.rs` and move/release methods in
  `movement.rs`. Preserve the legacy resize-frame and callback exclusions.
  Check drag geometry's current `ArcSwap` load/store publication and lock
  exclusions, plus resize input's diagnostics-mode dispatch and reply capture.

No production binding was changed. Existing real-host repeated toolbar-click
and resize-capture behavior tests remain wired for the managed Editor lib lane;
this slice adds no unproved pending-invalidation behavior claim.

## Validation and remaining gate

The pre-repair source replay identifies eight missing parent positives and the
obsolete lifecycle, resize, drag, and dispatch markers (9/9 expected-failure
checks). After repair, 37/37 source-shape checks passed against current
production owners, with zero
forbidden dirty writes outside the exact owners. Scoped `rustfmt --check`,
owner-path existence, whitespace, and inverse-diff checks are recorded in the
Batch S operational evidence. This is static evidence only.

Managed `zircon_editor --lib` execution, including the existing real-host
pointer and resize tests, remains pending. The Editor01 product requirement to
avoid pointer-time slow-path rebuilds and its CPU, allocator, RSS, and latency
gates remain open; updating source assertions does not establish a performance
improvement or a product pass.
