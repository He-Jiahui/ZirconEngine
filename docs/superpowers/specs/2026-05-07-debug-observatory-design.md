---
related_code:
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_editor/src/ui/workbench/debug_reflector/mod.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/workbench/debug_reflector/selection.rs
  - zircon_editor/src/ui/workbench/debug_reflector/export.rs
  - zircon_editor/src/ui/workbench/debug_reflector/overlay.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/debug_reflector_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
implementation_files:
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_editor/src/ui/workbench/debug_reflector/mod.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/workbench/debug_reflector/selection.rs
  - zircon_editor/src/ui/workbench/debug_reflector/export.rs
  - zircon_editor/src/ui/workbench/debug_reflector/overlay.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/debug_reflector_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
plan_sources:
  - user: 2026-05-07 continue improving all Debug Reflector tooling areas
  - docs/superpowers/plans/2026-05-07-debug-observatory-m0-m1.md
  - docs/superpowers/specs/2026-05-06-ui-debug-reflector-full-closure-design.md
  - docs/superpowers/plans/2026-05-06-ui-debug-reflector-full-closure.md
  - docs/zircon_editor/ui/workbench/debug_reflector.md
  - docs/ui-and-layout/shared-ui-core-foundation.md
  - docs/ui-and-layout/zircon-ui-unreal-slate-layout-gap-audit.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Widgets/SWidgetReflector.cpp
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Models/WidgetReflectorNode.h
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/WidgetSnapshotService.cpp
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Widgets/SWidgetSnapshotVisualizer.cpp
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/VisualTreeCapture.cpp
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Widgets/SWidgetEventLog.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Debugging/SlateDebugging.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/InvalidateWidgetReason.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Debugging/ConsoleSlateDebuggerInvalidate.cpp
  - dev/slint/tools/lsp/preview.rs
  - dev/slint/tools/lsp/preview/element_selection.rs
  - dev/slint/tools/lsp/preview/preview_data.rs
  - dev/slint/tools/lsp/preview/ui/property_view.rs
  - dev/slint/internal/compiler/passes/inject_debug_hooks.rs
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
  - dev/bevy/crates/bevy_ecs/src/change_detection/mod.rs
  - dev/bevy/crates/bevy_reflect/src/reflect.rs
  - dev/godot/core/object/property_info.h
  - dev/godot/scene/debugger/scene_debugger_object.cpp
  - dev/godot/editor/debugger/editor_debugger_inspector.cpp
  - dev/Fyrox/fyrox-core/src/reflect.rs
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/Fyrox/editor/src/command/panel.rs
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_editor/src/ui/workbench/debug_reflector/tests.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - tests/acceptance/ui-debug-observatory.md
doc_type: design-spec
---

# Debug Observatory Design

## Summary

Debug Observatory is the next Debug Reflector feature family. It extends the current `UiSurfaceDebugSnapshot`-based reflector into a staged tool for live surface capture, bounded history, hit-test explanation, invalidation and damage diagnosis, snapshot diff, export/replay, and guarded property editing.

The design is snapshot-first. `UiSurfaceFrame` and runtime-produced `UiSurfaceDebugSnapshot` remain the source of truth. Editor code presents, filters, exports, replays, and requests mutations through contracts; it must not rebuild layout, infer hit geometry from host widgets, or mutate runtime memory directly.

## Current Baseline

The current Debug Reflector already has the right lower-layer seam, but the live pane is not connected deeply enough yet.

- `zircon_runtime_interface::ui::surface::diagnostics` owns `UiSurfaceDebugSnapshot`, node rows, render stats, hit-grid stats, overdraw stats, invalidation/damage reports, event records, and overlay primitive DTOs.
- `zircon_runtime_interface::ui::surface::frame` owns `UiSurfaceFrame`, which packages arranged tree, render extract, hit grid, focus state, and rebuild stats.
- `zircon_runtime::ui::surface::diagnostics` can produce a surface debug snapshot from a `UiSurfaceFrame`.
- `zircon_runtime::ui::surface::surface` exposes `debug_snapshot`, `debug_snapshot_for_pick`, `debug_snapshot_for_selection`, `debug_snapshot_json`, `mutate_property`, and `reflector_snapshot`-adjacent surfaces.
- `zircon_editor::ui::workbench::debug_reflector` can project a snapshot into summary, node rows, details, render/hit/overdraw/invalidation/damage sections, selection, JSON helpers, and overlay filtering.
- Runtime Diagnostics currently still builds `EditorUiDebugReflectorModel::no_active_surface()` and empty overlay primitives when no active shared frame is injected into the pane payload path.

Known blockers and sibling-owned areas remain outside this design's first implementation slice: active input/window, Material/runtime-preview, Slate layout/render lanes, and broad editor-lib failures. Debug Observatory must advance from the shared snapshot and Runtime Diagnostics seam without editing those lanes unless a later milestone explicitly owns the boundary.

## Reference Evidence

Unreal Slate is the primary behavior reference for an engine-scale widget reflector. Slint, Bevy, Godot, and Fyrox constrain how this lands in Zircon's Rust/editor/runtime architecture.

| Reference | Files | Design pressure |
|---|---|---|
| Unreal Slate Reflector | `SWidgetReflector.cpp`, `SWidgetReflector.h`, `WidgetReflectorNode.h`, `WidgetReflectorNode.cpp` | Keep live and snapshot modes, hierarchy/details/events/hit-test panes, pick modes, and a node model that works for live and recorded data. |
| Unreal snapshot service | `WidgetSnapshotService.cpp`, `SWidgetSnapshotVisualizer.cpp` | Treat snapshots as versioned transportable payloads with target/request identity, save/load support, and visual picking over recorded captures. |
| Unreal visual capture | `VisualTreeCapture.cpp`, `SlateDebugging.h` | Capture paint/draw ownership and debug events through runtime instrumentation, not editor reconstruction. |
| Unreal invalidation debugger | `ConsoleSlateDebuggerInvalidate.cpp`, `ConsoleSlateDebuggerInvalidationRoot.cpp`, `InvalidateWidgetReason.h` | Model invalidation reasons, dirty chains, cache health, fast/slow paint paths, and overlays as explicit diagnostics. |
| Slint live preview | `preview.rs`, `element_selection.rs`, `preview_data.rs`, `ui/property_view.rs`, `inject_debug_hooks.rs` | Preserve session state, selection stacks, property metadata, source-linked inspection, and debug hooks with stable element/property IDs. |
| Bevy diagnostics/change detection | `bevy_diagnostic/src/diagnostic.rs`, `bevy_ecs/src/change_detection/mod.rs`, `bevy_reflect/src/reflect.rs` | Use bounded histories, explicit change epochs, deterministic reflection, and reflected patches instead of unbounded ad hoc logs. |
| Godot debugger/inspector | `property_info.h`, `scene_debugger_object.cpp`, `editor_debugger_inspector.cpp` | Use property usage flags, runtime object proxies, structured remote edits, and size/truncation markers. |
| Fyrox editor commands/reflection | `fyrox-core/src/reflect.rs`, `editor/src/command/mod.rs`, `editor/src/command/panel.rs` | Address properties by reflected paths and prefer reversible commands/event logs for replayable edits. |

Zircon deliberately diverges from Unreal's live C++ widget-address model. Zircon identities must be stable serializable node ids, tree ids, source/template paths, and reflection paths because the runtime UI is retained and exported through DTOs.

## Architecture

### Ownership

| Layer | Responsibility |
|---|---|
| `zircon_runtime_interface` | Stable serializable contracts for snapshots, timeline frames, diffs, replay packets, mutation requests/results, property metadata, event records, and overlay primitives. |
| `zircon_runtime` | Authoritative capture from `UiSurfaceFrame`, hit-grid dumps, render extract diagnostics, invalidation reasons, event ingestion, bounded history storage, and property mutation execution. |
| `zircon_editor` | Debug Observatory UI projection, Runtime Diagnostics pane payloads, timeline selection, export/import UI, diff presentation, overlay toggles, and mutation request UI. |
| Host/native painter | Draws debug overlays from shared primitives after normal pane content; it never owns layout, hit testing, render extraction, or routing. |

### Data Flow

```text
.ui.toml / UiTree
  -> UiSurface::compute_layout / surface rebuild
  -> UiSurfaceFrame { arranged_tree, render_extract, hit_grid, focus_state, rebuild_stats }
  -> UiSurfaceDebugSnapshot
  -> optional UiDebugTimelineStore
  -> editor Debug Observatory model
  -> Runtime Diagnostics pane, overlay painter, export, diff, replay, mutation request UI
```

No milestone may introduce an editor-local geometry cache as a source of truth. If the editor needs a field, the field belongs in a shared snapshot contract or in an optional backend-counter section with clear ownership.

### Public Contract Families

The feature family should converge around five DTO families rather than one growing struct with unrelated responsibilities.

| Contract family | Purpose |
|---|---|
| Snapshot | Current `UiSurfaceDebugSnapshot` plus capture metadata, node records, render/hit/overdraw/invalidation/damage/event sections. |
| Timeline | Bounded list of snapshot handles and summaries, frame cursor, retention limit, dropped-frame count, target/source identity, and capture options. |
| Diff | Read-only comparison between two snapshots using stable node ids, paths, command ids, hit-cell ids, and section counters. |
| Replay | Stored snapshot package with schema version, compatibility report, imported timeline frames, and read-only scrub state. |
| Mutation | Property metadata, mutation request, mutation result, validation errors, and eventual command/replay link. |

## Milestones

### M0 Baseline Freeze

Goal: preserve the existing Debug Reflector closure while new work is designed and started.

Scope:

- Keep current `UiSurfaceDebugSnapshot` projection behavior intact.
- Keep existing focused Debug Reflector and native host-contract checks as the baseline.
- Record current dirty-tree and sibling-owned blockers in the acceptance note.
- Do not widen into input/window, Material/runtime-preview, or active Slate render/layout ownership.

Completion gate:

- Existing Debug Reflector model tests still cover no-active state, snapshot projection, stale selection warning, JSON round trip, overlay filtering, and damage-region derivation.
- Existing Runtime Diagnostics host conversion tests still cover reflector fields and overlay primitives.
- Acceptance docs clearly distinguish Debug Observatory-owned work from integrated-tree blockers.

### M1 Live Snapshot Feed

Goal: connect Runtime Diagnostics to real shared `UiSurfaceDebugSnapshot` data when an active `UiSurfaceFrame` exists.

Scope:

- Add a narrow transport from active shared UI surface frame/snapshot into Runtime Diagnostics pane payload construction.
- Keep `no_active_surface()` as the fallback only when no shared frame is available.
- Preserve overlay primitives from the shared snapshot through pane payload projection and host contract conversion.
- Keep the pane payload as the editor presentation boundary; do not let the native host query runtime internals.

Completion gate:

- Runtime Diagnostics can show real node rows, detail lines, and render/hit/overdraw/invalidation/damage sections from a supplied snapshot.
- Overlay primitives in the snapshot are visible in the Runtime Diagnostics host painter path.
- Missing active surface produces the existing stable no-active placeholder.
- Tests cover real snapshot payload projection and fallback behavior.

### M2 Snapshot Timeline

Goal: add bounded history so the reflector can inspect recent frames without affecting runtime state.

Scope:

- Add a `UiDebugTimelineStore` or equivalent lower-layer store with fixed retention.
- Store snapshot summaries and snapshot payloads by stable frame handle.
- Track frame index, capture time, source target, schema version, capture options, current selection, and dropped-frame count.
- Expose editor read-model functions for latest frame, selected historical frame, next/previous, and retention summary.

Completion gate:

- Recent N frames can be captured and selected deterministically in tests.
- Selecting a historical frame does not mutate runtime UI state.
- Retention behavior is explicit when more than N frames are captured.
- Timeline summaries are deterministic except for normalized timestamp fields.

### M3 Hit-Test Explanation

Goal: explain why a point hit or did not hit a node.

Scope:

- Expand editor projection from target/rejected count into ordered hit-test details.
- Show pick point, grid cell lookup, candidate entries, front-to-back stack, accepted target, bubble/root path, and rejected entries.
- Standardize stable reject reason codes: outside frame, outside clip, hidden/collapsed, disabled, input policy ignore, not pointer target, stale grid entry, missing ancestry, and custom path unavailable.
- Include clip/focus/input flags for selected and rejected nodes when the snapshot already carries them.

Completion gate:

- A pick snapshot can explain accepted and rejected candidates without recomputing hit geometry in editor code.
- Tests cover at least one accepted top hit, one clipped rejection, one disabled/input-policy rejection, and one empty-cell miss.
- Existing hit-grid parity tests remain the lower-layer authority.

### M4 Invalidation And Damage Observatory

Goal: turn invalidation/damage counters into actionable causes and overlays.

Scope:

- Extend invalidation report with dirty nodes, rebuild reasons, recompute reason records, and warnings when a data source is unavailable.
- Preserve damage-region and paint-count data from native presenter-owned sources as optional snapshot sections.
- Add overlay primitive kinds only after the shared snapshot can carry dirty/invalidation regions.
- Keep invalidation overlays read-only; they cannot force layout, hit-grid, or render rebuilds.

Completion gate:

- The panel distinguishes layout, hit-grid, render, paint-only, and damage causes when present.
- Dirty/invalidation overlay primitives are derived from shared snapshot fields.
- Missing invalidation sources produce explicit warnings rather than fabricated zeros.
- Tests cover reason projection, missing-source warning, and overlay filtering.

### M5 Snapshot Diff

Goal: compare two snapshots without replaying or mutating runtime state.

Scope:

- Add a snapshot diff DTO and editor projection.
- Compare nodes by stable node id first, then path/source fallback when ids are absent.
- Report added/removed nodes, moved/resized frames, clip changes, render command count deltas, hit-cell deltas, invalidation reason changes, damage changes, and warning deltas.
- Keep diff output deterministic and serializable.

Completion gate:

- Tests cover added, removed, unchanged, and changed nodes.
- Tests cover frame, clip, render count, hit-cell, and damage deltas.
- Diffing two imported snapshots produces the same result as diffing two live-captured snapshots with equivalent payloads.

### M6 Export And Replay

Goal: make snapshots portable and inspectable after capture.

Scope:

- Define a versioned exported package for one snapshot or a timeline segment.
- Support deterministic JSON export/import first; binary/compressed payloads remain future work.
- Add compatibility reporting for schema mismatch, missing optional sections, unsupported future fields, and parse failures.
- Replay is read-only in this milestone: scrub imported snapshots and render panels/overlays from stored payloads.

Completion gate:

- Exported packages include schema version, source target, capture options, timeline summaries, and snapshot payloads.
- Loading an exported package recreates the reflector model and overlays without a live runtime surface.
- Compatibility errors are structured and user-readable.
- Tests cover single-snapshot export/import, timeline export/import, malformed JSON, unsupported schema, and missing optional sections.

### M7 Guarded Property Editing

Goal: allow controlled live property edits through reflection/mutation contracts, not direct editor mutation.

Scope:

- Use existing `UiSurface::mutate_property`, `UiPropertyMutationRequest`, and reflection metadata as the lower-layer path.
- Expose only writable, inspectable, non-secret properties.
- Show read-only, internal, secret, unsupported, and stale-schema states explicitly.
- Every edit returns a structured mutation report with target id/path, property path, old value policy, new value, validation result, side-effect summary, and refreshed snapshot handle when available.
- Defer replaying edits until command/history integration is explicit; snapshot replay remains read-only.

Completion gate:

- Editable property rows appear only for permitted properties.
- Invalid property path, type mismatch, read-only property, stale node, and missing surface all return structured errors.
- Successful edits go through runtime mutation APIs and refresh the reflector from a new snapshot.
- Tests cover allowed edit, denied edit, stale target, type mismatch, and no-active-surface behavior.

### M8 Tool Surface Closure

Goal: close the user-facing Debug Observatory workflow.

Scope:

- Add capture target selection when multiple surfaces are available.
- Add timeline filters, event stream filters, overlay filter groups, retention settings, and save/load commands.
- Separate live, history, imported replay, diff, and editing modes in the panel state.
- Add headless or deterministic fixtures for capture, timeline, diff, import, and no-active-surface flows.

Completion gate:

- The tool supports live inspect, historical inspect, imported replay, diff, overlay toggles, file export/import, and guarded property edits from one coherent pane model.
- Mode transitions are tested and do not cross-mutate runtime state accidentally.
- Acceptance documentation includes exact commands, passing/failing scope, and remaining integrated-tree blockers.

## Data Model Details

### Timeline Records

Timeline summaries should be cheap to render without deserializing full snapshots when possible.

Required fields:

- frame handle
- frame index
- captured-at millis or normalized test timestamp
- source target id and display label
- schema version
- node count
- render command count
- hit-grid cell count
- invalidation dirty count
- damage region presence
- warning count
- selected node id
- capture options

Retention must be explicit. A bounded ring buffer should expose capacity, current length, first frame, latest frame, selected frame, and dropped-frame count.

### Hit Explanation Records

The hit explanation model should avoid string-only storage. Human-readable lines are editor projection output, not the shared contract.

Required fields:

- query point
- grid cell id and bounds
- candidate entry ids in tested order
- node id, frame, clip frame, z index, paint order, and control id for each candidate
- accepted target and path
- rejected candidates with reason code and optional detail text
- warnings for stale or missing grid data

### Invalidation Records

Invalidation records should be append-friendly and reason-coded.

Required fields:

- rebuild booleans for layout, arranged tree, hit grid, render extract, and paint-only path
- dirty node ids when known
- reason code and source path for each reason when known
- slow-path/fast-path counters when host presenter data is available
- warnings for unavailable sources

### Diff Records

Diff records must stay compact enough for timelines.

Required fields:

- base frame handle
- target frame handle
- added node ids
- removed node ids
- changed node records
- section deltas for render, hit, overdraw, invalidation, damage, events, and warnings
- compatibility warnings when snapshots use different schema versions

### Mutation Records

Mutation requests and results should support future command integration without requiring it in the first editing milestone.

Required request fields:

- target surface id or active-surface token
- target node id and optional node path fallback
- property path
- typed value payload
- source mode: live inspector, replay attempt, automated test, or future command replay

Required result fields:

- accepted or rejected
- structured error code when rejected
- target identity used
- property path
- refreshed snapshot handle when available
- warning list

## Testing Strategy

Testing follows milestone stages, not per-edit cargo churn. Unit test code can be added during implementation slices, but command execution belongs to each milestone's testing stage unless explicitly requested earlier.

| Milestone | Required focused validation |
|---|---|
| M0 | Existing `ui_debug_reflector`, `native_host_contract`, and runtime diagnostics focused checks. |
| M1 | Runtime Diagnostics real snapshot projection, fallback no-active projection, overlay primitive preservation. |
| M2 | Timeline capacity, retention, selected frame, deterministic summaries, no runtime mutation on historical selection. |
| M3 | Hit explanation accepted path, clipped rejection, disabled/input-policy rejection, empty miss. |
| M4 | Dirty reason projection, missing invalidation source warning, dirty/damage overlay filtering. |
| M5 | Snapshot diff added/removed/changed/unchanged nodes and section deltas. |
| M6 | Export/import single snapshot, export/import timeline, malformed JSON, unsupported schema, missing optional sections. |
| M7 | Allowed mutation, read-only denied mutation, secret/internal hidden property, type mismatch, stale target, no active surface. |
| M8 | Mode transitions across live/history/imported/diff/edit, save/load commands, retention settings, acceptance document. |

Workspace validation remains blocked by current sibling-owned integrated failures until those lanes are fixed. Debug Observatory acceptance should therefore distinguish focused owned evidence from root workspace CI parity.

## Risks And Constraints

- Runtime Diagnostics currently lacks an active shared surface snapshot feed. M1 must fix this before history, diff, or replay can provide meaningful live value.
- Property editing can change runtime state. M7 must remain behind explicit writable/non-secret/inspectable metadata and structured mutation errors.
- Invalidation and damage sources are split between runtime surface rebuild stats and editor/native presenter paths. Optional snapshot sections are required to avoid fabricating data.
- Large snapshot histories can grow memory quickly. Timeline retention must be bounded from the first implementation.
- Diff/replay must not rely on wall-clock timestamps. Tests should normalize timestamps and compare deterministic payload fields.
- Current sibling lanes own input/window, Material/runtime-preview, text/layout/render/input, and broad editor host failures. Early milestones should avoid editing those surfaces unless a lower-layer blocker is explicitly reassigned.

## Acceptance Definition

Debug Observatory is complete when one coherent editor-facing tool can:

- inspect the current live shared UI surface snapshot;
- keep and scrub a bounded local timeline;
- explain hit-test results and rejected candidates;
- explain invalidation, damage, render, hit, and overdraw state from shared diagnostics;
- compare two snapshots;
- export and import one snapshot or a timeline segment;
- replay imported snapshots read-only;
- perform guarded property edits through runtime mutation contracts;
- render overlays only from shared overlay primitives;
- document focused validation and any remaining integrated-tree blockers.

## Next Step

After this design is approved, create an implementation plan that starts with M0/M1 only. Later milestones should remain planned but not implemented until M1 proves the live snapshot feed and Runtime Diagnostics transport are stable.
