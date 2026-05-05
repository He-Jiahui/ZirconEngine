---
related_code:
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_editor/src/ui/slint_host/app/invalidation.rs
  - zircon_editor/src/ui/slint_host/host_contract/redraw.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/workbench/reflection/model_build.rs
  - zircon_editor/src/ui/workbench/reflection/transient_ui_state.rs
implementation_files:
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_editor/src/ui/workbench/reflection/model_build.rs
  - zircon_editor/src/ui/workbench/reflection/transient_ui_state.rs
  - zircon_editor/src/ui/slint_host/app/invalidation.rs
  - zircon_editor/src/ui/slint_host/host_contract/redraw.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
plan_sources:
  - user: 2026-05-06 完善调试工具，参照 dev 下虚幻源码
  - user-provided: Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - docs/superpowers/specs/2026-05-06-ui-lifecycle-reflection-reflector-design.md
  - docs/ui-and-layout/slate-style-ui-surface-frame.md
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Widgets/SWidgetReflector.cpp
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Models/WidgetReflectorNode.h
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Widgets/SWidgetHittestGrid.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Debugging/SlateDebugging.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h
tests:
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - cargo test -p zircon_runtime --lib diagnostics --locked
  - cargo test -p zircon_runtime --lib hit_grid --locked
  - cargo test -p zircon_editor --lib native_host_contract --locked
  - cargo check -p zircon_runtime_interface --lib --locked
  - cargo check -p zircon_runtime --lib --locked
  - cargo check -p zircon_editor --lib --locked
doc_type: design-spec
---

# UI Debug Reflector Full Closure Design

## Summary

This design expands Zircon UI debugging into a complete Widget Reflector-style loop. The approved strategy is Shared-first: shared runtime snapshot and export data become the source of truth first, then the editor panel, overlay, and replay/export tools consume that same data.

The primary user entry is an editor Debug Reflector panel. The acceptance target is a complete loop: live inspect, select or pick a node, view layout/render/hit/invalidation state, visualize overlays, export a snapshot, and replay or inspect the exported data without rebuilding editor-local geometry.

## Reference Evidence

Unreal Slate is the behavior reference, not an API to copy.

- `SWidgetReflector.cpp` shows the editor tool shape: live and snapshot modes, widget hierarchy tab, details tab, event log, hit-test grid tab, pick modes, source access, and snapshot export commands.
- `WidgetReflectorNode.h` shows the node detail surface: type, visibility, focusability, enabled state, desired size, geometry, source asset or file, hit-test visibility, invalidation root status, attribute count, and live versus snapshot identity.
- `SWidgetHittestGrid.cpp` and `FHittestGrid.h` show hit-grid diagnostics: cell display, flags for disabled or unsupported focus widgets, navigation intermediate results, and sorted widget data.
- `SlateDebugging.h` shows diagnostic event families: input, focus, navigation, cursor query, warnings, mouse capture, invalidation, paint, and widget update events.
- `SlateInvalidationRoot.h` shows performance and invalidation state that should be exposed: slow path versus fast path paint, cached elements, hit-test grid ownership, invalidation reason, and update phase performance stats.
- `FSlateDrawElement` shows the render-debug vocabulary: ordered draw elements with kind, layer, paint geometry, clipping, tint, brush, text, lines, custom verts, and debug quads.

Zircon translates these responsibilities to `.ui.toml` retained UI, `UiSurfaceFrame`, neutral runtime-interface DTOs, runtime diagnostics, and editor-only authoring/debug presentation.

## Current Baseline

- `UiSurfaceFrame` already carries one arranged tree, render extract, hit grid, and focus state.
- `UiSurface::debug_snapshot()` and `debug_surface_frame(...)` already generate `UiSurfaceDebugSnapshot` from `UiSurfaceFrame`.
- The current snapshot already includes `UiWidgetReflectorNode`, render counters, material batch stats, hit-grid occupancy, overdraw summary, and focus state.
- `debug_hit_test_surface_frame(...)` already reports hit stack, hit path, inspected count, and basic reject reasons.
- Editor native host already has damage-region, presenter, paint-only invalidation, and host invalidation counters, but those are not yet folded into a shared UI debug snapshot.
- Existing docs already identify Widget Reflector, drawcall, overdraw, material batching, hittest, invalidation, and damage region as the remaining M7 debug-tool target.

## Target Architecture

### Ownership

- `zircon_runtime_interface::ui::surface` owns serializable debug DTOs.
- `zircon_runtime::ui::surface` owns snapshot generation from `UiSurfaceFrame` and retained runtime state.
- `zircon_editor::ui` owns the editor Debug Reflector panel, picker controls, visual overlay toggles, and export command UI.
- Runtime rendering and editor host rendering may report backend counters, but they do not become layout, hit-test, or tree authorities.

### Data Flow

The debug flow is always:

```text
.ui.toml / UiTree
  -> UiSurface::compute_layout / rebuild_dirty
  -> UiSurfaceFrame { arranged_tree, render_extract, hit_grid, focus_state }
  -> UiSurfaceDebugSnapshot
  -> editor Debug Reflector model
  -> panel, overlay, export, replay
```

Editor code must not rebuild hit rectangles or infer draw order from host widgets. If a field is missing from `UiSurfaceDebugSnapshot`, the fix belongs in the shared snapshot or a clearly owned backend-counter side channel.

### Snapshot Data Model

Extend `UiSurfaceDebugSnapshot` into a stable exported debug payload.

Required top-level fields:

- schema version
- tree id
- optional surface name or source asset id
- optional frame index and capture timestamp
- debug options used to build the snapshot
- roots and reflected nodes
- selected node id
- optional pick point and hit-test dump
- render debug stats and command records
- hit-grid debug stats and cell records
- overdraw stats and sampled cells
- focus, hover, capture, and navigation state
- invalidation and damage report
- event log entries when an event source is available

The exported payload should be deterministic for tests. Timestamps and frame indices should be optional or normalized in unit tests.

### Reflector Nodes

`UiWidgetReflectorNode` should continue to mirror arranged geometry. New fields should be added only when they are derived from retained UI truth or stable metadata.

Required node fields:

- node id, node path, parent, children
- component id, display name, and control id
- source asset, template path, or template node path when available
- frame, clip frame, z index, and paint order
- visibility, input policy, enabled, clickable, hoverable, focusable, pressed, and checked state
- lifecycle and dirty flags when available from existing runtime metadata
- render command count and command ids
- hit entry count, hit cell count, and hit cell ids
- focus, hover, capture, and selected annotations
- property/reflection summary when the lifecycle reflection spec has landed

The node model should not expose raw Rust addresses. Unreal exposes widget addresses for live C++ objects, but Zircon uses retained serializable nodes where stable node ids and paths are the intended identity.

### Render Diagnostics

Render diagnostics should have two levels.

The first level is deterministic CPU-side extraction from `UiRenderExtract`:

- command id
- node id
- command kind
- frame and clip frame
- paint order and inferred layer order
- opacity
- style/material key
- text or image summary
- visible frame after clipping
- estimated draw-call contribution
- material batch key and batch break reason

The second level is optional backend-confirmed counters when runtime renderer instrumentation is available:

- submitted draw calls
- pipeline or material switches
- texture or atlas switches
- glyph/text batch count
- clipped batch count

Until backend counters are connected, fields must be named as estimates and the editor panel must label them as such.

### Hit-Test Diagnostics

Hit diagnostics should expand from aggregate counts to inspectable grid records.

Required hit-grid fields:

- grid bounds, cell size, columns, and rows
- cell id, cell bounds, and entry indices
- entry node id, control id, frame, clip frame, z index, paint order, and sort key
- front-to-back stack for a picked point
- root-to-leaf and bubble routes
- reject records with stable reason codes and human-readable messages

Reject reason codes should remain small and stable: outside frame, outside clip, visibility filtered, disabled, input policy ignore, not pointer target, missing ancestry, stale grid entry, and custom hit path unavailable.

### Overdraw Diagnostics

The first implementation should keep CPU-side deterministic sampled overdraw. It should expose enough data for an overlay without requiring a GPU debug pass.

Required fields:

- sample cell size
- sample bounds
- columns and rows
- per-cell layer count for cells that are covered
- top contributing node ids per cell when practical
- covered cells, overdrawn cells, max layers, and total layer samples

A GPU overdraw pass can replace or augment this later, but the editor panel and exported schema should already be able to store either CPU sampled or backend measured results.

### Invalidation And Damage Diagnostics

Zircon should translate Slate invalidation ideas into existing runtime/editor mechanisms.

Required report fields:

- last rebuild report: layout recomputed, arranged rebuilt, hit grid rebuilt, render rebuilt
- dirty flags before rebuild when available
- recompute reasons from the editor host invalidation root when available
- slow path versus paint-only path counters for editor host presentation
- damage region, painted pixel estimate, full paint count, region paint count, and total painted pixels when the native presenter owns the data
- warnings when no invalidation data source is available for a surface

Runtime and editor can contribute different data. The shared snapshot should support optional sections instead of forcing editor-only damage counters into runtime surfaces.

### Editor Debug Reflector Panel

The editor panel is the primary user entry.

Required panel modes:

- Live mode reads the latest available snapshot from the active editor/runtime UI surface.
- Snapshot mode opens a previously exported payload.
- Pick mode selects a node by point using the shared hit-test dump.
- Visual pick mode selects the front-most render command or node under a point when render command data is available.

Required panel sections:

- hierarchy tree
- selected node summary
- geometry and visibility details
- hit-test path and reject list
- render commands and material batches
- overdraw summary and cell list
- invalidation and damage report
- event log when input/focus/capture events are recorded
- export controls

The first editor integration should reuse existing workbench reflection and host-contract projection patterns. It should not trigger a broad Slint host rewrite.

### Overlay

Overlay output must be derived from `UiSurfaceDebugSnapshot`.

Required overlay toggles:

- selected node frame and clip frame
- root-to-leaf hit path
- hit-grid occupied cells
- rejected node bounds for a picked point
- overdraw heat cells
- render command bounds by material batch
- damage region

The overlay should be a debug presentation layer. It must not change hit-testing, layout, render extraction, or event routing.

### Export And Replay

Snapshot export should serialize the complete debug payload to JSON. Replay does not need to recreate live UI behavior in the first milestone. It must support loading the payload into the editor panel and overlay model for offline inspection.

Required export behavior:

- include schema version
- include debug options
- include selected node and pick point when present
- normalize optional backend counters when absent
- return structured errors for serialization, missing active surface, and unsupported destination path

## Error Handling

- Missing active surface should show an empty panel state with a clear reason.
- Stale selected node should keep the snapshot open and mark the node selection invalid.
- Unknown command, node, or hit-cell ids in an imported snapshot should be reported as import warnings, not panics.
- Backend counters may be absent. The panel must distinguish estimates from measured values.
- Export failures must return user-visible errors and must not corrupt the active live snapshot.

## Approaches Considered

### Shared-first

Selected. It completes the shared DTO and snapshot generator first, then makes editor UI and overlays consumers. This prevents editor-only coordinate drift and aligns with `UiSurfaceFrame` as the single spatial authority.

### Panel-first

Rejected. It would provide fast visual feedback, but it risks filling the editor panel with host-only layout or hit-test logic before the shared snapshot is complete.

### Overlay-first

Rejected as the primary strategy. It is useful for visual debugging, but it would delay export/replay and Widget Reflector-style inspection.

## Milestones

### M1 Shared Snapshot Schema

Implementation slices:

- Extend runtime-interface debug DTOs with schema version, selected/pick context, command records, hit cells, overdraw cells, invalidation report, damage report, and event log containers.
- Keep existing snapshot fields source-compatible where practical.
- Add deterministic serialization tests for representative snapshots.

Testing stage:

- `cargo test -p zircon_runtime_interface --lib contracts --locked`
- `cargo check -p zircon_runtime_interface --lib --locked`

### M2 Runtime Snapshot Generation

Implementation slices:

- Generate render command records from `UiRenderExtract`.
- Generate hit cell records and stable reject reason codes from `UiSurfaceFrame`.
- Generate sampled overdraw cells from render command visible frames.
- Attach `UiSurfaceRebuildReport` and dirty-flag data where available.
- Add export roundtrip helpers for snapshot JSON.

Testing stage:

- `cargo test -p zircon_runtime --lib diagnostics --locked`
- `cargo test -p zircon_runtime --lib hit_grid --locked`
- `cargo check -p zircon_runtime --lib --locked`

### M3 Editor Reflector Panel

Implementation slices:

- Add an editor reflection model that consumes `UiSurfaceDebugSnapshot`.
- Add live and loaded snapshot modes.
- Add selected-node details, hit-test details, render details, overdraw details, invalidation/damage details, and export actions.
- Route pick mode through shared hit-test data.

Testing stage:

- `cargo test -p zircon_editor --lib native_host_contract --locked`
- `cargo check -p zircon_editor --lib --locked`

### M4 Debug Overlay

Implementation slices:

- Derive overlay primitives from selected snapshot sections.
- Add toggles for selected frame, hit path, hit grid, overdraw, material batch bounds, rejected bounds, and damage region.
- Keep overlay presentation separate from layout, hit-test, render extract, and dispatch state.

Testing stage:

- `cargo test -p zircon_editor --lib native_host_contract --locked`
- `cargo check -p zircon_editor --lib --locked`

### M5 Documentation And Acceptance

Implementation slices:

- Update `docs/ui-and-layout/slate-style-ui-surface-frame.md` with the completed debug reflector flow.
- Update `docs/ui-and-layout/index.md` to route readers to the debug reflector documentation.
- Add or update module-detail docs for any newly created source modules.
- Add an acceptance note for export/replay and overlay behavior.

Testing stage:

- Run the scoped package checks listed above.
- Expand to workspace validation during the final milestone testing stage if shared public DTOs or cross-crate wiring changed broadly.

## Acceptance Criteria

- The editor has a Debug Reflector panel that can inspect the active UI surface in live mode.
- The panel can load an exported snapshot without requiring a live surface.
- Selecting or picking a point resolves through shared `UiSurfaceFrame` hit data.
- Node details show geometry, visibility, input, focus/capture/hover, render count, hit count, and source metadata when available.
- Render diagnostics show command records, material batches, and clearly labeled estimated versus measured counters.
- Hit diagnostics show grid cells, hit stack, hit path, and reject reasons.
- Overdraw diagnostics expose sampled cells and aggregate stats.
- Invalidation and damage diagnostics show available runtime/editor data without inventing false values when a source is absent.
- Overlay toggles visualize selected node, hit path, hit grid, overdraw, material batch bounds, rejected bounds, and damage region from the snapshot.
- Exported JSON roundtrips through the editor panel.
- Docs list plan sources, implementation files, and validation commands.

## Non-Goals

- Do not copy Unreal Slate APIs or convert Zircon to live `SWidget` inheritance.
- Do not make the editor host a second layout, render, or hit-test authority.
- Do not require a real GPU overdraw pass in the first closure milestone.
- Do not implement full keyboard, IME, text shaping, drag/drop, or popup debugging as part of this debug-tool milestone.
- Do not introduce compatibility shims for old editor-only hit-test paths.

## Risks

- The current worktree already has many UI changes. Implementation must read touched files carefully and avoid overwriting unrelated edits.
- Existing editor host projection may not yet expose every active `UiSurfaceFrame`. The panel should start with surfaces that already produce frames and show a clear unavailable state for the rest.
- Backend-confirmed drawcall counters may require renderer changes outside the UI crate. The first milestone should preserve CPU-side estimates and make measured counters optional.
- Damage/invalidation data is split between shared runtime and native editor host. The schema must support optional contributors without pretending every runtime surface has editor-host damage data.

## Approval

User approved the Shared-first strategy, Editor panel as the main entry, and complete live/snapshot/overlay/export closure on 2026-05-06.
