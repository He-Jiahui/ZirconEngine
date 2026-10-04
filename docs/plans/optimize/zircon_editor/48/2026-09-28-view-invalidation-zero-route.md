---
title: Editor48 View Invalidation Zero-route Correctness Repair
category: zircon_editor
report_id: Editor48-view-invalidation-zero-route-2026-09-28
date: 2026-09-28
implementation_status: implemented_pending_validation
validation_status: managed_validation_pending
---

# Editor48 View Invalidation Zero-route Correctness Repair

## Scope

This repair addresses E-MSG-P0-01's zero-subscriber refresh failure. It changes the editor host's
view invalidation path only; it does not change the public message bus contract or claim the other
Editor48 lifecycle, admission, delivery, and performance milestones are complete.

## Implementation

`EditorHostEventController::mark_view_invalidation` now records the authoritative dirty view
directly through `SharedEditorMessageBus::mark_view_dirty`. It no longer wraps invalidation in a
custom `view.invalidated` message whose dirty mark depended on a subscriber accepting delivery.
Empty masks remain no-ops. `refresh_view` and retained-host event refreshes use this same path.

As a result, a `PRESENTATION_DATA` refresh enters the full reflection refresh path and a
`TREE_STRUCTURE` refresh enters scene-inspection publication even when there is no listener for a
separate invalidation topic. This does not mark unrelated messages dirty and does not add a dummy
subscriber.

## Regression Coverage

The real `EventRuntimeHarness` refresh regression starts with an empty dirty set. It publishes a
dirty-bearing probe to `view.invalidated`, verifies that the host has no delivery target and the
bus dirty set remains empty, then calls `refresh_view` and asserts the requested presentation mask
is returned and the current snapshot backend is refreshed. The existing structure-only regression
verifies the separate scene-delta path without full reflection fallback.

## Validation

- Scoped rustfmt and direct UTF-8/LF/final-newline/trailing-whitespace checks passed. Scoped
  `git diff --check` reported only Git's LF-to-CRLF advisories for the Rust files.
- Cargo compilation and execution of the focused refresh tests remain pending managed validation.
- Managed validation must include the zero-subscriber presentation refresh and structure-only
  scene-delta regressions. No dynamic pass is claimed here.
- Editor48's broader subscriber pressure, message admission, lifecycle, and performance gates remain
  open.
