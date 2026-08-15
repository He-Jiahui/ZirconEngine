# UI Asset root dirty propagation performance audit

Status: `static_audit_complete / current_source_profile_pending / production_change_deferred`

## Trigger

The non-Cargo UI Asset performance contract suite reported two current-source
failures:

- `test_preview_resize_borrows_root_ids`
- `test_node_projection_borrows_roots_and_moves_owned_extract_text`

Both failures detect `surface.tree.roots.clone()` before marking roots dirty.
The same suite initially found a stale `presentation_state.rs` test path; that
contract now follows the hard-cut `session/presentation/*.rs` modules and passes.
The current suite result is 28 passed and 2 failed. The two failures remain RED
until a current-source profile justifies and verifies a production change.

## Audited data flow

| Entry | Trigger cadence | Root work | Downstream work |
|---|---|---|---|
| `ui_asset_editor_node_projection(size)` | Workbench projection call; resize branch only when cached size changes | clones every root id, marks layout/hit-test/render | `UiSurface::rebuild_dirty`, then full editor-node projection |
| `UiAssetPreviewHost::rebuild_with_size` | legacy UI Asset preview preset or size change; equal sizes return early | clones every root id, marks layout/hit-test/render | `UiSurface::rebuild_dirty` |
| V2 preview rebuild | preview preset change | builds a new V2 preview surface | does not use either failing clone path |

The lower shared boundary is `zircon_runtime_interface::ui::tree::UiTree`.
It exposes `roots: Vec<UiNodeId>` and `node_mut`, but no operation that safely
iterates root ids while mutating the disjoint node map. Both editor owners copy
the root-id vector to satisfy that borrowing boundary. Equivalent root copies
also exist in Runtime UI layout, input, hot reload, and surface rebuild code, so
an editor-only helper would leave the shared support gap unresolved.

## Complexity and allocation evidence

- The copied payload is `root_count * size_of::<UiNodeId>()`, plus one vector
  allocation when capacity is non-zero. This is source-derived, not measured.
- Root marking is `O(root_count * log(node_count))` because `UiTreeNodes` uses a
  `BTreeMap`; `rebuild_dirty` and render extraction are separate downstream
  costs and may dominate. No relative cost claim is made without profiling.
- The resize fast path for an unchanged `UiSize` performs no clone or rebuild.
- The current tests only ban the allocation pattern; they do not report elapsed
  time, allocation bytes, root count, or frame impact.

## Required profile before optimization

Use a coordinator-built current-source Windows editor binary. Capture WPR CPU
sampling plus heap allocation data for these scenarios:

1. Stable UI Asset Editor for 300 frames without a size change.
2. Repeated preview preset changes for a one-root document.
3. Repeated node-projection resizes at 640, 900, and 1260 logical pixels.
4. Synthetic multi-root documents at 1, 64, and 1,000 roots, with node counts
   held constant between before/after runs.

Record per scenario:

- root count and resize/rebuild invocation count;
- allocations and bytes attributed to root-id vector cloning;
- p50/p95 time for root dirty propagation and `rebuild_dirty` separately;
- total editor-frame p50/p95 and any frame above the MVP budget;
- call stacks proving the measured allocation belongs to current source.

`wpr.exe` is available on this host, but no managed current-source editor binary
or terminal build receipt is available in this session. No trace was captured,
and no timing or allocation number is fabricated.

## Candidate shared repair after profile

If the profile confirms actionable allocation or the architecture owner retains
the no-clone contract, add one narrow `UiTree` root-mutation API that borrows
`roots` immutably and `nodes` mutably as disjoint fields while preserving
`UiTreeNodes` mutation tracking. Migrate the two editor callers through that
shared API first, then assess equivalent Runtime call sites independently.

Do not add an editor-only cached root list, compatibility facade, unsafe alias,
or a second tree truth. Re-run the lowest `UiTree` mutation tests, the two
focused UI Asset contracts, the 30-test performance group, managed editor tests,
and the same WPR scenarios before claiming improvement.

## Current decision

- Completed: module/data-flow audit, lower-layer identification, stale test-path
  hard cut, focused 3/3 palette contract, and 28/30 performance contract result.
- Deferred: production root iteration change, managed Cargo validation, WPR
  capture, before/after comparison, and acceptance.
