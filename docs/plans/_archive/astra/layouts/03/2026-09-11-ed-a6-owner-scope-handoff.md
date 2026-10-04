---
record_kind: dependency_handoff
status: blocked_owner_scope
created_at: 2026-09-11
plan: docs/plans/astra/layouts/03-per-leaf-workbench-projection.md
milestone: ED-A6 per-leaf projection and exact split-tree persistence
session: astra-ed-a6-owner-handoff-20260911-01a090c1
---

# ED-A6 owner-scope handoff

## Finding

The current tree/persistence work is present in the checkout, but the
behavior-bearing ED-A6 surface is not an isolated implementation lane. The
identity and recursive projection additions are staged or dirty without a live
owner, while their consumers are independently dirty. This session therefore
made no source edits and does not claim an ED-A6 fix or acceptance.

The Astra contract requires each leaf to retain node identity, tab/content
selection, focus, toolbar/chrome, render viewport, and input routing, with the
same keyed frames consumed by paint/hit-test/input and exact nested save/reopen
of axes, ratios, order, assignments, and active tabs. The current plan still
records the remaining gaps as independent cameras and persistent per-leaf
camera/focus/session ownership, complete hit-index/content-patch consumers,
no-op generation checks, and real Windows split/reopen acceptance.

## Evidence and ownership boundary

- `zircon_editor/src/ui/workbench/layout/document_node.rs:39-55` and the
  staged `layout/document_node_id.rs`, `layout/document_leaf_layout.rs`, and
  `layout/workbench_layout/deserialize.rs` introduce node IDs and duplicate-ID
  normalization, but the files have no current live attribution. The staged
  additions are visible in the index and must be adopted/validated by a single
  layout-persistence owner before projection consumers depend on them.
- `zircon_editor/src/ui/workbench/layout/manager/normalize.rs:18-32,53-70`
  normalizes IDs and split ratios, but is dirty with stale archived attribution
  (`astra-editor-layout-20260905`) and no live lease.
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/document_leaves.rs:16-107`
  recursively derives leaf frames and active tabs, and the staged
  `scene_projection/document_leaves.rs:3-57` publishes keyed scene surfaces.
  These are unowned staged additions, not a validated product boundary.
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs:65-100`
  computes the leaf vector but still assigns the legacy scalar
  `document_pane` from `.first()`, preserving a flattened compatibility path.
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs:129-147`
  and `shell_content_selection.rs:46-52` still select one global active
  document tab. `scene_projection.rs:277-300` and `host_data.rs:309-315,549-556`
  consume the dirty leaf DTOs, so changing one consumer alone would leave
  paint, input, and retained-scene authorities inconsistent.
- Coordinator ownership evidence reports the projection paths as
  `attribution_missing`, while `pane_projection.rs` has stale archived
  attribution (`root-runtime-editor-optimize-20260901-r6`) and
  `live_lease_missing`. The related layout identity/persistence paths likewise
  have staged/dirty content and no executable owner. No live ED-A6 owner or
  lease was found.

## Why this is blocked

Claiming a projection file now would merge into foreign staged/dirty changes
and could silently discard the intended owner’s tree/persistence contract. A
one-file patch cannot safely close ED-A6 because the leaf DTOs, shell fallback,
scene conversion, retained paint, hit/input routing, and persistence identity
normalization must move together. The repository also has an active managed
Cargo blocker and maintenance hold; this handoff deliberately performs no
Cargo or native-window command.

## Dependency-ready owner order

1. **Layout/persistence owner (lowest boundary):** adopt and attribute the
   staged identity/persistence set before further edits:
   `zircon_editor/src/ui/workbench/layout/document_node.rs`,
   `layout/document_node_id.rs`, `layout/document_leaf_layout.rs`,
   `layout/workbench_layout.rs`, `layout/workbench_layout/deserialize.rs`,
   `layout/manager/normalize.rs`,
   `ui/workbench/snapshot/workbench/document_workspace_snapshot.rs`,
   `snapshot/workbench/resolve_document_workspace.rs`,
   `model/document_tabs/collect.rs`, and the exact nested persistence tests.
   Validate unique IDs, legal ratios, legacy migration, and non-0.5 nested
   round-trip before handing off.
2. **Projection owner:** claim the full host-window set as one scope:
   `document_leaves.rs`, `shell_presentation.rs`, `pane_projection.rs`,
   `shell_content_selection.rs`, `scene_projection.rs`,
   `scene_projection/document_leaves.rs`, `scene_projection/geometry.rs`,
   `host_data.rs`, and `mod.rs`. Replace the scalar fallback with leaf-keyed
   pane/header/content frames while preserving the single-leaf default.
3. **Interaction/session owner:** route focus, toolbar, viewport/camera,
   pointer capture, and keyboard commands by `DocumentNodeId`; add focused
   tests for two leaves with independent active tabs and no-op isolation.
4. **Managed validation owner:** run the declared focused Rust batch, then
   real Windows split/resize/save/reopen and DPI evidence. Keep the plan
   `in_progress` until those gates pass.

## Status

`blocked_owner_scope` is intentional. This record is the only file written by
this session; no ED-A5 file and no ED-A6 source file was modified. The next
owner must establish attribution and exact leases for the complete dependency
slice before implementation.
