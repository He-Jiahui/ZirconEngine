# Assets Content Scroll and Hover Paint Transform Design

Status: approved by continued goal execution on 2026-07-10

## Context

The populated Assets Activity milestone now projects real rows and shares row geometry with the retained pointer bridge. The remaining response defect is below the layout layer:

- `AssetContentListPointerBridge` computes scroll offsets and hovered row indices correctly;
- `apply_asset_pointer_state_to_ui(...)` forwards those values;
- `PaneSurfaceHostContext::set_activity_asset_content_*` and the equivalent Browser setters are no-ops;
- template rows are painted at their authored frames and receive only the pane clip, so they cannot move or gain hover state from content interaction;
- `content_layout.rs` currently zeroes rows outside the initial viewport to prevent them painting over Preview, which also makes later scrolling unable to reveal them.

The normal path must be repaired at the shared state/paint boundary. The feature must not add a second native asset-row renderer or rebuild the full editor presentation for every pointer event.

## Considered Approaches

### A. Paint-time template-node transform (selected)

Store the real content interaction state, then apply a pane-scoped transform while collecting template-node paint commands. Asset content nodes keep stable source geometry; paint-time projection subtracts the scroll offset, intersects their clip with the content panel, and marks only the hovered row.

This keeps the component surface, text, badge, selected state, and runtime text path in the existing template painter. Pointer and visual geometry remain connected through the shared asset-content metrics.

### B. Native painter duplicates asset rows (rejected)

This would make hover and scrolling easy to draw, but it would duplicate row geometry, text compaction, badge styling, selection priority, and visual assets. It recreates the exact visual/pointer double ownership just removed by S15.4ov/S15.6nu.

### C. Rebuild pane presentation on every move or scroll (rejected)

This would reuse projection code but couples high-frequency pointer input to snapshot and presentation rebuilds. It is more expensive, complicates damage tracking, and turns transient interaction state into application presentation state.

## Architecture

### 1. Interaction state is real host state

`HostPaneInteractionStateData` gains four explicit fields:

- Activity content scroll offset;
- Activity content hovered row index;
- Browser content scroll offset;
- Browser content hovered row index.

The existing setters write these fields, clamp scroll offsets to non-negative values, and preserve `-1` as the no-hover sentinel. No compatibility fields or duplicate storage are introduced.

Activity is the first visual consumer in this slice. Browser state is stored at the same boundary so its existing pointer writeback is no longer discarded, but Browser table/thumbnail paint transformation remains a later migration because its current node identities and layout owners differ.

### 2. Generic template paint transform extension

The template-node pipeline gains an optional internal transform contract. It receives the owned `TemplatePaneNodeData` returned by the model, may adjust the node and node-specific clip, and may suppress an off-viewport node. Existing callers continue through the unchanged no-transform path.

The contract belongs in the folder-backed `paint_template_nodes/template_node_pipeline` owner. Asset-specific matching does not enter this generic module.

### 3. Activity asset-content projector

A new folder-backed leaf under `paint_workbench_renderer/docks/pane/template_nodes/asset_content/` owns Activity projection:

- locate `AssetsActivityContentPanel` in pane-local coordinates;
- recognize stable generated Activity content control ids for folder/item rows and their badge/name/meta children;
- count folder rows so item-local indices map to the pointer bridge's folder-first/item-second row order;
- subtract the current content scroll offset from every recognized content node frame;
- intersect the node clip with the content-panel frame translated into pane/body coordinates;
- set `hovered=true` only on the row whose shared row index matches interaction state;
- suppress nodes that do not intersect the content viewport after transformation.

Missing content panels or unknown node identities fall back to unchanged painting rather than hiding unrelated pane content.

### 4. Stable source geometry and clipping

`assets_activity/content_layout.rs` lays out every generated folder and item row, including rows below the initial viewport. It no longer sets overflow rows to zero width and height. The source model therefore represents the full scrollable list; the projector is the sole viewport clip owner.

This change is required for scrolling to reveal later rows. It also removes projection-time knowledge of the current paint viewport from row existence.

### 5. Hover and scrollbar visuals

Hover uses the existing `workbench-list-row` component state and style priority. It does not introduce custom colors or a native overlay rectangle.

The native shared vertical scrollbar receives an Activity content entry point. Its viewport comes from the projected content-panel frame, its extent comes from `AssetContentLayoutMetrics::list_height(...)`, and its active state follows content hover. Track/thumb dimensions continue through the existing Starship scrollbar owner.

## Data Flow

1. Pointer move/scroll enters the existing Activity content callback.
2. `AssetContentListPointerBridge` resolves the folder/item route and clamps scroll offset using shared metrics.
3. `apply_asset_pointer_state_to_ui("activity")` writes hovered row and scroll offset into `HostPaneInteractionStateData`.
4. The pane repaint selects the Activity asset-content projector.
5. The template pipeline transforms only Activity content nodes, applies the content clip, and paints existing component commands.
6. The native pane layer paints the shared content scrollbar from the same state and extent.

No editor snapshot rebuild and no second asset-row paint implementation occurs.

## Error and Boundary Handling

- Negative scroll values are clamped at the setter and pointer bridge boundaries.
- A missing content panel disables the transform for that pane and preserves ordinary painting.
- A stale hovered index outside the current row count produces no hovered row.
- Empty content has zero list extent, no scrollbar, and keeps its explicit empty-state label.
- Activity utility and toolbar nodes never match the content identity parser and remain unaffected.
- Browser visual transformation is explicitly outside this slice; only its state storage stops being a no-op.

## Testing and Evidence

Testing proceeds bottom-up:

1. State setter tests lock Activity/Browser content hover and scroll storage/clamping.
2. Identity and pure projection tests lock folder-first/item-second indices, scroll translation, node-specific clipping, hover assignment, missing-panel fallback, and unrelated-node pass-through.
3. Assets Activity layout tests lock non-zero geometry for rows below the initial viewport.
4. Painter tests prove an offscreen row is absent before scrolling, visible after scrolling, and receives the standard hover surface without affecting utility pixels.
5. Existing pointer tests re-run upward to prove scroll/click behavior still follows the normal bridge.
6. A dedicated scrolled/hovered Assets drawer artifact is written to `docs/tests/editor`; no screenshot is written under repository or external Cargo targets.
7. Rustfmt, diff, file-size, hard-cutover, magic-constant, plan-output, target scan, and scoped Cargo checks complete before plan status is marked done.

## Non-goals

- Replacing the Asset Browser table/thumbnail presentation in this slice;
- changing runtime text shaping or measurement ownership;
- adding compatibility shims for removed asset pointer metrics;
- rebuilding the full workbench presentation on pointer input;
- introducing local RGB values, concrete font families, or pixel-positioned window layouts.

## Acceptance Criteria

- Activity content hover and scroll setters are no longer no-ops.
- Scrolling can reveal rows that were below the initial viewport.
- Hover affects exactly one visible row through the shared list-row style.
- Content pixels are clipped to the content panel and never overlap Preview/References.
- Pointer hit geometry and painted row geometry agree at non-zero scroll offsets.
- The shared content scrollbar appears only when list extent exceeds the viewport.
- Focused tests and the scrolled visual artifact pass, and editor-layout status/output records are updated with concrete evidence.
