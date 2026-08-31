# Runtime pseudo-style delta publication review

Date: 2026-08-28

Status: current-source architecture review and deterministic pressure model
complete; production cutover and product timing remain pending.

## Outcome

The host invalidation scheduler is not currently promoting a pure pointer-hover
request into a full host recompute. `POINTER_HOVER` and `PAINT_ONLY` can complete on
the paint-only path, while pending workbench template rows use the scoped
`WORKBENCH_PROJECTION` patch. This part of the architecture should be retained.

The expensive work occurs before that host patch. A button hover mutates a runtime
pseudo-state property, then `UiV2RuntimeStyleIndex` reconstructs complete per-node
attribute/style/token maps. If a selector allows an ancestor pseudo-state to affect
descendants, it traverses the entire descendant subtree and repeats that work for
every styled node, even when only a small indexed subset can match.

## Current-source evidence

- `property_transaction.rs:181-200` routes an accepted pseudo-state change through
  `mark_component_state_render_dirty` or `apply_runtime_state_style_subtree`.
- `state_invalidation.rs:146-165` detects ancestor-affecting selectors and selects
  either the whole subtree or the changed node.
- `style.rs:229-245` only records a set of ancestor selector segments. It can answer
  whether descendants might change, not which descendants/rules are affected.
- `style.rs:286-352` performs a depth-first visit over the whole selected subtree.
- `style.rs:390-400` uses the terminal rule index, which reduces rule candidates but
  does not reduce subtree node visits.
- `style.rs:403-429` clones `base_attributes`, resolved self values,
  `base_style_overrides`, and `base_style_tokens` while resolving each visited node.
- `EditorWorkbenchTemplateSurface::refresh_after_state_change` and
  `apply_workbench_projection_presentation` subsequently patch changed rows. Those
  later incremental stages cannot recover CPU or allocations already spent in
  pseudo-style evaluation.

All production files above are shared dirty paths. This slice does not edit them.

## Unreal reference

Unreal's checked-in `SlateInvalidationRoot.cpp` is relevant for its ownership
boundary, not its C++ implementation details:

- lines 179-187 preserve typed pre/post invalidation reasons including `Paint`;
- `FSlateInvalidationRoot::InvalidateWidget` inserts the affected widget proxy into
  retained update structures;
- `ProcessAttributeUpdate` iterates registered attribute updates;
- `PaintFastPath` consumes the final retained update list and cached element data.

The transferable contract is that a property change publishes typed affected work.
It does not cause every widget below a possible selector ancestor to reconstruct
its complete property maps before the invalidation root can apply a local update.

## Target architecture

Style publication must compile immutable pseudo-state dependency products:

```text
PublishedPseudoStyleIndex
  self_state[(node, state)] -> [CompiledPropertyDelta]
  ancestor_state[(node, state)] -> [AffectedTerminal]

AffectedTerminal
  terminal_node
  selector/rule candidates
  compiled property deltas
  invalidation metadata (paint/layout/input/hit)
```

For a self-state change, evaluate only the changed node's compiled candidates. For
an ancestor-state change, visit the published affected terminals, not every node in
the subtree. A dense selector that genuinely changes all descendants remains
`O(D)` for `D` affected nodes; the target must not claim constant complexity.

Resolved node state should use an immutable baseline plus a compact runtime overlay
or structural-sharing map. Applying three hover declarations patches three
properties; it must not clone all base attributes, overrides, and token-source
entries. The delta carries dirty-domain metadata so layout/render/hit invalidation
is derived from properties that actually changed.

Topology, selector, theme, token, and stylesheet generations rebuild or patch the
dependency index before frame publication. Input callbacks only consume a published
generation. A missing/stale dependency entry must produce a typed fallback reason;
the fallback is counted and cannot silently become the normal hover path.

## Deterministic pressure model

`tools/runtime_ui_pseudo_style_delta_pressure.py` counts node visits, rule checks,
and map-entry copies. It is not CPU or latency timing. The fixture uses 1,000 state
changes, 24 baseline attributes, eight style overrides, eight token entries, four
candidate-rule checks, and three changed properties.

| Scenario | Current units | Target units | Reduction |
| --- | ---: | ---: | ---: |
| Self hover | 51,000 | 4,000 | 12.75x |
| 10,000-node ancestor subtree, 64 affected | 510,000,000 | 256,000 | 1,992.19x |
| 10,000-node ancestor subtree, all affected | 510,000,000 | 40,000,000 | 12.75x |

The sparse ancestor case avoids 10,000,000 subtree node visits and 460,000,000
full-map entry copies across the sample. The dense case deliberately stays linear
in the 10,000,000 real affected-node visits; its gain comes from applying three
property deltas instead of reconstructing 46 map entries per node.

Artifact:
`E:\zircon-profiles\runtime-ui-pseudo-style-delta-pressure-20260828.json`

## Implementation order

1. Add profiling counters for self/ancestor state events, subtree nodes visited,
   affected terminals, rule checks, full-map materializations, property deltas,
   dependency-index rebuild/patch, and typed fallback reasons.
2. Extend style compilation with stable rule IDs and explicit pseudo-state
   dependency metadata. Keep the existing terminal rule index as one input.
3. Build the self and ancestor dependency products at stylesheet/surface
   publication. Patch them with topology generation or rebuild on structural change.
4. Introduce immutable baselines plus runtime property overlays. Compare computed
   values before publishing a delta and preserve token-source semantics.
5. Route dirty flags, host projection rows, render command patches, damage, and
   frame publication from the exact changed property set.
6. Delete the event-path full-map reconstruction and subtree traversal fallback in
   the same hard cut after parity tests pass.

## Acceptance

- 1,000 stable same-target moves perform no pseudo-state mutation, style work, host
  projection patch, command patch, damage, or present.
- 1,000 cross-target hover changes materialize zero complete attribute/override/
  token maps and patch only old/new self-state targets when no ancestor dependency
  exists.
- A 10,000-node subtree with 64 matching descendants visits 64 affected terminals,
  not 10,000 nodes, per ancestor-state transition.
- A selector that truly affects all 10,000 descendants preserves pixel/layout/hit
  parity and reports 10,000 affected terminals without full-map materialization.
- Theme/token/stylesheet/topology changes invalidate dependency products with exact
  generations; stale products never dispatch a route or publish a frame.
- Managed tests cover selector specificity/order, multiple matching rules, token
  restoration, custom pseudo states, descendant combinators, removal/reparenting,
  layout-affecting declarations, and typed fallback.
- Product profiling records p50/p95/p99 input-to-published-style,
  input-to-hover-paint, allocator deltas, CPU, RSS, affected node counts, command
  patches, damage pixels, and fallback reasons for self, sparse ancestor, and dense
  ancestor fixtures.

## Validation status

Static source guards and the Python pressure model run without Cargo. Managed Rust
and Editor product-path validation remain pending official lane authorization and
current-source closure. No dynamic performance claim is made here.
