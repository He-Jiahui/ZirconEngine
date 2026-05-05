# UI Debug Reflector Full Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a complete Unreal Slate Widget Reflector-style debug loop for Zircon UI: shared snapshots, editor inspection panel, overlays, export/replay, and documentation.

**Architecture:** Use `UiSurfaceFrame` as the single spatial truth. `zircon_runtime_interface::ui::surface` owns serializable debug DTOs, `zircon_runtime::ui::surface` generates deterministic snapshots from arranged/render/hit/focus data, and `zircon_editor::ui` only consumes those snapshots for panel, overlay, and export/replay UI. Editor host damage/invalidation and renderer counters are optional contributors, not alternate layout or hit-test authorities.

**Tech Stack:** Rust, Cargo, Serde/serde_json, `.ui.toml`, `UiSurfaceFrame`, `UiSurfaceDebugSnapshot`, softbuffer native host diagnostics, existing editor workbench reflection/projection, Unreal Slate reference sources under `dev/UnrealEngine`.

---

## Execution Policy

- Work directly on `main` in the existing checkout. Do not create worktrees or feature branches.
- Do not commit unless the user explicitly asks for a commit.
- Preserve unrelated dirty work. The current worktree already contains many UI edits; read files before editing and do not revert changes outside this plan.
- Follow `zirconEngine` milestone-first cadence. Implementation slices may add production code, test code, comments, and docs; compile/build/unit-test execution belongs to each milestone's named testing stage unless a blocker requires a scoped `cargo check` earlier.
- If an editor panel or overlay test fails, diagnose shared snapshot generation first. Do not add editor-only geometry, hit-test, or draw-order reconstruction to make an upper layer pass.
- Keep root wiring files structural. If new debug DTOs or builders grow, split by declaration and behavior instead of adding large sections to `mod.rs` or broad files.
- `zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs` is 881 lines. Avoid adding more than a small call site there; put new overlay drawing helpers under a focused painter child module if overlay code grows.
- Do not copy Unreal APIs. Reference `SWidgetReflector`, `WidgetReflectorNode`, `SWidgetHittestGrid`, `SlateDebugging`, `FHittestGrid`, and `SlateInvalidationRoot` for responsibilities and evidence only.

## Current Baseline

- `zircon_runtime_interface/src/ui/surface/diagnostics.rs` already has `UiSurfaceDebugSnapshot`, `UiWidgetReflectorNode`, render stats, material batch stats with `break_reason`, hit-grid stats, overdraw summary, and rebuild stats.
- `zircon_runtime_interface/src/ui/surface/frame.rs` already includes `UiSurfaceFrame.last_rebuild`.
- `zircon_runtime/src/ui/surface/diagnostics.rs` already builds basic reflector nodes, material batches, overdraw summary, and rebuild stats from `UiSurfaceFrame`.
- `zircon_runtime/src/ui/surface/frame_hit_test.rs` already supports query-aware hit-test debug dumps with virtual pointer and cursor radius.
- `zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs`, `presenter.rs`, `redraw.rs`, and `app/invalidation.rs` already expose host refresh, damage, and invalidation counters.
- `zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.ui.toml` and the `RuntimeDiagnostics` pane already exist and can host the first Debug Reflector view without creating a new top-level editor system.

## File Structure

### Shared Runtime Interface

- Modify `zircon_runtime_interface/src/ui/surface/diagnostics.rs`: extend the debug schema with export metadata, selected/pick context, command records, hit cell records, overdraw cell records, invalidation/damage records, event records, and overlay primitives. Keep declaration-only type definitions here.
- Modify `zircon_runtime_interface/src/ui/surface/hit.rs`: replace string-only reject reasons with stable reason codes while preserving a human-readable message field.
- Modify `zircon_runtime_interface/src/ui/surface/mod.rs`: re-export new DTOs only.
- Modify `zircon_runtime_interface/src/tests/contracts.rs`: add serialization/roundtrip contract coverage for the expanded schema.

### Runtime Snapshot Generation

- Modify `zircon_runtime/src/ui/surface/diagnostics.rs`: keep the public entry points, but split internal logic when adding command/cell/overlay/export builders.
- Create `zircon_runtime/src/ui/surface/diagnostics/render_records.rs` if command-record generation makes `diagnostics.rs` exceed a single responsibility.
- Create `zircon_runtime/src/ui/surface/diagnostics/hit_records.rs` for hit cell, entry, and reject-code conversion.
- Create `zircon_runtime/src/ui/surface/diagnostics/overdraw_cells.rs` for per-cell overdraw sampling and top contributor extraction.
- Create `zircon_runtime/src/ui/surface/diagnostics/overlay.rs` for snapshot-derived overlay primitive generation.
- Modify `zircon_runtime/src/ui/surface/mod.rs`: re-export only public debug entry points and keep it structural.
- Modify `zircon_runtime/src/ui/surface/surface.rs`: expose helper methods for debug snapshot with pick context and export payload, but keep retained tree mutation outside debug code.
- Modify `zircon_runtime/src/ui/tests/diagnostics.rs`: add focused runtime tests for command records, hit cells, overdraw cells, overlay primitives, and JSON export roundtrip.

### Editor Reflector Consumer

- Create `zircon_editor/src/ui/workbench/debug_reflector/mod.rs`: structural module wiring only.
- Create `zircon_editor/src/ui/workbench/debug_reflector/model.rs`: editor-facing read model derived from `UiSurfaceDebugSnapshot`.
- Create `zircon_editor/src/ui/workbench/debug_reflector/selection.rs`: selected-node and pick-state handling.
- Create `zircon_editor/src/ui/workbench/debug_reflector/export.rs`: JSON export/import wrappers and user-visible errors.
- Create `zircon_editor/src/ui/workbench/debug_reflector/overlay.rs`: editor overlay toggle state and conversion from shared overlay primitives to host presentation data.
- Modify `zircon_editor/src/ui/workbench/mod.rs`: add `pub mod debug_reflector;` only.
- Modify `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs`: add a `UiDebugReflectorV1` payload or extend `RuntimeDiagnosticsPanePayload` only if the first implementation intentionally embeds reflector into Runtime Diagnostics.
- Modify `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs`: include a debug-reflector summary section from the latest available UI snapshot when embedding into Runtime Diagnostics.
- Modify `zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs`: project debug-reflector fields into template attributes.
- Modify `zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.ui.toml`: add a Debug Reflector section with tree/detail/render/hit/overdraw/invalidation summaries while preserving the existing Focus Diagnostics action.
- Modify `zircon_editor/src/ui/slint_host/host_contract/data/panes.rs`: add host payload fields only if native painter needs structured reflector rows.
- Modify `zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs`: add only the call site for reflector overlay drawing.
- Create `zircon_editor/src/ui/slint_host/host_contract/painter/debug_reflector_overlay.rs`: draw selected frame, clip frame, hit cells, overdraw cells, material bounds, reject bounds, and damage region.
- Modify `zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs`: wire the overlay drawing helper.
- Modify `zircon_editor/src/tests/host/slint_window/native_host_contract.rs`: cover native host reflector payload/overlay behavior.
- Add `zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs` and register it in `zircon_editor/src/tests/host/slint_window/mod.rs` if the test surface becomes too large for `native_host_contract.rs`.

### Documentation And Acceptance

- Modify `docs/ui-and-layout/slate-style-ui-surface-frame.md`: document the completed debug snapshot, export/replay, overlay, and editor panel path.
- Modify `docs/ui-and-layout/index.md`: add the debug reflector closure to the UI docs routing text.
- Create or update `docs/zircon_runtime/ui/surface/diagnostics.md` if new runtime diagnostics modules are created and no existing doc owns the behavior.
- Create or update `docs/zircon_editor/ui/workbench/debug_reflector.md` when the editor debug-reflector module is added.
- Create `tests/acceptance/ui-debug-reflector-full-closure.md`: record milestone validation commands, accepted risks, and export/replay/overlay acceptance evidence.

## Milestone 1: Shared Snapshot Schema

- Goal: Expand the neutral debug DTO contract so runtime and editor can exchange complete reflector snapshots without editor-only geometry reconstruction.
- In-scope behaviors: schema version, capture context, selected node, pick query, command records, hit cells, overdraw cells, invalidation/damage diagnostics, event records, overlay primitive DTOs, stable reject reason codes, serialization defaults.
- Dependencies: existing `UiSurfaceDebugSnapshot`, `UiHitTestDebugDump`, `UiRenderCommand`, `UiHitTestGrid`, `UiDirtyFlags`, and `UiFocusState`.

### Implementation Slices

- [ ] Add `pub const UI_SURFACE_DEBUG_SCHEMA_VERSION: u32 = 1` in `zircon_runtime_interface/src/ui/surface/diagnostics.rs`.
- [ ] Extend `UiSurfaceDebugOptions` with `include_command_records`, `include_hit_cells`, `include_overdraw_cells`, and `include_overlay_primitives` booleans. Default all four to `true` to make editor live mode useful without custom options.
- [ ] Add `UiSurfaceDebugCaptureContext` with fields `schema_version: u32`, `surface_name: Option<String>`, `source_asset: Option<String>`, `frame_index: Option<u64>`, `captured_at_millis: Option<u64>`, `selected_node: Option<UiNodeId>`, and `pick_query: Option<UiHitTestQuery>`.
- [ ] Add `capture: UiSurfaceDebugCaptureContext` to `UiSurfaceDebugSnapshot` with `#[serde(default)]` so older tests can still construct default snapshots during migration.
- [ ] Add `UiRenderCommandDebugRecord` with `command_id: u64`, `node_id`, `kind`, `frame`, `clip_frame`, `visible_frame`, `z_index`, `paint_order: u64`, `opacity`, `material_key`, `batch_key`, `batch_break_reason`, `estimated_draw_calls`, `text_summary`, and `image_summary`.
- [ ] Add `command_records: Vec<UiRenderCommandDebugRecord>` and `measured: Option<UiBackendRenderDebugStats>` to `UiRenderDebugStats`. `UiBackendRenderDebugStats` contains optional `submitted_draw_calls`, `pipeline_switches`, `texture_switches`, `glyph_batches`, and `clipped_batches`.
- [ ] Add `UiHitGridCellDebugRecord` with `cell_id`, `bounds`, `entry_indices`, and `entry_node_ids`.
- [ ] Add `cell_records: Vec<UiHitGridCellDebugRecord>` to `UiHitGridDebugStats`.
- [ ] Replace `UiHitTestReject.reason: String` with `reason: UiHitTestRejectReason` and `message: String`. Define enum variants `OutsideFrame`, `OutsideClip`, `VisibilityFiltered`, `Disabled`, `InputPolicyIgnore`, `NotPointerTarget`, `MissingAncestry`, `StaleGridEntry`, and `CustomHitPathUnavailable`.
- [ ] Add `UiOverdrawCellDebugRecord` with `cell_id`, `bounds`, `layer_count`, and `node_ids`.
- [ ] Add `cells: Vec<UiOverdrawCellDebugRecord>` to `UiOverdrawDebugStats`.
- [ ] Add `UiInvalidationDebugReport` with `rebuild: UiSurfaceRebuildDebugStats`, `dirty_flags`, `dirty_node_count`, `recompute_reasons: Vec<String>`, and `warnings: Vec<String>`.
- [ ] Add `UiDamageDebugReport` with `damage_region: Option<UiFrame>`, `painted_pixels: Option<u64>`, `full_paint_count: Option<u64>`, `region_paint_count: Option<u64>`, `total_painted_pixels: Option<u64>`, and `warnings: Vec<String>`.
- [ ] Add `UiDebugEventRecord` with `event_id`, `kind`, `node_id`, `route`, `summary`, and `handled`.
- [ ] Add `UiDebugOverlayPrimitive` plus a small `UiDebugOverlayPrimitiveKind` enum covering `SelectedFrame`, `ClipFrame`, `HitCell`, `HitPath`, `RejectedBounds`, `OverdrawCell`, `MaterialBatchBounds`, and `DamageRegion`.
- [ ] Add `invalidation`, `damage`, `events`, and `overlay_primitives` fields to `UiSurfaceDebugSnapshot` with serde defaults.
- [ ] Update `zircon_runtime_interface/src/ui/surface/mod.rs` re-exports for every new public DTO.
- [ ] Update `zircon_runtime_interface/src/tests/contracts.rs::ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats` to construct the new fields explicitly and assert JSON contains `schema_version`, `command_records`, `cell_records`, `overdraw`, `overlay_primitives`, and a reject reason code.

### Testing Stage: Shared Schema Gate

- [ ] Run `cargo test -p zircon_runtime_interface --lib ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --message-format short --color never`.
- [ ] If either command fails, fix the lowest shared DTO or serde issue first, then rerun both commands.
- [ ] Record exact commands, pass/fail output, and any remaining schema risk in `tests/acceptance/ui-debug-reflector-full-closure.md`.

### Lightweight Checks

- During implementation, a scoped `cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --message-format short --color never` is allowed if compile errors block further editing.

### Exit Evidence

- Runtime-interface contract test passes.
- `UiSurfaceDebugSnapshot` can be serialized with all new schema sections.
- `UiHitTestRejectReason` is a stable enum rather than string-only debug text.

## Milestone 2: Runtime Snapshot Generation And Export

- Goal: Generate the expanded debug snapshot from `UiSurfaceFrame` and export/import it as deterministic JSON.
- In-scope behaviors: command records, material batch break reasons, hit cell records, stable hit reject reasons, sampled overdraw cells, overlay primitive records, selected node/pick query capture, optional backend counter placeholders, JSON roundtrip.
- Dependencies: Milestone 1 schema and existing `debug_surface_frame_with_options(...)`, `debug_hit_test_surface_frame_with_query(...)`, and `UiSurface::surface_frame()`.

### Implementation Slices

- [ ] Update `debug_surface_frame_with_options(...)` to fill `capture` with schema version and options defaults while keeping `surface_name`, `source_asset`, and timestamp fields optional.
- [ ] Add `debug_surface_frame_for_selection(surface_frame, selected_node, options)` to build selected-node overlay primitives without requiring editor code to mutate the snapshot.
- [ ] Add `debug_surface_frame_for_pick(surface_frame, query, options)` to attach `pick_query`, hit-test dump, selected top-hit node, reject records, and hit-path overlay primitives.
- [ ] Generate `UiRenderCommandDebugRecord` from each `UiRenderCommand`. Use command index as `command_id`, `command_visible_frame(...)` for `visible_frame`, `material_batch_key(...)` for `material_key` and `batch_key`, `material_batch_break_reason(...)` for `batch_break_reason`, and current CPU estimate for `estimated_draw_calls`.
- [ ] Generate `UiHitGridCellDebugRecord` from `surface_frame.hit_grid.cells`. Compute cell bounds from grid bounds, cell size, row, and column.
- [ ] Convert every hit-test reject path in `frame_hit_test.rs` to `UiHitTestRejectReason` with a clear `message` string. Do not keep string matching as the primary reason contract.
- [ ] Generate `UiOverdrawCellDebugRecord` for covered sample cells. Include node ids for commands whose visible frames contribute to the cell when practical; otherwise include an empty list and keep aggregate counts correct.
- [ ] Generate overlay primitives from selected node, hit path, hit cells, overdraw cells, material batch bounds, reject bounds, and damage report when data exists.
- [ ] Add `UiSurface::debug_snapshot_for_pick(query, options)` and `UiSurface::debug_snapshot_for_selection(selected_node, options)` convenience methods that call through `surface_frame()`.
- [ ] Add `UiSurface::debug_snapshot_json(...) -> Result<String, serde_json::Error>` and `UiSurfaceDebugSnapshot` roundtrip test helpers in runtime tests. Keep file I/O out of `zircon_runtime`; editor owns choosing export paths.
- [ ] Update `zircon_runtime/src/ui/tests/diagnostics.rs` with tests named `surface_debug_snapshot_reports_command_records_and_hit_cells`, `surface_debug_snapshot_reports_stable_reject_reason_codes`, `surface_debug_snapshot_reports_overdraw_cells_and_overlay_primitives`, and `surface_debug_snapshot_json_roundtrips_export_payload`.
- [ ] Keep `zircon_runtime/src/ui/surface/mod.rs` as public wiring only. If diagnostics logic grows, split to the child modules listed in File Structure.

### Testing Stage: Runtime Generation Gate

- [ ] Run `cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`.
- [ ] If a diagnostics test fails through editor-like behavior, re-check `UiSurfaceFrame`, hit-grid, and render-extract generation before editing any editor code.
- [ ] Record commands, failures fixed, and JSON roundtrip evidence in `tests/acceptance/ui-debug-reflector-full-closure.md`.

### Lightweight Checks

- A scoped `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never` is allowed before the testing stage if new module wiring blocks editing.

### Exit Evidence

- Expanded snapshots are produced entirely from `UiSurfaceFrame` plus optional selected/pick context.
- JSON export roundtrips in runtime unit tests.
- Hit reject reasons use stable enum codes and human-readable messages.
- Overlay primitives exist without editor host geometry reconstruction.

## Milestone 3: Editor Debug Reflector Model And Pane

- Goal: Add the editor Debug Reflector panel surface using the shared snapshot as its only UI debug authority.
- In-scope behaviors: live model, loaded snapshot model, selected-node summary, tree rows, render/hit/overdraw/invalidation/damage summaries, export/import errors, and initial integration into the existing Runtime Diagnostics pane.
- Dependencies: Milestone 2 runtime snapshot generation and existing Runtime Diagnostics pane projection.

### Implementation Slices

- [ ] Add `zircon_editor/src/ui/workbench/debug_reflector/model.rs` with `EditorUiDebugReflectorModel`, `EditorUiDebugReflectorNodeRow`, `EditorUiDebugReflectorSection`, and `EditorUiDebugReflectorSummary`. These types should store strings and ids ready for pane projection, not references to live runtime objects.
- [ ] Add a constructor `EditorUiDebugReflectorModel::from_snapshot(snapshot: &UiSurfaceDebugSnapshot) -> Self` that builds tree rows, selected-node summary, render summary, hit summary, overdraw summary, invalidation summary, and damage summary.
- [ ] Add `selection.rs` with `EditorUiDebugReflectorSelection { selected_node: Option<UiNodeId>, pick_point: Option<UiPoint> }` and helpers to select the top-hit node from a snapshot.
- [ ] Add `export.rs` with `load_snapshot_json(text: &str) -> Result<UiSurfaceDebugSnapshot, EditorUiDebugReflectorExportError>` and `snapshot_to_json(snapshot: &UiSurfaceDebugSnapshot) -> Result<String, EditorUiDebugReflectorExportError>`.
- [ ] Add tests for `from_snapshot`, stale selected-node handling, import warning behavior for unknown ids, and JSON parse errors.
- [ ] Extend `RuntimeDiagnosticsPanePayload` with `ui_debug_reflector_summary: String`, `ui_debug_reflector_nodes: Vec<String>`, `ui_debug_reflector_details: Vec<String>`, and `ui_debug_reflector_export_status: String` unless a dedicated `UiDebugReflectorV1` payload is less invasive in the current pane pipeline.
- [ ] Update `pane_payload_builders/runtime_diagnostics.rs` to include a placeholder reflector state when no active `UiSurfaceDebugSnapshot` is available: `UI Debug Reflector: no active surface snapshot`.
- [ ] If an active pane `UiSurfaceFrame` is available through `PaneData.body_surface_frame` or toolbar frame, build a live `UiSurfaceDebugSnapshot` and feed it into `EditorUiDebugReflectorModel`. If no shared frame is available, do not derive geometry from host rectangles.
- [ ] Update `pane_payload_projection.rs` to project reflector summary, node rows, details, and export status into template attributes.
- [ ] Update `runtime_diagnostics_body.ui.toml` to render the reflector section from those attributes. Keep this text/list based for the first closure; do not add a complex custom widget.
- [ ] Update `zircon_editor/src/ui/workbench/mod.rs` to expose `debug_reflector` and keep the file structural.
- [ ] Add editor tests in `zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs` for no-active-surface state, snapshot-derived rows, and JSON export/import projection.
- [ ] Register the new test module in `zircon_editor/src/tests/host/slint_window/mod.rs`.

### Testing Stage: Editor Panel Gate

- [ ] Run `cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib pane_body_documents --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`.
- [ ] If a pane projection test fails because data is missing, fix the shared snapshot availability or pane payload projection. Do not add a separate coordinate table.
- [ ] Record panel acceptance and any unavailable active-surface limitations in `tests/acceptance/ui-debug-reflector-full-closure.md`.

### Lightweight Checks

- A scoped `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` is allowed if new module wiring or payload type changes block further editing.

### Exit Evidence

- Runtime Diagnostics includes a Debug Reflector section.
- The section can display a no-active-surface state without panic.
- When given a snapshot, the editor model shows tree rows and selected-node/render/hit/overdraw/invalidation details.
- JSON export/import works through editor model helpers.

## Milestone 4: Snapshot-Derived Debug Overlay

- Goal: Draw editor debug overlays from shared snapshot primitives, not from editor-local geometry reconstruction.
- In-scope behaviors: selected frame, clip frame, hit cells, hit path, rejected bounds, overdraw heat cells, material batch bounds, and damage region toggles.
- Dependencies: Milestone 2 overlay primitive records and Milestone 3 editor model/selection state.

### Implementation Slices

- [ ] Add overlay toggle state to `zircon_editor/src/ui/workbench/debug_reflector/overlay.rs`: selected frame, clip frame, hit grid, hit path, rejected bounds, overdraw, material batches, and damage.
- [ ] Convert `UiDebugOverlayPrimitive` into host painter records with frame, label, color class, and opacity. Keep this conversion independent of `HostWindowPresentationData` internals where possible.
- [ ] Add fields to the relevant host presentation data structs only when required by the native painter. Prefer a single `ui_debug_overlay_primitives` vector over multiple unrelated fields.
- [ ] Create `zircon_editor/src/ui/slint_host/host_contract/painter/debug_reflector_overlay.rs` to draw thin rectangles and labels for overlay primitives.
- [ ] Modify `painter/mod.rs` to expose `draw_debug_reflector_overlay(...)` internally.
- [ ] Modify `painter/workbench.rs` only to call the overlay helper after normal content and before the existing refresh-rate marker, so overlay does not affect layout or hit-test.
- [ ] Add tests that verify overlay primitives are generated from snapshot node frames and overdraw cells, and that disabled toggles suppress corresponding painter records.
- [ ] Add tests that verify damage-region overlay uses `UiDamageDebugReport.damage_region` when present and shows no damage primitive when absent.

### Testing Stage: Overlay Gate

- [ ] Run `cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib shell_window --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`.
- [ ] If overlay rendering fails, verify shared overlay primitive content first, then painter conversion, then native painter drawing.
- [ ] Record overlay acceptance in `tests/acceptance/ui-debug-reflector-full-closure.md`.

### Lightweight Checks

- Before touching `painter/workbench.rs`, re-check its line count. If overlay logic would push it above roughly 900 lines with a second responsibility, extract `debug_reflector_overlay.rs` first.

### Exit Evidence

- Overlay primitives come from `UiSurfaceDebugSnapshot`.
- Native painter draws overlay records without changing layout, hit testing, render extract, or dispatch state.
- Tests cover enabled and disabled overlay toggles.

## Milestone 5: Documentation, Acceptance, And Final Validation

- Goal: Close the debug reflector milestone with docs, acceptance records, and scoped-to-broad validation evidence.
- In-scope behaviors: docs updated with related-code headers, acceptance file completed, scoped package tests run, workspace validation decision recorded.
- Dependencies: Milestones 1-4 complete.

### Implementation Slices

- [ ] Update `docs/ui-and-layout/slate-style-ui-surface-frame.md` with the final snapshot schema, command records, hit cell records, overdraw cells, invalidation/damage reports, overlay primitives, editor panel, export, and replay/import behavior.
- [ ] Update `docs/ui-and-layout/index.md` so readers can find the debug reflector section from the UI and Layout overview.
- [ ] Create `docs/zircon_runtime/ui/surface/diagnostics.md` if new `zircon_runtime/src/ui/surface/diagnostics/*` modules exist. Include machine-readable frontmatter with all related code files, implementation files, plan sources, and tests.
- [ ] Create `docs/zircon_editor/ui/workbench/debug_reflector.md` if `zircon_editor/src/ui/workbench/debug_reflector/*` exists. Include machine-readable frontmatter.
- [ ] Complete `tests/acceptance/ui-debug-reflector-full-closure.md` with schema, runtime generation, editor panel, overlay, export/import, commands run, failures fixed, and accepted limitations.
- [ ] Update `docs/superpowers/specs/2026-05-06-ui-debug-reflector-full-closure-design.md` only if implementation intentionally diverges from the approved design.

### Testing Stage: Final Acceptance Gate

- [ ] Run `cargo test -p zircon_runtime_interface --lib ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`.
- [ ] Run `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`.
- [ ] If shared public DTOs changed in a way that affects broad workspace consumers, run `cargo build --workspace --locked --verbose` and `cargo test --workspace --locked --verbose` or the repository validator `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1` with an explicit target directory.
- [ ] If disk space is low or workspace validation is too expensive for the current handoff, record that workspace validation remains open and list the scoped commands that did run.

### Lightweight Checks

- No additional lightweight checks. This is the testing stage.

### Exit Evidence

- Docs contain related-code headers and current implementation/test file lists.
- Acceptance file records export/import and overlay behavior.
- Scoped runtime-interface, runtime, and editor validation commands pass or failures are documented with fixes and remaining risks.
- Any skipped workspace-wide validation is explicitly reported as a remaining risk.

## Coverage Checklist

- Shared schema version and export context: Milestone 1.
- Command records and material batch break reasons: Milestones 1-2.
- Hit-grid cell records and stable reject reason codes: Milestones 1-2.
- Overdraw cell data: Milestones 1-2.
- Invalidation and damage reports: Milestones 1, 3, and 5.
- Editor panel live/no-active-surface/snapshot model: Milestone 3.
- JSON export/import and replay inspection model: Milestones 2-3.
- Snapshot-derived overlay: Milestone 4.
- Docs and acceptance evidence: Milestone 5.

## Out Of Scope

- Full keyboard, IME, drag/drop, popup, and text-shaping debug tools.
- GPU overdraw pass as a hard requirement. CPU sampled overdraw is accepted for this closure.
- Converting Zircon retained UI into Unreal live `SWidget` inheritance.
- Restoring old editor-only hit-test or coordinate-table paths.
