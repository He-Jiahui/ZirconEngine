---
title: Editor60 native child tab drop source routing
category: zircon_editor
date: 2026-09-28
implementation_status: implemented_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
related_code:
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/drag_resize.rs
  - zircon_editor/src/ui/retained_host/app/native_windows/store.rs
  - zircon_editor/src/ui/retained_host/floating_window_projection.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/retained_host/app/tests/child_window_tabs.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60/2026-09-27-native-tab-resize-pointer-owner.md
validation_manifest: .codex/state/session-coordinator/async-validation-batches/2026-09-28-editor60-child-drop-static-checks.json
source_manifest: .codex/state/session-coordinator/async-validation-batches/2026-09-28-editor60-child-drop-successor-source-manifest.json
tests:
  - zircon_editor/src/ui/retained_host/app/tests/child_window_tabs.rs
---

# Editor60 native child tab drop source routing

## Source ownership and coordinate path

The shared native tab callback previously called the root `UiHostContext` for every window. A child tab arms its drag state in that child `UiHostContext`, so a child Up could see an empty root drag and silently skip its drop. Even when the child state was read, the pointer route bridge uses root-zero Workbench frames; passing child-local x/y directly could route the release to a different target or to no target.

The callback now carries `Option<&MainPageId>` as its source window. Root input uses `None`, keeps the root context, and keeps the original point. Child input uses the configured native presenter ID, reads/writes that presenter's `UiHostContext`, and maps local x/y by adding the child outer frame from the same `FloatingWindowProjectionBundle` used to build drag hit geometry. Down, Move, target-group sync, and Up use this mapping so the route highlight and release resolve against one coordinate space. Missing, removed, hidden, or no-longer-native presenters return without falling back to root state.

The `source_window_id` follows the same source-first callback boundary as native resize. It is event-scoped and does not become retained host state. Drop dispatch may synchronously recompute presenters while the host is mutably borrowed, so this path does not introduce an app Cancel callback from programmatic hide; hide retirement must clear child-local capture without reentering the host callback.

## Product behavior and modeled tests

Child tab input through installed native presenter callbacks now has product-level modeled coverage for three release routes:

- Drop over the root document drag target attaches the tab to a root document workspace and removes the emptied source window.
- Drop over another child's projected floating frame attaches the tab to that target child while preserving the target window.
- Drop in empty space outside the projected Workbench drag surface follows the existing `DetachToWindow` policy and creates a new floating native presenter.

Each case drives the child's native press, move, and release test hooks using child-local coordinates and checks the committed view host/layout result. These tests exercise the installed callback route and Workbench model; they do not run the OS window system or prove that a platform delivers captured Up across windows.

## Validation and remaining gates

Scoped source checks and direct whitespace/final-newline checks are recorded in the successor manifest. Cargo was not run in this slice. Managed Rust execution remains pending. Actual Windows/winit cross-window capture and Up delivery, multiple-monitor or mixed-DPI movement, and live Editor interaction remain unverified. No pointer-event latency, allocation, or other performance measurement was run; the product performance gate remains pending.
