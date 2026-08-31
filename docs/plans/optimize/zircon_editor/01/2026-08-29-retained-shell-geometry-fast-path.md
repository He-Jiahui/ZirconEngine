---
source_binding:
  head: 189f72219eaf16a6d0db880b53f3f68b4f5ee15a
status: current_source_static_incremental_index_implementation
product_timing: false
---

# Retained shell geometry fast path

## Decision

Window metrics invalidation now has two publication paths. A stable shell keeps
one retained `ShellPresentation` and its lifecycle pane payloads. Resize frames
recompute only shell geometry and template layout frames, convert chrome and
floating-window geometry, and publish a new host geometry presentation. The
geometry builder clones the last domain scene and refreshes only
chrome/dock/floating frames; it does not call pane projection builders.
Existing pane values are cloned shallowly so their `UiSurfaceFrame`, model, text,
and image handles remain shared.

Any structural, pane-content, workbench-node, or scoped asset change invalidates
the retained shell handle. A missing handle or payload falls back to the current
full presentation path; no partial geometry frame is published in that case.

## Ownership contract

`HostPresentationGeneration` now separates semantic structure identity from the
composed geometry view. `structure_generation` changes only when semantic host
structure changes; `geometry_generation` advances for every geometry publication.
Geometry publication now patches the workbench hit and paint indexes before it
publishes the new generation. The host state changes only after both products are
available, so input and paint observe one coordinate generation. The previous
generation remains immutable for any route or paint operation already holding it.

Floating windows are projected by stable `window_id`: their frame, header, tabs,
and edge metadata are refreshed while the current active pane is retained. This
keeps popup/floating content out of the resize hot path without preserving stale
outer coordinates.

Visual resources already have a shared retention boundary: SVG trees are cached
by canonical path and source stamp, raster pixels by asset/target/tint key, and
the GPU presenter checks resource residency before staging uploads. Geometry
publication therefore retains image handles and does not introduce another SVG
parse or image-upload path.

The host-contract merge assembles the next presentation by field group rather
than cloning the previous full scene before replacement. Only the retained
interaction/resource state and pane payload handles are copied from the prior
generation.

The geometry-only responsibilities are isolated in
`scene_projection/geometry.rs` and `apply_presentation/geometry.rs`; the parent
modules remain orchestration boundaries rather than accumulating another large
rendering path.

## Current-source pressure evidence (2026-08-30)

The structural audit at
`E:\zircon-profiles\ui-structural-audit-20260830-r1\ui-structural-hotspots.json`
is bound to the current source snapshot (`189f72219eaf16a6d0db880b53f3f68b4f5ee15a`).
It covers 4,858 files and 520,738 lines, with 3,021 dirty paths. The heuristic
counts are 5,963 clone calls, 3,022 vector materializations, 193 sort calls,
8,136 string allocations, 3,248 traversal signals, and 2,167 dirty hotspots.
These counts prioritize inspection; they are not CPU, memory, or GPU timing.

The deterministic pressure models are stored outside the repository and are
also non-timing evidence:

- `E:\zircon-profiles\editor-pane-surface-retention-20260830-r1.json` models
  64 panes with 2,048 nodes each. Retention reduces surface builds from
  128,064 to 1,064 and stage visits from 1,049,100,288 to 8,716,288
  (120.36x fewer visits), avoiding 64,000 stable surface builds.
- `E:\zircon-profiles\editor-window-resize-reflow-20260830-r1.json` models
  affected-node and damage-region publication for resize; it does not claim
  product timing.
- `E:\zircon-profiles\editor-incremental-hit-index-20260830-r1.json` models the
  new row/cell algorithm. With 10,000 semantic nodes and 4,096 retained cells,
  the sparse 64-row/192-cell case falls from 40,000 to 2,560 modeled work units
  (15.62x), while a broad 1,200-row/640-cell resize falls to 9,520 (4.20x).
  Candidate-bucket constants and all real timing/memory/GPU costs are excluded.
- `E:\zircon-profiles\runtime-ui-surface-input-publication-20260830-r1.json`
  is the historical full-publication target model for 64 surfaces and 300,000
  input events. Current source has since hard-cut raw `MouseMotion` before
  Surface fanout, routes Keyboard/Text/IME and Navigation/Analog to retained
  owners, and uses a retained cell-to-Surface pointer directory with affine
  lookup against the last published viewport. The modeled
  pointer/focus/navigation/raw set is now 400,000 dispatches with zero
  event-path rebuild probes. Typed admission now reserves reverse fanout for the
  cold unpublished state and rejects invalid pointer/viewport input in O(1).
  The current source-bound authority is the 20260831-r10 artifact.
- `E:\zircon-profiles\runtime-ui-render-dependency-product-20260830-r1.json`
  models stable image/text dependency products. Image segment visits fall from
  262,144 to 288 (910.22x); a one-segment text delta falls from 65,536 to
  1,024 dependency entries (64x). The model has no product timing claim.

## Reference review and selected algorithm

Primary comparison: Unreal `FHittestGrid` keeps stable widget metadata and a
cell-to-widget index. `AddWidget` removes a widget only from its old cell range
when its paint-space bounding cells change, then inserts it into the new range.
`UpdateFastPathRenderTransform` applies a cached transform delta and updates that
widget instead of rebuilding the complete grid. Cell candidates retain an
explicit paint ordering key before front-to-back hit evaluation. Relevant source:
`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h`,
`.../Private/Input/HittestGrid.cpp`, and `.../Private/Widgets/SWidget.cpp`.

Secondary comparison: Fyrox invalidates cached measure/arrange/visual state up
the ancestor chain and skips measure or arrange when the previous constraint is
still valid. Its unrestricted hit test recursively walks the widget hierarchy and
tests retained draw commands, so it is useful evidence for layout caching but not
the target hit-test complexity. Relevant source:
`dev/Fyrox/fyrox-ui/src/lib.rs` (`handle_layout_events`, `arrange_node`,
`measure_node`, and `pick_node`).

Zircon cannot directly copy Unreal's mutable game-thread grid because
`HostPresentationGeneration` values remain live across input and paint. The
selected design therefore keeps the same affected-cell algorithm but stores cell
buckets in an immutable AVL map. Updating `D` cells path-copies `O(D log C)` map
nodes, deletes vacated cells, shares every untouched branch, and leaves the old
generation queryable. Bucket rows are sorted by `(z_index, row)`; this also fixes
the prior main-hit behavior where reverse row order could beat a higher z node.

The workbench template surface publishes an exact sorted pending row set only
when it contains geometry changes and no semantic changes. The host conversion
clones only these retained rows and replaces only frame, clip, and z data. It does
not reconstruct text, bindings, styles, accessibility strings, or image handles.
Mount-origin or presentation-scale changes are typed fallbacks because those
changes also affect popup anchors and visual metrics. Geometry and hit-index
products are committed atomically; a topology, identity, input-membership,
paint-model-cardinality, or ordering mismatch returns to the complete path.

For stable scale and topology, the workbench portion is bounded by
`O(H_aff + D log C)` time and `O(H_aff + D log C)` new storage, where `H_aff` is
the exact geometry-changed host row set, `D` is its old/new cell union, and `C` is
the retained cell count. It is independent of the unchanged semantic payload
size. Geometry-bearing chrome models outside the workbench model are still
rebuilt per affected model; retained pane models remain shared.

### Known residuals

- Window resize can legitimately affect more than 1,000 workbench frames. This
  slice removes semantic conversion and full-index reconstruction, but layout
  work remains proportional to the runtime's actual geometry dirty set.
- Geometry chrome projection still rebuilds small menu/page/status/dock/floating
  frame models. A later slice may give these models their own sparse row deltas if
  product profiling shows material cost.
- SVG parse, raster pixel, and GPU residency cache ownership was reviewed but not
  edited in this slice. The existing render cache file is an external worktree
  dependency and is excluded from this candidate.

## Static acceptance

- Geometry publication regression checks semantic-generation stability, geometry
  generation advancement, hit-index advancement, changed frames, and pointer
  identity for all four dock body surface frames.
- Lower regressions cover a geometry-only row patch that preserves semantic fields
  and unchanged row identity, z-correct overlapping hit order, old/new coordinate
  behavior, immutable old-index behavior, persistent bucket deletion, and bounded
  tree height under repeated updates.
- Source contracts expose geometry row indices only after the initial full product
  is committed, reject semantic pending work, and avoid the full workbench node
  conversion in `apply_presentation/geometry.rs`.
- Scene conversion regression checks geometry-only conversion preserves all four
  pane `UiSurfaceFrame` handles.
- Source contracts assert the WindowMetrics geometry path runs before pane payload
  collection, selects the retained domain scene, does not call the full scene
  builder or runtime pane conversion, and does not increment the pane projection
  counter. A scene-level guard rejects semantic pane projection calls from the
  geometry builder.
- Scoped `rustfmt --edition 2021 --config skip_children=true` and `git diff --check`
  pass for the owned implementation paths.

Managed Cargo validation, Editor product timing, CPU/RSS/GPU counters, and
visual/input parity remain pending the authorized validation lane. No Cargo run
is claimed by this record.

## Follow-up gates

1. Run lower host-contract tests through the official managed lane and verify the
   generation and handle-identity regressions.
2. Capture WindowMetrics resize with 1/4/16/64 surfaces and compare pane
   projection count, input-to-present latency, CPU, private working set, and GPU
   upload counters against the full-path baseline.
3. Add a product regression for floating-window geometry and a stale-coordinate
   hit check before accepting the fast path as a milestone.
4. Record the new counters
   `ui.window_resize.hit_index_geometry_patch_{count,row_count,cell_count}` and
   `ui.window_resize.paint_index_geometry_rebuild_model_count`; require zero
   full-index builds and zero geometry-patch fallbacks during a stable-scale resize
   pressure run before accepting this slice.
