# Debug Observatory M2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the Debug Observatory M2 bounded snapshot timeline and register the debugging tool as a Window-surface ActivityWindow.

**Architecture:** Shared timeline DTOs live in `zircon_runtime_interface`; runtime owns a bounded ring-buffer store over authoritative `UiSurfaceDebugSnapshot` payloads; editor owns read-model projection and window/menu registration only. The new `Debug Observatory` ActivityWindow reuses `PanePayloadKind::RuntimeDiagnosticsV1` and `PaneRouteNamespace::Diagnostics`, so M2 does not fork the existing debug pane payload path.

**Tech Stack:** Rust workspace on `main`, `serde` DTOs, `VecDeque` bounded retention, existing workbench `ViewDescriptor`/`MenuAction::OpenView` seams, milestone-stage Cargo validation with `--locked --jobs 1` and `D:\cargo-targets\zircon-shared`.

---

## Repository Policy

- Work in the existing `main` checkout; do not create a worktree or feature branch.
- Do not commit unless the user explicitly requests a commit.
- Preserve unrelated dirty work and sibling-owned UI surface/layout/input changes.
- Add tests during implementation, but run Cargo/rustfmt only in the M2 testing stage unless a compile blocker needs earlier evidence.
- If validation fails in active sibling-owned incremental layout/render/pool or editor input/perf files, classify the failure before editing those files.

## Architecture Note

- Owner crate for shared contracts: `zircon_runtime_interface::ui::surface::timeline`.
- Owner crate for retained runtime behavior: `zircon_runtime::ui::surface::timeline`.
- Owner module for editor strings/selection: `zircon_editor::ui::workbench::debug_reflector::timeline`.
- Owner module for tool registration: `zircon_editor::ui::host::builtin_views::activity_windows` plus `zircon_editor::ui::workbench::model::menu`.
- Runtime snapshots remain authoritative; historical selection changes a timeline cursor only and never calls `UiSurface` mutation, layout, hit-test, or rebuild APIs.

## File Map

### Create

- `zircon_runtime_interface/src/ui/surface/timeline.rs`
  - Shared serializable timeline handle, summary, retention, and snapshot DTOs.
- `zircon_runtime/src/ui/surface/timeline.rs`
  - Runtime-owned bounded `UiDebugTimelineStore` and capture/select/read helpers.
- `zircon_runtime/src/ui/tests/timeline.rs`
  - Runtime store capacity, retention, ordering, selected-frame, and no-mutation tests.
- `zircon_editor/src/ui/workbench/debug_reflector/timeline.rs`
  - Editor read model for timeline summaries and selected historical snapshot projection.
- `zircon_editor/src/ui/host/builtin_views/activity_windows/debug_observatory_view_descriptor.rs`
  - ActivityWindow descriptor for `editor.debug_observatory`.

### Modify

- `zircon_runtime_interface/src/ui/surface/mod.rs`
  - Export shared timeline DTOs.
- `zircon_runtime/src/ui/surface/mod.rs`
  - Export `UiDebugTimelineStore`.
- `zircon_runtime/src/ui/tests/mod.rs`
  - Register the new `timeline` test module.
- `zircon_editor/src/ui/workbench/debug_reflector/mod.rs`
  - Wire the timeline read model and keep root module structural.
- `zircon_editor/src/ui/host/builtin_views/activity_windows/mod.rs`
  - Declare the debug ActivityWindow descriptor module.
- `zircon_editor/src/ui/host/builtin_views/activity_windows/activity_window_descriptors.rs`
  - Add `debug_observatory_view_descriptor()` to the built-in ActivityWindow registry.
- `zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs`
  - Gate `editor.debug_observatory` on the runtime diagnostics capability.
- `zircon_editor/src/ui/workbench/model/menu/window_menu.rs`
  - Add a Window menu item that opens `editor.debug_observatory`.
- `zircon_editor/src/ui/workbench/model/menu_item_model.rs`
  - Map `editor.debug_observatory` to `Window.DebugObservatory.Open`.
- `zircon_editor/src/core/editor_operation.rs`
  - Register the built-in `Window.DebugObservatory.Open` operation with `Window/Debug Observatory` menu path.
- `zircon_editor/src/ui/slint_host/menu_pointer/menu_items_for_layout.rs`
  - Add the fallback `OpenView.editor.debug_observatory` action to the Window menu layout.
- `zircon_editor/src/tests/host/builtin_window_descriptors.rs`
  - Assert the new descriptor is an ActivityWindow and reuses the Runtime Diagnostics pane payload.
- `zircon_editor/src/tests/workbench/view_model/shell_projection.rs`
  - Assert the Window menu contains the Debug Observatory action and operation path.
- `zircon_editor/src/tests/workbench/host_events/menu_binding.rs`
  - Assert the debug ActivityWindow menu binding round-trips through headless dispatch.
- `docs/superpowers/specs/2026-05-07-debug-observatory-design.md`
  - Record M2 timeline/window registration scope and plan source.
- `docs/zircon_editor/ui/workbench/debug_reflector.md`
  - Document timeline ownership, editor read model, and Window tool registration.
- `tests/acceptance/ui-debug-observatory.md`
  - Add M2 implementation/validation evidence.
- `.codex/sessions/20260507-1924-debug-observatory-m2.md`
  - Keep coordination state current and retire it at closeout.

## Milestone M2: Snapshot Timeline And Window Tool Registration

### Goal

Implement bounded timeline storage and expose Debug Observatory as a Window-surface tool without disturbing the existing Runtime Diagnostics drawer.

### In-Scope Behaviors

- A runtime timeline store captures `UiSurfaceDebugSnapshot` values into a bounded `VecDeque`.
- Captures receive stable `UiDebugTimelineFrameHandle` ids and deterministic summaries.
- When capacity is exceeded, old frames drop from the front and `dropped_frame_count` increments.
- Selecting a retained handle updates only the selected timeline handle.
- Editor read model derives latest/selected/previous/next labels from the shared snapshot DTO.
- The Window menu opens `editor.debug_observatory`; the existing View menu entry for `editor.runtime_diagnostics` stays a bottom drawer.

### Out Of Scope

- Detailed hit-test explanation expansion, invalidation cause expansion, diff/export/replay package, guarded property editing, and runtime UI incremental layout changes.
- New host native painter behavior; M2 reuses existing Runtime Diagnostics payload/projection/overlay plumbing.

### Implementation Slices

- [x] **M2.1 Add shared timeline DTOs**

  Create `zircon_runtime_interface/src/ui/surface/timeline.rs` with these public types:

  ```rust
  use serde::{Deserialize, Serialize};

  use crate::ui::event_ui::UiNodeId;

  use super::{UiSurfaceDebugOptions, UiSurfaceDebugSnapshot};

  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
  pub struct UiDebugTimelineFrameHandle(pub u64);

  #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
  pub struct UiDebugTimelineFrameSummary {
      pub handle: UiDebugTimelineFrameHandle,
      pub frame_index: u64,
      pub captured_at_millis: Option<u64>,
      pub source_target_id: String,
      pub source_label: String,
      pub schema_version: u32,
      pub node_count: usize,
      pub render_command_count: usize,
      pub hit_grid_cell_count: usize,
      pub invalidation_dirty_count: usize,
      pub has_damage_region: bool,
      pub warning_count: usize,
      pub selected_node: Option<UiNodeId>,
      pub capture_options: UiSurfaceDebugOptions,
  }

  #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
  pub struct UiDebugTimelineRetention {
      pub capacity: usize,
      pub len: usize,
      pub first_frame: Option<UiDebugTimelineFrameHandle>,
      pub latest_frame: Option<UiDebugTimelineFrameHandle>,
      pub selected_frame: Option<UiDebugTimelineFrameHandle>,
      pub dropped_frame_count: u64,
  }

  #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
  pub struct UiDebugTimelineSnapshot {
      pub selected_frame: Option<UiDebugTimelineFrameHandle>,
      pub summaries: Vec<UiDebugTimelineFrameSummary>,
      pub frames: Vec<UiSurfaceDebugSnapshot>,
      pub retention: UiDebugTimelineRetention,
  }
  ```

  Modify `zircon_runtime_interface/src/ui/surface/mod.rs`:

  ```rust
  mod timeline;

  pub use timeline::{
      UiDebugTimelineFrameHandle, UiDebugTimelineFrameSummary, UiDebugTimelineRetention,
      UiDebugTimelineSnapshot,
  };
  ```

- [x] **M2.2 Add runtime timeline tests first**

  Create `zircon_runtime/src/ui/tests/timeline.rs` with three tests:

  - `ui_debug_timeline_store_retains_latest_frames_and_reports_dropped_count`: capture frames 10, 11, and 12 into capacity 2, assert retained handles are `2` and `3`, retained frame indices are 11 and 12, latest and selected are handle `3`, and dropped count is 1.
  - `ui_debug_timeline_store_selects_retained_frame_and_rejects_evicted_handle`: capture frames 1 and 2, select handle `1`, capture frame 3 to evict it, assert selecting handle `1` returns false and selected/latest are handle `3`.
  - `ui_debug_timeline_selection_does_not_mutate_surface_snapshot_source`: capture a snapshot from a rebuilt surface, select historical handles, then assert a fresh `surface.debug_snapshot()` still has the same tree id, node count, render command count, and selected node as before selection.

  Use the same `UiSurface` fixture style as `zircon_runtime/src/ui/tests/diagnostics.rs`: create a surface, rebuild it, capture snapshots with distinct `surface_name`, `frame_index`, and `captured_at_millis`, then assert ordered summary fields.

  Register the module in `zircon_runtime/src/ui/tests/mod.rs`:

  ```rust
  mod timeline;
  ```

- [x] **M2.3 Implement runtime timeline store**

  Create `zircon_runtime/src/ui/surface/timeline.rs` with:

  ```rust
  use std::collections::VecDeque;

  use zircon_runtime_interface::ui::surface::{
      UiDebugTimelineFrameHandle, UiDebugTimelineFrameSummary, UiDebugTimelineRetention,
      UiDebugTimelineSnapshot, UiSurfaceDebugOptions, UiSurfaceDebugSnapshot,
  };

  #[derive(Clone, Debug)]
  pub struct UiDebugTimelineStore {
      capacity: usize,
      next_handle: u64,
      dropped_frame_count: u64,
      selected_frame: Option<UiDebugTimelineFrameHandle>,
      frames: VecDeque<UiDebugTimelineEntry>,
  }

  #[derive(Clone, Debug)]
  struct UiDebugTimelineEntry {
      summary: UiDebugTimelineFrameSummary,
      snapshot: UiSurfaceDebugSnapshot,
  }
  ```

  Required methods:

  ```rust
  impl UiDebugTimelineStore {
      pub fn new(capacity: usize) -> Self;
      pub fn capacity(&self) -> usize;
      pub fn capture_snapshot(&mut self, snapshot: UiSurfaceDebugSnapshot, options: UiSurfaceDebugOptions) -> UiDebugTimelineFrameHandle;
      pub fn select_frame(&mut self, handle: UiDebugTimelineFrameHandle) -> bool;
      pub fn latest_handle(&self) -> Option<UiDebugTimelineFrameHandle>;
      pub fn selected_snapshot(&self) -> Option<&UiSurfaceDebugSnapshot>;
      pub fn snapshot(&self) -> UiDebugTimelineSnapshot;
  }
  ```

  Export it from `zircon_runtime/src/ui/surface/mod.rs`:

  ```rust
  mod timeline;
  pub use timeline::UiDebugTimelineStore;
  ```

  The store must coerce `capacity == 0` to `1`, set the selected frame to the newly captured handle, and if the selected handle is evicted, move selection to the latest retained frame.

- [x] **M2.4 Add editor timeline read model**

  Create `zircon_editor/src/ui/workbench/debug_reflector/timeline.rs`:

  ```rust
  use zircon_runtime_interface::ui::surface::{
      UiDebugTimelineFrameHandle, UiDebugTimelineSnapshot, UiSurfaceDebugSnapshot,
  };

  use super::model::EditorUiDebugReflectorModel;

  #[derive(Clone, Debug, Default, PartialEq, Eq)]
  pub(crate) struct EditorUiDebugTimelineModel {
      pub retention: String,
      pub selected: String,
      pub latest: String,
      pub previous_frame: Option<UiDebugTimelineFrameHandle>,
      pub next_frame: Option<UiDebugTimelineFrameHandle>,
      pub frame_rows: Vec<String>,
      pub selected_reflector: EditorUiDebugReflectorModel,
  }
  ```

  Required behavior:

  - `from_timeline(snapshot: &UiDebugTimelineSnapshot) -> Self` uses the selected frame when present, otherwise the latest frame, otherwise `EditorUiDebugReflectorModel::no_active_surface()`.
  - Frame rows include handle, frame index, source label, node count, render command count, hit-grid cell count, dirty count, warning count, and selected node.
  - Previous/next handles are neighbors in `snapshot.summaries` around the selected handle.

  Add focused tests to this same file under `#[cfg(test)]` or to `debug_reflector/tests.rs` proving selected frame projection and previous/next handle derivation.

  Modify `zircon_editor/src/ui/workbench/debug_reflector/mod.rs`:

  ```rust
  mod timeline;
  pub(crate) use timeline::EditorUiDebugTimelineModel;
  ```

- [x] **M2.5 Add debug ActivityWindow descriptor tests first**

  Modify `zircon_editor/src/tests/host/builtin_window_descriptors.rs` to assert:

  ```rust
  let descriptor = descriptors
      .iter()
      .find(|descriptor| descriptor.descriptor_id == ViewDescriptorId::new("editor.debug_observatory"))
      .expect("missing Debug Observatory descriptor");
  assert_eq!(descriptor.kind, ViewKind::ActivityWindow);
  assert_eq!(descriptor.default_title, "Debug Observatory");
  assert_eq!(descriptor.icon_key, "debug-observatory");
  assert_eq!(descriptor.pane_template.as_ref().unwrap().body.payload_kind, PanePayloadKind::RuntimeDiagnosticsV1);
  assert_eq!(descriptor.pane_template.as_ref().unwrap().body.route_namespace, PaneRouteNamespace::Diagnostics);
  ```

  Import `PanePayloadKind` and `PaneRouteNamespace` in that test file.

- [x] **M2.6 Implement debug ActivityWindow descriptor**

  Create `zircon_editor/src/ui/host/builtin_views/activity_windows/debug_observatory_view_descriptor.rs`:

  ```rust
  use crate::ui::workbench::autolayout::default_constraints_for_content;
  use crate::ui::workbench::snapshot::ViewContentKind;
  use crate::ui::workbench::view::{
      PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
      PreferredHost, ViewDescriptor, ViewDescriptorId, ViewKind,
  };

  pub(super) fn debug_observatory_view_descriptor() -> ViewDescriptor {
      ViewDescriptor::new(
          ViewDescriptorId::new("editor.debug_observatory"),
          ViewKind::ActivityWindow,
          "Debug Observatory",
      )
      .with_preferred_host(PreferredHost::DocumentCenter)
      .with_default_constraints(default_constraints_for_content(
          ViewContentKind::RuntimeDiagnostics,
      ))
      .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
          "pane.runtime.diagnostics.body",
          PanePayloadKind::RuntimeDiagnosticsV1,
          PaneRouteNamespace::Diagnostics,
          PaneInteractionMode::TemplateOnly,
      )))
      .with_icon_key("debug-observatory")
  }
  ```

  Wire it into `mod.rs`, `activity_window_descriptors.rs`, and `builtin_view_descriptors.rs`. Capability gating must match `editor.runtime_diagnostics` by requiring `EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS`.

- [x] **M2.7 Add Window menu tests first**

  Modify `zircon_editor/src/tests/workbench/view_model/shell_projection.rs` to assert the Window menu contains `Debug Observatory`:

  ```rust
  let debug_observatory_item = model
      .menu_bar
      .menus
      .iter()
      .find(|menu| menu.label == "Window")
      .and_then(|menu| menu.items.iter().find(|item| item.label == "Debug Observatory"))
      .expect("debug observatory window menu item");
  assert_eq!(
      debug_observatory_item.action,
      Some(MenuAction::OpenView(ViewDescriptorId::new("editor.debug_observatory")))
  );
  assert_eq!(
      debug_observatory_item.operation_path.as_ref().map(|path| path.as_str()),
      Some("Window.DebugObservatory.Open")
  );
  ```

  Modify `zircon_editor/src/tests/workbench/host_events/menu_binding.rs` with a direct binding roundtrip for `OpenView.editor.debug_observatory`.

- [x] **M2.8 Implement Window menu registration**

  Modify `zircon_editor/src/ui/workbench/model/menu/window_menu.rs` so the menu contains `Debug Observatory` and `Reset Layout`, both enabled. Use `MenuItemModel::leaf(...)` to avoid duplicate struct literals.

  Modify `zircon_editor/src/ui/workbench/model/menu_item_model.rs`:

  ```rust
  "editor.debug_observatory" => Some("Window.DebugObservatory.Open"),
  ```

  Modify `zircon_editor/src/core/editor_operation.rs` built-in view operations to include:

  ```rust
  (
      "Window.DebugObservatory.Open",
      "Open Debug Observatory",
      "Window/Debug Observatory",
      "editor.debug_observatory",
  ),
  ```

  Modify `zircon_editor/src/ui/slint_host/menu_pointer/menu_items_for_layout.rs` fallback Window menu branch to include:

  ```rust
  menu_action("OpenView.editor.debug_observatory", true),
  ```

- [x] **M2.9 Update docs and acceptance evidence before testing stage**

  Update `docs/zircon_editor/ui/workbench/debug_reflector.md` with:

  - M2 plan source in the header.
  - `related_code` and `implementation_files` entries for timeline files and window/menu registration files.
  - A `## Snapshot Timeline` section explaining runtime ownership and editor read-only projection.
  - A `## Debug Observatory Window Tool` section explaining the `editor.debug_observatory` ActivityWindow and Window menu entry.

  Update `tests/acceptance/ui-debug-observatory.md` with an `## M2 Evidence` section listing implemented source changes and reserving command results for M2.T.

### Testing Stage: M2.T Timeline And Window Registration Validation

Run these commands after all M2 implementation slices and docs are complete:

```powershell
rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/surface/timeline.rs" "zircon_runtime_interface/src/ui/surface/mod.rs" "zircon_runtime/src/ui/surface/timeline.rs" "zircon_runtime/src/ui/surface/mod.rs" "zircon_runtime/src/ui/tests/timeline.rs" "zircon_runtime/src/ui/tests/mod.rs" "zircon_editor/src/ui/workbench/debug_reflector/mod.rs" "zircon_editor/src/ui/workbench/debug_reflector/timeline.rs" "zircon_editor/src/ui/host/builtin_views/activity_windows/mod.rs" "zircon_editor/src/ui/host/builtin_views/activity_windows/activity_window_descriptors.rs" "zircon_editor/src/ui/host/builtin_views/activity_windows/debug_observatory_view_descriptor.rs" "zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs" "zircon_editor/src/ui/workbench/model/menu/window_menu.rs" "zircon_editor/src/ui/workbench/model/menu_item_model.rs" "zircon_editor/src/core/editor_operation.rs" "zircon_editor/src/ui/slint_host/menu_pointer/menu_items_for_layout.rs" "zircon_editor/src/tests/host/builtin_window_descriptors.rs" "zircon_editor/src/tests/workbench/view_model/shell_projection.rs" "zircon_editor/src/tests/workbench/host_events/menu_binding.rs"
```

Expected result: no formatting diff.

```powershell
cargo test -p zircon_runtime --lib timeline --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: timeline store tests pass.

```powershell
cargo test -p zircon_editor --lib debug_observatory --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: Debug Observatory descriptor/menu/binding tests that include `debug_observatory` in the test name pass.

```powershell
cargo test -p zircon_editor --lib ui_debug_timeline --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: editor timeline read-model tests pass.

```powershell
cargo test -p zircon_editor --lib builtin_activity_windows_expose_window_template_documents --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: existing ActivityWindow template descriptor test remains green.

```powershell
cargo test -p zircon_editor --lib workbench_view_model_projects_menu_strip_drawers_and_status --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: Window menu projection includes Debug Observatory and existing Reset Layout operation path remains green.

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: editor lib compile succeeds with only existing warning noise, or any sibling-owned blocker is classified with file ownership.

### Debug/Correction Loop

- If `cargo test -p zircon_runtime --lib timeline` fails on ordering, inspect `VecDeque` eviction before changing tests; summaries must preserve oldest-to-newest retained order.
- If selecting an evicted handle returns true, fix `select_frame(...)` to check retained handles only.
- If `debug_observatory` descriptor tests fail due missing capability, update `with_builtin_required_capabilities(...)` rather than weakening `EditorCapabilitySnapshot`.
- If Window menu binding fails, keep the action id as `OpenView.editor.debug_observatory`; do not add a new `MenuAction` variant for this milestone.
- If editor lib validation fails in sibling-owned runtime surface/layout/input files, read current `.codex/sessions/` notes and record the blocker before patching.

### Exit Evidence

- M2.T command results are recorded in `tests/acceptance/ui-debug-observatory.md`.
- `docs/zircon_editor/ui/workbench/debug_reflector.md` documents the timeline and Window tool registration.
- `.codex/sessions/20260507-1924-debug-observatory-m2.md` is retired or archived with validation evidence.
