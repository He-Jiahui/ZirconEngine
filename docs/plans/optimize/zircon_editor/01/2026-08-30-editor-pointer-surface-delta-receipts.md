---
related_code:
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_sync.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/rebuild_surface.rs
status: editor_private_receipt_candidate_static_validated_managed_validation_pending
product_timing: false
---

# Editor pointer-surface delta receipts

## Review result

The Runtime authored-geometry transaction removes full arranged, hit, render,
projected-hit, and navigation rebuilds after Editor supplies exact changed node
IDs. Two Editor bridges still rediscover those IDs by walking data they own.

### Viewport Toolbar

`sync_surface_frame` correctly skips a frame whose authoritative hit-grid `Arc`
is unchanged. When that grid changes, however, it performs these passes:

1. scan every hit entry and project matching toolbar controls;
2. compare the complete projected control vector;
3. walk every retained root, surface, control, route, and node path to prove the
   private `UiSurface` topology still matches;
4. compare every retained frame again to recover changed node IDs.

The source hit-entry scan is currently unavoidable because `UiSurfaceFrame`
does not yet publish exact hit-entry changes. Passes 2-4 are not. The bridge is
the sole mutator of its private Surface, dispatcher, and route map, so a typed
classification made while projecting controls is a stronger authority than a
defensive full tree audit on every accepted frame.

### Viewport Overlay

The pointer hot path is already O(1) when the interaction-extract `Arc` and world
generation are unchanged: `sync_scene` returns before candidate projection.
For a genuinely changed camera, viewport, extract, or world, projecting every
candidate is semantically required because every screen-space frame may change.
The current changed-product path adds multiple avoidable passes after that work:

1. project all precision candidates;
2. walk the complete retained Surface to validate topology;
3. scan the shared candidate map to compare route identity;
4. scan all candidates to patch geometry/order;
5. compare every candidate-map key;
6. scan all candidates again to replace map values.

The optimization target is therefore not a false O(1) camera update. It is one
staged classification pass plus one required apply pass, with no retained-tree
walk and no preflight key scan.

## Algorithm

Introduce Editor-private receipts with three outcomes:

```rust
enum RetainedSurfaceDelta<T> {
    NoChange,
    Geometry(T),
    Topology,
}
```

The concrete payloads remain domain-specific rather than exposing a generic
public abstraction.

### Toolbar receipt

1. While projecting controls from a changed hit grid, compare each projected
   control with the same indexed retained control.
2. A count, surface-key order, or action-key change returns `Topology`.
3. Stable action keys with changed frames return `Geometry` containing exact
   surface/control node IDs. Surface-origin changes add the root, surface, and
   affected controls without rescanning the tree.
4. `NoChange` updates only the applied source-frame cursor.
5. Only `Topology` reconstructs `UiSurface`, dispatcher, routes, and nodes.

The retained bridge state, not repeated node-path string parsing, is the
topology authority. A debug-only invariant test may still inspect the tree; it
must not run on the product input path.

### Overlay receipt

1. Candidate projection produces the staged entry vector and classifies stable
   node identity, route identity, frame changes, and z-order changes as entries
   are produced.
2. A candidate node-ID/count change is `Topology`. The old Surface is not
   mutated until classification completes.
3. Stable topology applies candidate frames in one pass. Exact changed IDs feed
   the Runtime authored-geometry transaction. A z-order change retains Surface
   identity but selects the existing full authored-order fallback.
4. Route changes release pointer capture before shared candidate values are
   published.
5. Stable topology updates the shared map by direct keyed replacement. A missing
   key is an invariant failure that rebuilds the map once; no complete key
   comparison runs before every update.

## Complexity contract

| path | current additional work | target additional work |
| --- | --- | --- |
| unchanged pointer extract | O(1) | O(1) |
| Toolbar changed source frame | source O(H) + bridge O(C) + tree O(C) | source O(H) + exact O(K) apply |
| Overlay camera/viewport change | required O(N) projection + about four O(N) validation/publication passes | required O(N) projection + one O(N) apply |
| one exact Toolbar frame change after projection | O(C) bridge comparison/patch | O(K) patch |
| topology change | O(N) validation then O(N) rebuild | O(N) staged classification then O(N) rebuild |

`H` is source hit entries, `C` Toolbar controls, `N` Overlay candidates, and `K`
is the exact changed retained-node set. This milestone does not claim that the
Toolbar source scan is solved. That requires a later Runtime-owned frame delta
receipt and must wait for the current interface module graph to become a
copy-complete validation input.

Root-size changes also remain an explicit `RootSizeChanged` Runtime fallback.
Removing that fallback requires a separate transaction which updates Surface
bounds and regrids hit cells coherently; merely passing the root node through a
geometry receipt would leave the hit-grid bounds stale. This receipt milestone
therefore removes redundant Editor passes but does not claim O(K) window resize.

## Regression order

1. Toolbar: stable action identity plus one moved control selects geometry and
   publishes exactly that control node.
2. Toolbar: unchanged projected controls select `NoChange` and do not invoke a
   retained-tree topology walk.
3. Toolbar: action/count/surface-key changes select `Topology` before mutation.
4. Overlay: stable candidate identity with one moved frame retains Surface and
   candidate-map allocation while publishing the exact node.
5. Overlay: changed route releases capture; changed z-order selects the authored
   order fallback; changed node identity selects topology rebuild.
6. Source guards reject retained-tree topology scans and complete candidate-map
   key preflight scans from the changed-product path.

## Acceptance

- No product method named `retained_surface_topology_matches` remains in either
  pointer bridge.
- Stable Toolbar geometry never parses node paths or validates route bindings.
- Overlay retains the O(1) unchanged-extract early return.
- Overlay changed-product work records projected, classified, changed, applied,
  topology-fallback, ordering-fallback, and route-change counts.
- Runtime local publication receives only exact changed node IDs.
- Focused Rust regressions, scoped formatting, source guards, and managed Editor
  validation are required before this milestone is accepted.
- Product CPU/input-to-present timing remains a later measured acceptance gate;
  this document contains complexity bounds, not timing claims.

## Current static evidence

- Editor source-contract tests: 3/3 passed.
- Runtime pressure and source-contract tests: 9/9 passed.
- `python -m py_compile` passed for the new Editor contract test.
- Scoped `rustfmt --edition 2021 --config skip_children=true --check` passed for
  the seven touched Editor modules.
- Scoped `git diff --check` passed; Git reported only its existing LF/CRLF
  conversion warnings.
- Managed Cargo validation has intentionally not been submitted from this
  candidate because the shared RuntimeInterface overlay is not yet
  copy-complete. This is a validation boundary, not a passing-build claim.
