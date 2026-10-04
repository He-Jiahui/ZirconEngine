---
related_code:
  - zircon_editor/src/tests/host/retained_window/native_mode.rs
  - zircon_editor/src/tests/host/retained_window/activity_rail_template_boundary.rs
  - zircon_editor/src/tests/host/retained_tab_drag/surface_contract.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/surface_contract.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/runtime_fixtures.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/retained_projection.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/host_shells.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
tests:
  - zircon_editor/src/tests/host/retained_window/native_mode.rs
  - zircon_editor/src/tests/host/retained_window/activity_rail_template_boundary.rs
  - zircon_editor/src/tests/host/retained_tab_drag/surface_contract.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/runtime_fixtures.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/retained_projection.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/host_shells.rs
doc_type: milestone-detail
title: Editor1024 retained window, menu, and template structural gate owners
category: zircon_editor
report_id: Editor1024-retained-structural-gates-2026-09-27
date: 2026-09-27
session_id: astra-optimize-20260926-batch-a
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: product_gate_pending
---

# Editor1024 retained window, menu, and template structural gate owners

## Failure evidence

Seven Editor test owners read former parent modules or searched for retired
implementation shapes. Their current production children confirm the failures:
native floating-window presentation, store, handle, snapshot and DTO declarations
are separate; side-rail DTO and scene patch logic live in child modules; tab
drop uses committed shell layout frames; disabled menu labels live in
`menu_chrome.rs`; the pane fixture and template node declarations live below
folder-backed modules. The builtin runtime cache, UI Asset Editor projection,
view projection, and welcome action also changed source shape. A source-only pre-repair replay
found 13/13 representative old assertions absent. No pre-repair Cargo test
execution is claimed.

The old popup route test expected a route node and linear index advancement
for each item. Current code inserts one `PopupSurface` node for the root layer
and one for each open submenu layer. It computes a visible item row from popup
geometry at pointer dispatch, then looks up the published path-to-index map.
`refresh_popup_items` builds that map when content changes. The existing
10,000-item virtual-surface and flipped nested-popup tests exercise observable
route, action, scroll, and node-count behavior; the old source markers no
longer describe the implementation.

## Repair

- Point each structural test at the implementation child that owns its
  assertion. Native window, activity rail, template node, fixture, and welcome
  checks keep their original required behavior. The pane fixture check also
  verifies the child modules are wired by `mod.rs`.
- Require tab drop to read the published committed layout and frames. Retain
  every prohibition on root-shell geometry fallback in the drag test.
- Require menu chrome slot-frame mapping and disabled-item muting in the real
  projection child. Require one popup surface insertion site for the root and
  one for nested layers, geometry-derived item selection, and lookup in the
  published route index. Keep the old hit-target, clone, and closed-state
  exclusions. Preserve the wired large-popup and nested-popup behavior tests.
- Match the current cached `.zui` registration and `UiSurface` projection
  calls while retaining the legacy-schema and recursive-projection exclusions.
  Read both the view projection cache and build children so the hard-cut
  requirement still covers the actual `UiV2SurfaceBuilder` owner.

No production binding changed. Three template test leaves were preexisting
untracked files; their complete byte preimages were saved before editing.
The activity-rail test had a clean Git status and CRLF worktree conversion;
its original CRLF style was preserved.

## Validation and remaining gate

The post-repair source replay passed 182/182 owner-marker, source-path,
view-projection, field, and fixture checks. Scoped
`rustfmt --check`, diff/whitespace review, exact inverse replay, and source
hashes are recorded in the Batch T operational evidence. These are static
checks only. The managed `zircon_editor --lib` lane, including the existing
large-popup, nested-popup, window, drag, fixture and template behavior tests,
is pending. Editor01 CPU, allocation, RSS, and latency product gates remain
open; this test repair makes no performance improvement claim.
