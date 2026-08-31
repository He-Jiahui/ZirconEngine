---
related_code:
  - docs/plans/optimize/zircon_editor/01/2026-08-30-editor-pointer-surface-delta-receipts.md
  - zircon_runtime_interface/src/ui/surface/persistent_sequence.rs
  - zircon_runtime_interface/src/ui/surface/arranged.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs
  - zircon_runtime/src/ui/surface/arranged.rs
  - zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/render/cache.rs
  - zircon_runtime/src/ui/surface/surface/frame_publication.rs
  - tools/runtime_ui_authored_geometry_delta_pressure.py
  - tools/tests/test_runtime_ui_authored_geometry_delta_pressure.py
status: persistent_authority_resize_envelope_and_editor_receipt_static_validated_managed_validation_pending
product_timing: false
evidence_artifact: E:/zircon-profiles/runtime-ui-authored-geometry-delta-pressure-20260830-r7.json
evidence_sha256: 59754BB231C9285A9195BF65D08B2455B8134A40EB12F4C797C0B2A32866DB24
source_revision: cc5cadbd597c3707954ebd6109fad0fd5643a152
source_set_sha256: 93BE786936FE5F9DD26AB787084B14BD064D8DE95D2E01D7E99AA75CC447AF15
---

# Runtime authored-geometry delta publication

## Problem

Editor pointer Surfaces receive frames that have already been computed by shell,
viewport, tab, menu, and projection owners. `UiSurface::rebuild_authored_frames`
publishes those frames correctly, but it calls the full `rebuild()` path. A
one-control move therefore rebuilds the complete arranged tree, hit grid, render
extract, projected hit authority, navigation index, and published frame domains.

`rebuild_dirty` cannot be reused by merely marking layout dirty. Its geometry
path is owned by Runtime layout constraints and may recompute an externally
authored frame. Marking only hit/render dirty detects the changed frame during
input patch validation and conservatively falls back to the same full rebuild.
The first missing abstraction is an exact-node transaction for already-authored
geometry.

The review also found a second independent O(N) boundary after that transaction.
`UiSurfaceFramePublication::surface_frame` clones the complete
`UiArrangedTree` whenever layout changes and clones the complete authoritative
`UiHitTestGrid` whenever hit geometry changes. The render snapshot already uses
a persistent segmented directory, but arranged nodes, hit entries, and hit-cell
membership were flat `Vec` products. Therefore the first Runtime transaction
changed mutable-domain visits to O(K), but frame publication still performed
O(N + E + C) record copies. The e4 candidate removes that residual by making
the ordered arranged/hit authorities persistent themselves; `UiSurfaceFrame`
now shares their roots in O(1).

## Reference contract

Unreal Slate keeps the same authority at paint/update time instead of deriving
geometry again during input. `WidgetProxy.cpp::UpdateFastPathRenderTransform`
updates the affected widget and descendants. `HittestGrid.cpp::AddWidget` reads
`GetPaintSpaceGeometry()` and `GetRenderBoundingRect()`, preserves the widget
index when possible, and changes cell membership only when bounds change.

Zircon should copy the ownership rule, not the C++ object model: one changed-node
set must drive arranged, hit, render, projected hit, navigation, and immutable
frame publication before the event hot path resumes.

## Phase 1: Runtime mutable-authority transaction

Add one Runtime-owned operation with a typed result equivalent to:

```rust
pub enum UiAuthoredGeometryPublication {
    Unchanged,
    Local(UiSurfaceRebuildReport),
    FullFallback {
        reason: UiAuthoredGeometryFallbackReason,
        report: UiSurfaceRebuildReport,
    },
}
```

The operation accepts the current root size, the exact changed-node set, and the
caller's observed topology generation. It owns the following transaction:

1. Reject an empty or stale request before mutation. Validate tree identity,
   topology generation, and node/index presence. Root-size changes are part of
   the authored geometry receipt and no longer force a layout fallback.
2. Expand only clip-owning changed nodes to affected descendants. Do not scan
   unrelated branches.
3. Patch arranged frame, clip frame, and slot geometry through the existing
   staged replacement path.
4. Patch hit entries and old/new cell membership from the arranged result. The
   hit grid keeps a 64px-aligned, geometric-growth capacity envelope; ordinary
   window resize stays O(K), and only crossing an envelope boundary performs a
   hit-cell regrid. The regrid is local to hit authority and does not rebuild
   layout, text, or render products.
5. Patch geometry-only render commands and damage ranges. Unsupported text,
   size-dependent, or missing command products trigger a typed full fallback.
6. Synchronize popup-projected hit geometry and navigation geometry from the
   same affected set.
7. Mark exact render ranges/domain generations and eagerly publish the new
   `UiSurfaceFrame` before returning. Input must remain a read-only lookup.

The resize regression changes both the root and a child beyond the original
hit-grid bounds, verifies the old frame remains immutable, and proves the new
frame hit path equals the instance path. A second lower-layer hit-index case
proves that growth inside the capacity envelope patches cells, while crossing a
geometric boundary requests only a regrid.

This phase removes the redundant mutable arranged/hit/render walks. Persistent
arranged and hit roots now make publication O(1); producer mutation remains
bounded by touched leaves and hit-cell membership.

No derived domain may be published early. Existing arranged, hit, and render
patchers stage validation before applying updates; any later failure must run
the full authored-frame fallback before frame publication so consumers never
observe mixed generations.

Typed fallback reasons must distinguish at least:

- missing node or arranged index;
- topology generation changed;
- clip descendant expansion failed;
- hit-grid capacity regrid (handled locally inside the transaction; it is not a
  whole-surface fallback);
- render command is not geometry-patchable;
- projected-hit or navigation patch failed.

## Phase 2: Persistent published arranged/hit domains

Bring immutable arranged and hit publication to the same structural-sharing
contract already used by `UiRenderFrameExtract`:

1. Keep full rebuilds efficient by collecting and sorting in temporary flat
   vectors, then convert once to persistent authority products.
2. Store arranged nodes and hit entries in fixed-cardinality persistent
   segments. A local mutation copies only touched leaf segments and directory
   paths; publication shares the resulting roots.
3. Publish hit cells with independently shared membership so moving one entry
   clones only old/new affected cells, not every cell and entry reference.
4. Store immutable topology products (`roots`, draw order, canvas layers) in the
   same persistent representation and keep route nodes shared by `Arc` while
   topology/input ancestry is stable.
5. Preserve old frame immutability when a consumer retains it. In-place reuse is
   allowed only when ownership proves no older observer exists; otherwise COW
   must copy the bounded persistent paths.
6. Keep public frame lookup indexed. A persistent container must provide O(1)
   or O(log segments) indexed access and allocation-free iteration; it may not
   turn hit testing into a linear search.

A surface-owned `Arc::make_mut` wrapper around flat `Vec` is only a conditional
fast path, not the architectural end state: a renderer retaining the previous
frame would force a full COW clone. Acceptance therefore requires persistent
segment counters, including the retained-previous-frame case.

### Published-domain data structure

The implementation review rejected a separate mutable-tree/frame-tree pair.
That pair would require duplicate public types, duplicate hit-query plumbing,
and an explicit ID-to-frame-index publication receipt even though Runtime
already owns stable indices. The accepted candidate instead uses one authority:

- `UiPersistentSequence<T>` uses 64-item leaves, directory fanout 32, bounded
  depth, O(log32 N) indexed access, allocation-free O(N) iteration, and flat
  serde compatibility.
- `UiArrangedTree` uses it for roots, nodes, draw order, and canvas layers.
  Full rebuild still sorts a temporary `Vec`; only the completed product is
  converted into the persistent directory.
- `UiHitTestGrid` uses it for entries and cells, while each cell membership list
  is an independent `Arc<Vec<usize>>`. Existing entry/cell maps remain the
  identity authority; local `get_mut_with_stats` calls COW only the old/new
  touched leaf paths and only the touched cells' membership lists.
- `UiSurfaceFramePublication` clones the outer arranged/grid values, which now
  shares all persistent roots and `route_nodes`; it performs zero arranged-node,
  hit-entry, or hit-cell-entry record copies.
- COW statistics distinguish item, segment, and directory-node copies for base
  arranged, base hit, and popup-projected hit mutations.

The hit query continues to index one cell, walk its ordered entry indices, and
resolve entries from the persistent sequence. There is no delta replay,
node-map lookup, arranged-tree scan, render-command scan, or snapshot repair on
the event path.

### Rejected alternatives

- Flat `Vec` behind `Arc::make_mut`: fast only when no old frame survives and
  therefore fails the renderer-retained-frame acceptance case.
- `HashMap<UiNodeId, UiArrangedNode>` snapshots: avoids vector copies but adds
  hashing and fragmented iteration to paint/debug consumers, while hit cells
  still need stable integer entry identity.
- Unbounded base-plus-delta frame chains: cheap publication but makes hit lookup
  proportional to chain depth and creates periodic compaction spikes.
- Rebuilding cells from persistent entries: preserves entry locality but still
  scans all entries and defeats pointer/resize responsiveness.

## Phase 3: Editor typed delta

Runtime locality alone leaves the Viewport Toolbar bridge performing an O(N)
topology validation before every patch. Editor mutation entry points must
eventually return one of `NoChange`, `Geometry { node_ids }`, or `Topology`.
Existing control-index and action-route comparisons already know which case
occurred; that information should reach publication instead of being discarded
and rediscovered by walking the retained tree.

`Topology` remains the only path allowed to reconstruct `UiSurface`, dispatcher,
route map, nodes, and routes. `Geometry` preserves every identity and supplies
the exact node IDs to the Runtime transaction.

Current candidate state: the Viewport Overlay and Viewport Toolbar retained
bridges now collect exact changed frame IDs before mutation and call the Runtime
transaction. Overlay ordering changes are explicitly excluded and use the full
authored fallback. This removes full mutable-domain rebuilds after the bridges
have accepted stable topology. It does not yet remove their O(N) topology
preflight or full layout-to-bridge frame comparison; producer-owned `NoChange /
Geometry / Topology` receipts remain required for the Phase 3 end state.

The detailed producer-to-bridge review, staged algorithms, complexity contract,
and regression order are recorded in
`2026-08-30-editor-pointer-surface-delta-receipts.md`. Its first implementation
slice is intentionally Editor-private so it can remove redundant retained-tree
and candidate-map scans without extending the currently non-copy-complete
RuntimeInterface frame contract.

## Pressure bound

The deterministic default uses 529 nodes, 1,000 one-node frame patches, and 10
topology changes:

| mutable-domain work | current authored publication | Runtime exact patch |
| --- | ---: | ---: |
| full pipeline rebuilds | 1,010 | 10 |
| arranged node visits | 534,290 | 6,290 |
| hit node visits | 534,290 | 6,290 |
| render node visits | 534,290 | 6,290 |
| total avoided domain visits | 0 | 1,584,000 |

Those 1,584,000 avoided visits apply to arranged, hit, and render logical work.
The pre-e4 `UiSurfaceFrame` also cloned 534,290 arranged-node records and
534,290 hit-entry records. The persistent-authority candidate reduces both
publication counts to zero. With 64-item leaves, its conservative arranged and
hit-entry item-copy upper bound is 69,290 per domain: 5,290 items built for true
topology changes plus at most 64,000 items copied across 1,000 retained-frame
one-node mutations. Actual hit-cell member-reference copies remain distribution
dependent and must come from product counters rather than this model.

Logical arranged and hit updates are 6,290 in the same scenario, while physical
copies are bounded by touched persistent segments and old/new hit cells. After
Phase 3, Editor topology validation falls from 534,290 node visits
to 5,290 visits for true topology changes plus 1,000 exact geometry identity
checks. Clip expansion raises `K` to the affected subtree size; it does not
permit a silent global scan.

These are operation counts. They exclude actual CPU time, hit-cell cardinality,
render command count, allocation/RSS, GPU work, and input-to-present latency.

## Regression order

1. Runtime: move one authored-frame control; the old point must miss, the new
   point must hit, and arranged/hit/render visit counts must equal the affected
   set.
2. Runtime: move a clip owner and prove descendant clip/hit entries update from
   the same expansion set.
3. Runtime: prove popup frame-path and instance-path hit parity after a local
   authored geometry patch.
4. Runtime: prove navigation geometry and immutable `UiSurfaceFrame` domains are
   current before the first event/read consumer.
5. Frame interface: retain a sequence clone, patch one item, and prove exactly
   one leaf/directory path is copied while unrelated leaves remain shared.
6. Runtime: retain the previous surface frame, patch one node, and prove its old
   arranged geometry, hit entry, and cell membership remain unchanged while the
   new frame is authoritative.
7. Frame interface: drop the previous frame and prove the ownership fast path
   does not allocate or copy unrelated segments.
8. Runtime: cover every typed fallback and prove the fallback publishes a fully
   coherent frame, never a partially patched generation.
9. Editor: same action and stable topology must retain Surface, dispatcher,
   route, node, and hit identity while changing only exact frames.
10. Editor: action or node-order changes must select the topology fallback.

## Acceptance

- Stable one-node patches perform no full mutable arranged/hit/render scan and no
  full immutable arranged/hit snapshot clone.
- Local work is bounded by changed nodes, clip-affected descendants, changed
  render ranges, touched persistent segments, and old/new hit cells.
- Event hit testing scans neither tree nodes nor render commands.
- Full fallback count is zero in the stable-frame scenario and every fallback
  records a typed reason.
- Product runs report affected nodes, hit cells, render ranges, arranged/hit
  full-clone counts, persistent segment copies, domain generations, allocation
  bytes, and input-to-present p50/p95/p99.
- Managed Rust and product validation are required before implementation status
  can advance; no Cargo command was run for this design milestone.

## Static verification

- Focused authored-geometry pressure/source-contract tests: 9/9 passed.
- The v3 evidence manifest binds 12 current source files, including the
  persistent sequence/arranged/hit contracts, typed Runtime transaction, both
  Editor consumers, and zero-copy frame-publication counters.
- `python -m py_compile` for both new pressure tools and tests: passed.
- `rustfmt --edition 2021 --check --config skip_children=true` for the persistent
  interface, Runtime transaction, COW instrumentation, and focused regressions:
  passed.
- Scoped `git diff --check` for the Runtime candidate passed; Git emitted only
  the repository's existing LF/CRLF conversion warning.
- Managed focused Runtime ticket
  `00d3186393f8491db2ffca9bdafe375a` was submitted against manifest
  `5d13cb404958ae69e553544bc72dd7bfce903f1a3f1a8f067cf42371b8bc6f6d`;
  and reached terminal `snapshot_stale` without validating the current source.
  Subsequent Runtime, persistent-interface, and Editor review changed the
  candidate byte set. A rebound ticket was not submitted because the shared
  `zircon_runtime_interface` module surface still contains external uncommitted
  module-graph changes; an exact owned overlay is not yet copy-complete, while
  absorbing those foreign paths would violate ownership. Acceptance still
  requires one current-source persistent-interface/Runtime ticket followed by
  the focused Editor regressions.
