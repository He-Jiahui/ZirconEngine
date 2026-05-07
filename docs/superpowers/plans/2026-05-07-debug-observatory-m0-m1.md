# Debug Observatory M0/M1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect Runtime Diagnostics to a real shared `UiSurfaceDebugSnapshot` input while preserving the existing no-active-surface fallback and current Debug Reflector focused evidence.

**Architecture:** This plan implements only Debug Observatory M0/M1 from `docs/superpowers/specs/2026-05-07-debug-observatory-design.md`. `UiSurfaceFrame` and runtime-produced `UiSurfaceDebugSnapshot` remain authoritative; `zircon_editor` only transports a supplied snapshot through `PanePayloadBuildContext` into `RuntimeDiagnosticsPanePayload` and host projection. It deliberately avoids active sibling-owned input/window, Material/runtime-preview, root-frame cleanup, text/layout/render/input, and broad painter lanes.

**Tech Stack:** Rust workspace on `main`, `zircon_runtime_interface` UI DTOs, `zircon_runtime::ui::surface` diagnostics, `zircon_editor` Runtime Diagnostics pane projection, Cargo focused validation with `--locked --jobs 1`, shared target dir `D:\cargo-targets\zircon-shared`.

---

## Repository Policy

- Work in the existing checkout on `main`; do not create a worktree or feature branch.
- Do not commit unless the user explicitly requests a commit.
- Preserve unrelated dirty work. If a file contains unrelated changes, read it and make only the planned edit.
- Do not run per-slice Cargo build/test loops. Add code and test code first, then run the declared M1 testing stage.
- Use `D:\cargo-targets\zircon-shared` for Cargo validation unless a concrete lock/contention issue requires a different target dir.
- If validation fails in sibling-owned input/window, Material/runtime-preview, root-frame cleanup, text/layout/render/input, or broad painter code, classify and record it instead of patching that lane from this task.

## File Map

### Create

- `tests/acceptance/ui-debug-observatory.md`
  - Acceptance record for M0/M1 scope, focused evidence, and any integrated-tree blockers.

### Modify

- `.codex/sessions/20260507-1202-debug-observatory-m0-m1.md`
  - Live coordination note for this task. Update before source edits, after validation, and at handoff/cleanup.
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_presentation.rs`
  - Add optional borrowed active UI debug snapshot to `PanePayloadBuildContext` plus a builder method.
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs`
  - Build `EditorUiDebugReflectorModel` from `context.active_ui_debug_snapshot` when present; otherwise keep `no_active_surface()`.
  - Populate `ui_debug_reflector_overlay_primitives` from `EditorUiDebugReflectorOverlayState::default().primitives_from_snapshot(snapshot)` for the active snapshot path.
- `zircon_editor/src/ui/workbench/debug_reflector/mod.rs`
  - Re-export `EditorUiDebugReflectorOverlayState` inside the crate so the Runtime Diagnostics payload builder can use the same overlay filtering rules as tests/host code.
- `zircon_editor/src/tests/host/pane_presentation.rs`
  - Add a focused builder test proving Runtime Diagnostics uses a supplied active `UiSurfaceDebugSnapshot` and still falls back when absent.
- `zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs`
  - Extend the projection fixture to supply an active UI debug snapshot and assert root attributes contain the live reflector summary/details instead of the no-active placeholder.
- `zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs`
  - Add or adjust a focused host conversion assertion for snapshot-carried overlay primitives from the payload path, not only manually constructed host data.
- `docs/zircon_editor/ui/workbench/debug_reflector.md`
  - Document that M1 allows the Runtime Diagnostics payload builder to consume an explicitly supplied active shared debug snapshot before host-body refresh.
- `docs/superpowers/specs/2026-05-07-debug-observatory-design.md`
  - Add the implementation plan path to `plan_sources` if not already present.

### Do Not Touch In This Plan

- `zircon_editor/src/ui/slint_host/host_contract/window.rs`
- Material asset/theme files and Material global boundary tests.
- Runtime input dispatch files under `zircon_runtime/src/ui/surface/input` and `zircon_runtime_interface/src/ui/dispatch`.
- Root-frame cleanup files under `zircon_editor/src/ui/slint_host/root_shell_projection.rs`, `shell_pointer`, `tab_drag`, `floating_window_projection.rs`, and `drawer_resize.rs` unless validation proves the current M1 slice cannot compile without a local import update.

## Milestone M0: Baseline Freeze And Coordination

### Goal

Record the current baseline and active overlap before source edits. This milestone does not claim workspace green.

### In-Scope Behaviors

- Preserve existing Debug Reflector behavior and focused evidence.
- Keep the active coordination note accurate.
- Create an acceptance document for Debug Observatory M0/M1.
- Do not edit source code in M0.

### Dependencies

- `docs/superpowers/specs/2026-05-07-debug-observatory-design.md` is approved.
- Recent coordination has been scanned with the 4-hour lookback.

### Implementation Slices

- [ ] **M0.1 Update session note before source edits**

  Edit `.codex/sessions/20260507-1202-debug-observatory-m0-m1.md`:

  ```markdown
  status: active-m0-baseline
  updated_at: 2026-05-07 <current local time> +08:00
  ```

  Add these facts under `## Current Step`:

  ```markdown
  - M0 baseline is being recorded before source edits. Planned source scope is `PanePayloadBuildContext`, Runtime Diagnostics payload builder, Debug Reflector crate re-export, focused host/projection tests, docs, and acceptance only.
  ```

- [ ] **M0.2 Create acceptance record**

  Create `tests/acceptance/ui-debug-observatory.md` with this structure:

  ```markdown
  # UI Debug Observatory Acceptance

  Plan source: `docs/superpowers/plans/2026-05-07-debug-observatory-m0-m1.md`
  Design source: `docs/superpowers/specs/2026-05-07-debug-observatory-design.md`

  ## Scope

  - M0 records the current Debug Reflector baseline and active coordination constraints.
  - M1 connects Runtime Diagnostics pane payload construction to an explicitly supplied active `UiSurfaceDebugSnapshot`.
  - M1 preserves the existing no-active-surface fallback when no snapshot is supplied.
  - M1 does not implement timeline, hit-test explanation expansion, invalidation overlays, diff, replay, or property editing.

  ## M0 Baseline

  - Branch policy: work remains on `main`; no worktree or feature branch.
  - Coordination: active Slate M8 and drawer/window/menu lanes were present during planning.
  - Existing known risk: broad workspace validation can still expose sibling-owned dirty-tree failures.

  ## M1 Evidence

  - Pending implementation.

  ## Remaining Out Of Scope

  - Snapshot timeline/history.
  - Detailed hit-test explanation beyond current snapshot projection.
  - Invalidation/damage cause overlays beyond existing snapshot fields.
  - Snapshot diff/export package/replay.
  - Guarded property editing.
  ```

- [ ] **M0.3 Update design spec with plan source**

  In `docs/superpowers/specs/2026-05-07-debug-observatory-design.md`, add this line under `plan_sources`:

  ```yaml
  - docs/superpowers/plans/2026-05-07-debug-observatory-m0-m1.md
  ```

### Testing Stage: M0.T Baseline Hygiene

Run only lightweight docs hygiene, not Cargo:

```powershell
git diff --check -- docs/superpowers/specs/2026-05-07-debug-observatory-design.md docs/superpowers/plans/2026-05-07-debug-observatory-m0-m1.md tests/acceptance/ui-debug-observatory.md .codex/sessions/20260507-1202-debug-observatory-m0-m1.md
```

Expected result: no trailing-whitespace or patch whitespace errors. Windows LF/CRLF warnings may appear and should be recorded but are not a source-code failure.

### Exit Evidence

- Acceptance file exists and states M0/M1 scope.
- Session note states active source-edit scope and overlap warnings.
- Design spec references this implementation plan.

## Milestone M1: Runtime Diagnostics Live Snapshot Feed

### Goal

Allow Runtime Diagnostics payload construction to use an explicitly supplied active `UiSurfaceDebugSnapshot`, while preserving no-active fallback and existing host-body refresh behavior.

### In-Scope Behaviors

- `PanePayloadBuildContext` can carry `Option<&UiSurfaceDebugSnapshot>`.
- Runtime Diagnostics payload builder prefers that snapshot for Debug Reflector summary, node rows, detail lines, export status, and overlay primitives.
- Absence of a snapshot keeps current no-active placeholder behavior.
- Template runtime root attributes reflect the active snapshot when provided.
- Host conversion receives snapshot-derived overlay primitives through `RuntimeDiagnosticsPanePayload`.

### Dependencies

- Existing `EditorUiDebugReflectorModel::from_snapshot(...)` and `EditorUiDebugReflectorOverlayState::primitives_from_snapshot(...)` behavior remains valid.
- Existing runtime snapshot contracts in `zircon_runtime_interface::ui::surface::diagnostics` remain stable.

### Implementation Slices

- [ ] **M1.1 Re-export overlay state for crate-local builder use**

  Modify `zircon_editor/src/ui/workbench/debug_reflector/mod.rs`:

  ```rust
  mod export;
  mod model;
  mod overlay;
  mod selection;

  pub(crate) use model::EditorUiDebugReflectorModel;
  pub(crate) use overlay::EditorUiDebugReflectorOverlayState;

  #[cfg(test)]
  mod tests;
  ```

- [ ] **M1.2 Add active snapshot field to payload build context**

  Modify `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_presentation.rs`.

  Add this import near the top:

  ```rust
  use zircon_runtime_interface::ui::surface::UiSurfaceDebugSnapshot;
  ```

  Update `PanePayloadBuildContext`:

  ```rust
  #[derive(Clone)]
  pub(crate) struct PanePayloadBuildContext<'a> {
      pub chrome: &'a EditorChromeSnapshot,
      pub animation_pane: Option<&'a AnimationEditorPanePresentation>,
      pub runtime_diagnostics:
          Option<&'a zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
      pub active_ui_debug_snapshot: Option<&'a UiSurfaceDebugSnapshot>,
      pub module_plugins: Option<&'a ModulePluginsPaneViewData>,
      pub build_export: Option<&'a BuildExportPaneViewData>,
  }
  ```

  Update `new(...)`:

  ```rust
  pub fn new(chrome: &'a EditorChromeSnapshot) -> Self {
      Self {
          chrome,
          animation_pane: None,
          runtime_diagnostics: None,
          active_ui_debug_snapshot: None,
          module_plugins: None,
          build_export: None,
      }
  }
  ```

  Add this builder method after `with_runtime_diagnostics(...)`:

  ```rust
  pub fn with_active_ui_debug_snapshot(
      mut self,
      snapshot: &'a UiSurfaceDebugSnapshot,
  ) -> Self {
      self.active_ui_debug_snapshot = Some(snapshot);
      self
  }
  ```

- [ ] **M1.3 Prefer active snapshot in Runtime Diagnostics payload builder**

  Modify `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs`.

  Replace the imports:

  ```rust
  use crate::ui::workbench::debug_reflector::{
      EditorUiDebugReflectorModel, EditorUiDebugReflectorOverlayState,
  };
  ```

  Replace the current reflector construction in `build(...)`:

  ```rust
  let (reflector, overlay_primitives) = context
      .active_ui_debug_snapshot
      .map(|snapshot| {
          (
              EditorUiDebugReflectorModel::from_snapshot(snapshot),
              EditorUiDebugReflectorOverlayState::default().primitives_from_snapshot(snapshot),
          )
      })
      .unwrap_or_else(|| (EditorUiDebugReflectorModel::no_active_surface(), Vec::new()));
  ```

  Use `overlay_primitives` in the payload:

  ```rust
  ui_debug_reflector_overlay_primitives: overlay_primitives,
  ```

- [ ] **M1.4 Add pane payload builder tests for active snapshot and fallback**

  Modify `zircon_editor/src/tests/host/pane_presentation.rs`.

  Add imports if missing:

  ```rust
  use zircon_runtime_interface::ui::{
      event_ui::{UiNodeId, UiNodePath, UiTreeId},
      layout::UiFrame,
      surface::{
          UiDebugOverlayPrimitive, UiDebugOverlayPrimitiveKind, UiRenderDebugStats,
          UiSurfaceDebugCaptureContext, UiSurfaceDebugSnapshot, UiWidgetReflectorNode,
      },
      tree::{UiInputPolicy, UiVisibility},
  };
  ```

  Add this fixture near the existing runtime diagnostics fixture helpers:

  ```rust
  fn active_ui_debug_snapshot_fixture() -> UiSurfaceDebugSnapshot {
      UiSurfaceDebugSnapshot {
          capture: UiSurfaceDebugCaptureContext {
              surface_name: Some("Runtime Diagnostics fixture".to_string()),
              selected_node: Some(UiNodeId::new(2)),
              ..UiSurfaceDebugCaptureContext::default()
          },
          tree_id: UiTreeId::new("editor.runtime_diagnostics.active_debug"),
          roots: vec![UiNodeId::new(1)],
          nodes: vec![
              UiWidgetReflectorNode {
                  node_id: UiNodeId::new(1),
                  node_path: UiNodePath::new("runtime/root"),
                  parent: None,
                  children: vec![UiNodeId::new(2)],
                  frame: UiFrame::new(0.0, 0.0, 120.0, 80.0),
                  clip_frame: UiFrame::new(0.0, 0.0, 120.0, 80.0),
                  z_index: 0,
                  paint_order: 0,
                  visibility: UiVisibility::Visible,
                  input_policy: UiInputPolicy::Ignore,
                  enabled: true,
                  clickable: false,
                  hoverable: false,
                  focusable: false,
                  control_id: Some("RuntimeDiagnosticsRoot".to_string()),
                  render_command_count: 1,
                  hit_entry_count: 0,
                  hit_cell_count: 0,
              },
              UiWidgetReflectorNode {
                  node_id: UiNodeId::new(2),
                  node_path: UiNodePath::new("runtime/root/live_button"),
                  parent: Some(UiNodeId::new(1)),
                  children: Vec::new(),
                  frame: UiFrame::new(8.0, 12.0, 64.0, 24.0),
                  clip_frame: UiFrame::new(8.0, 12.0, 64.0, 24.0),
                  z_index: 1,
                  paint_order: 1,
                  visibility: UiVisibility::Visible,
                  input_policy: UiInputPolicy::Receive,
                  enabled: true,
                  clickable: true,
                  hoverable: true,
                  focusable: true,
                  control_id: Some("LiveDebugButton".to_string()),
                  render_command_count: 2,
                  hit_entry_count: 1,
                  hit_cell_count: 1,
              },
          ],
          render: UiRenderDebugStats {
              command_count: 3,
              estimated_draw_calls: 3,
              ..UiRenderDebugStats::default()
          },
          overlay_primitives: vec![UiDebugOverlayPrimitive {
              kind: UiDebugOverlayPrimitiveKind::SelectedFrame,
              node_id: Some(UiNodeId::new(2)),
              frame: UiFrame::new(8.0, 12.0, 64.0, 24.0),
              label: Some("live".to_string()),
              severity: None,
          }],
          ..UiSurfaceDebugSnapshot::default()
      }
  }
  ```

  Add this focused test after `pane_payload_builders_emit_stable_body_metadata_for_first_wave_views`:

  ```rust
  #[test]
  fn runtime_diagnostics_payload_uses_active_ui_debug_snapshot_when_available() {
      let chrome = chrome_fixture();
      let runtime_diagnostics = runtime_diagnostics_fixture();
      let active_snapshot = active_ui_debug_snapshot_fixture();
      let context = PanePayloadBuildContext::new(&chrome)
          .with_runtime_diagnostics(&runtime_diagnostics)
          .with_active_ui_debug_snapshot(&active_snapshot);

      let body = build_pane_body_presentation(
          &pane_body_spec("editor.runtime_diagnostics"),
          &context,
      );

      let PanePayload::RuntimeDiagnosticsV1(payload) = body.payload else {
          panic!("expected runtime diagnostics payload");
      };

      assert_eq!(payload.summary, "3 runtime systems available");
      assert_eq!(
          payload.ui_debug_reflector_summary,
          "UI Debug Reflector: 2 nodes, 3 commands, schema v1"
      );
      assert!(payload
          .ui_debug_reflector_nodes
          .iter()
          .any(|node| node.contains("runtime/root/live_button") && node.contains("node=2")));
      assert!(payload
          .ui_debug_reflector_details
          .iter()
          .any(|detail| detail.contains("Selected: runtime/root/live_button")));
      assert!(payload
          .ui_debug_reflector_export_status
          .contains("JSON export ready"));
      assert_eq!(payload.ui_debug_reflector_overlay_primitives.len(), 1);
      assert_eq!(
          payload.ui_debug_reflector_overlay_primitives[0].kind,
          UiDebugOverlayPrimitiveKind::SelectedFrame
      );
  }
  ```

  Keep existing assertions for the no-active fallback unchanged in existing tests. Those existing assertions prove fallback remains stable.

- [ ] **M1.5 Add template projection test for active snapshot attributes**

  Modify `zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs`.

  Add a compact local fixture or import the same constructors used in `pane_presentation.rs`. Use a self-contained fixture in this file to avoid cross-test coupling.

  Add imports if missing:

  ```rust
  use zircon_runtime_interface::ui::{
      event_ui::{UiNodeId, UiNodePath, UiTreeId},
      layout::UiFrame,
      surface::{UiRenderDebugStats, UiSurfaceDebugCaptureContext, UiSurfaceDebugSnapshot, UiWidgetReflectorNode},
      tree::{UiInputPolicy, UiVisibility},
  };
  ```

  Add a fixture equivalent to `active_ui_debug_snapshot_fixture()` with `tree_id = "editor.runtime_diagnostics.projection_debug"` and the selected node path `runtime/projection/live_label`.

  In `editor_ui_host_runtime_projects_pane_body_payload_metadata_into_root_attributes`, change context construction to:

  ```rust
  let active_snapshot = active_ui_debug_snapshot_fixture();
  let context = PanePayloadBuildContext::new(&chrome)
      .with_animation_pane(&animation)
      .with_runtime_diagnostics(&runtime_diagnostics)
      .with_active_ui_debug_snapshot(&active_snapshot);
  ```

  Replace the no-active assertions for Runtime Diagnostics with:

  ```rust
  assert_eq!(
      diagnostics_projection
          .root
          .attributes
          .get("payload_ui_debug_reflector_summary"),
      Some(&Value::String(
          "UI Debug Reflector: 2 nodes, 2 commands, schema v1".to_string()
      ))
  );
  assert!(diagnostics_projection
      .root
      .attributes
      .get("payload_ui_debug_reflector_export_status")
      .and_then(Value::as_str)
      .is_some_and(|text| text.contains("JSON export ready")));
  assert!(diagnostics_projection
      .root
      .attributes
      .get("payload_ui_debug_reflector_details")
      .and_then(Value::as_array)
      .is_some_and(|details| details.iter().any(|detail| {
          detail
              .as_str()
              .is_some_and(|text| text.contains("Selected: runtime/projection/live_label"))
      })));
  ```

- [ ] **M1.6 Add host conversion assertion for payload snapshot overlays**

  Modify `zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs`.

  Add a test that uses `runtime_diagnostics_pane_with_overlay(...)` and asserts both generated nodes and payload overlay survive conversion:

  ```rust
  #[test]
  fn runtime_diagnostics_host_conversion_keeps_payload_reflector_text_and_overlay() {
      let pane = runtime_diagnostics_pane_with_overlay(UiDebugOverlayPrimitive {
          kind: UiDebugOverlayPrimitiveKind::SelectedFrame,
          node_id: Some(UiNodeId::new(7)),
          frame: UiFrame::new(6.0, 7.0, 40.0, 18.0),
          label: Some("from-payload".to_string()),
          severity: None,
      });

      let projected =
          to_host_contract_runtime_diagnostics_pane_from_host_pane(&pane, pane_size(220.0, 120.0));

      assert!(model_texts(&projected.nodes)
          .iter()
          .any(|text| text == "summary"));
      assert_eq!(projected.overlay_primitives.row_count(), 1);
      let primitive = projected
          .overlay_primitives
          .row_data(0)
          .expect("payload overlay primitive should project");
      assert_eq!(primitive.kind, UiDebugOverlayPrimitiveKind::SelectedFrame);
      assert_eq!(primitive.node_id.as_str(), "7");
      assert_eq!(primitive.label.as_str(), "from-payload");
  }
  ```

  This test is intentionally payload-level. Do not edit the native painter in this slice.

- [ ] **M1.7 Update Debug Reflector docs**

  Modify `docs/zircon_editor/ui/workbench/debug_reflector.md`:

  Add this plan source under `plan_sources`:

  ```yaml
  - docs/superpowers/specs/2026-05-07-debug-observatory-design.md
  - docs/superpowers/plans/2026-05-07-debug-observatory-m0-m1.md
  ```

  Add a paragraph under `## Runtime Diagnostics Pane`:

  ```markdown
  Debug Observatory M1 adds a direct payload-build seam for an explicitly supplied active `UiSurfaceDebugSnapshot`. When `PanePayloadBuildContext::with_active_ui_debug_snapshot(...)` is used, Runtime Diagnostics projects the supplied snapshot into summary rows, details, export status, and shared overlay primitives before host conversion. When no active snapshot is supplied, the pane keeps the stable no-active placeholder and the later host-body refresh path can still derive a diagnostics-only snapshot from the pane's own `body_surface_frame`.
  ```

- [ ] **M1.8 Update acceptance record after implementation**

  In `tests/acceptance/ui-debug-observatory.md`, replace `Pending implementation.` under `## M1 Evidence` with a bullet list of implemented files and leave command results as pending until the testing stage runs:

  ```markdown
  ## M1 Evidence

  Implemented source changes:

  - `PanePayloadBuildContext` can carry an active borrowed `UiSurfaceDebugSnapshot`.
  - Runtime Diagnostics payload builder uses the active snapshot when present and preserves no-active fallback when absent.
  - Runtime Diagnostics payload builder derives overlay primitives through `EditorUiDebugReflectorOverlayState` so the same shared overlay filters are used by pane payloads and tests.
  - Focused tests cover active snapshot projection, fallback behavior, template root attributes, and payload overlay host conversion.

  Validation commands:

  - Pending M1.T.
  ```

### Lightweight Checks Before M1 Testing Stage

Only run this if the implementation feels mechanically risky before the full M1 testing stage:

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: compile succeeds or any failure is classified. Do not treat sibling-owned failures as this milestone's failure without checking the active session notes.

### Testing Stage: M1.T Live Snapshot Feed Validation

Run these commands after all M1 implementation slices and docs are complete:

```powershell
rustfmt --edition 2021 --check "zircon_editor/src/ui/workbench/debug_reflector/mod.rs" "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_presentation.rs" "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs" "zircon_editor/src/tests/host/pane_presentation.rs" "zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs" "zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs"
```

Expected result: no formatting diff.

```powershell
cargo test -p zircon_editor --lib pane_payload_builders_emit_stable_body_metadata_for_first_wave_views --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: existing first-wave builder test passes and still proves no-active fallback for cases without active snapshot.

```powershell
cargo test -p zircon_editor --lib runtime_diagnostics_payload_uses_active_ui_debug_snapshot_when_available --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: active snapshot builder test passes.

```powershell
cargo test -p zircon_editor --lib pane_payload_projection --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: template projection tests pass, including active snapshot root attributes.

```powershell
cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: Debug Reflector model/overlay/host tests pass.

```powershell
cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: runtime snapshot generation remains green.

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never
```

Expected result: editor lib compile succeeds with only existing warning noise.

### Debug/Correction Loop

- If `pane_payload_builders_emit_stable_body_metadata_for_first_wave_views` fails because it now receives an active snapshot, make sure that test still builds a context without `with_active_ui_debug_snapshot(...)` except in the new active-snapshot test.
- If template projection fails on summary text, check the command count in the local fixture and align the expected string to the fixture's `UiRenderDebugStats.command_count`.
- If overlay primitive count is larger than expected, inspect whether `EditorUiDebugReflectorOverlayState::primitives_from_snapshot(...)` is deriving render visualizer or damage primitives from default fixture fields. Use a minimal fixture with only `overlay_primitives` and no damage/render visualizer entries for exact-count tests.
- If `cargo check -p zircon_editor` fails in input/window, Material/runtime-preview, root-frame cleanup, or broad painter code, read the active session note and record it as a sibling-owned blocker unless this M1 code directly caused the import/type error.

### Exit Evidence

- M1.T commands run and results are recorded in `tests/acceptance/ui-debug-observatory.md`.
- `docs/zircon_editor/ui/workbench/debug_reflector.md` documents the new active snapshot feed.
- `.codex/sessions/20260507-1202-debug-observatory-m0-m1.md` is updated with validation status and blockers.

## Completion And Handoff

- If M0/M1 completes with focused validation and no handoff is needed, delete `.codex/sessions/20260507-1202-debug-observatory-m0-m1.md` before final response.
- If focused validation passes but broad workspace validation remains blocked by sibling-owned areas, keep or archive the session note with exact blocker ownership and evidence.
- Do not start M2 Snapshot Timeline in this plan. Create a new plan for M2 after M1 is accepted.
