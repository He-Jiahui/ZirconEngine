# Runtime UI Retained Taffy Parent Product Review

Date: 2026-08-30

Status: architecture decision and measurement plan; production cutover not yet accepted

Source binding used for this review:

- revision: `f31fd06f69fdaedb70a0a56fe6d0268de1af83a6`
- worktree: dirty, with concurrent owner changes in the Runtime UI layout paths
- validation restriction: no raw Cargo; product timing remains gated on a source-bound managed binary

## Outcome

Zircon does not rebuild one global Taffy tree on every input event. It rebuilds a small Taffy tree for every visited auto-layout parent. The current bridge clears one reusable scratch tree, creates one Taffy leaf per visible direct child, creates the parent, solves the local tree, and reads every direct-child layout. For a set `P` of visited Taffy-owned parents, current topology construction work is:

```text
tree_builds = |P|
taffy_nodes_created = sum(parent in P, visible_children(parent) + 1)
layout_reads = sum(parent in P, visible_children(parent))
```

This is better than a global `O(all UI nodes)` rebuild, but it still discards Taffy's stable node identity and internal compute cache at every visited parent. A single changed child in a wide flex/grid/wrap parent recreates all sibling Taffy nodes. A changed descendant under nested auto-layout parents can repeat that cost at every propagated ancestor.

The ordered-child lookup is no longer part of this defect. `UiLayoutSlotIndex` owns an `Arc<[UiNodeId]>` per parent, patches it from `layout_order_generation`, and returns the same allocation while order is stable. The warm-path model therefore records one retained-order lookup and zero child sorts per visited parent. The remaining topology cost begins after that lookup, inside the Taffy bridge.

The selected direction is a retained, per-parent Taffy product cache. It preserves Zircon's existing recursive measure/arrange authority and uses Taffy only for the same direct-child allocation boundary that exists today. A single global Taffy mirror is rejected.

## Current Source Evidence

The current bridge behavior is explicit:

- `zircon_runtime/src/ui/layout/taffy_bridge/compute.rs:65` starts each parent by calling `begin_children`.
- `compute.rs:116-117` clears the complete `TaffyTree`.
- `compute.rs:98` creates a new leaf for each child.
- `compute.rs:198` creates a new parent with those children.
- `compute.rs:205` calls `compute_layout`.
- `zircon_runtime/src/ui/layout/pass/taffy_arrange.rs:89-131` performs this sequence for each visited supported parent and then reads every child frame.

The current order authority is also explicit:

- `zircon_runtime/src/ui/layout/pass/slot.rs:41-43` stores the layout-order generation and retained parent products.
- `slot.rs:100-130` patches only affected parent order products.
- `slot.rs:132-162` returns the retained ordered child slice.
- `slot.rs:339-381` sorts only while rebuilding an affected product.
- `slot.rs:681-750` proves stable `Arc` reuse and same-cardinality reorder replacement.

Incremental routing is real and must be retained:

- `zircon_runtime/src/ui/layout/pass/incremental.rs:144` builds required node paths rather than measuring the full tree.
- `incremental.rs:200` propagates a dirty descendant through parents whose layout boundary propagates child invalidation or whose container is auto-layout.
- `zircon_runtime/src/ui/layout/pass/engine.rs:76` can arrange only required children for independent Zircon-owned containers when the parent geometry is unchanged.
- `zircon_runtime/src/ui/surface/surface/rebuild/report.rs:43-45` already publishes Taffy tree-build and node-build counts for each rebuild.

The important distinction is therefore:

1. Dirty routing determines how many parent containers must be revisited.
2. The Taffy bridge determines how much redundant topology work each revisited parent performs.
3. Retaining Taffy nodes can remove topology construction and reuse internal compute caches, but it cannot assume that all ancestor solves or child-frame reads are unnecessary.

## Reference Engine Findings

### Unreal Slate, primary reference

Slate keeps persistent widget proxies and routes invalidation into ordered update heaps. `SWidget::SlatePrepass` checks `bNeedsPrepass` on the fast update path (`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp:674-713`), and `Prepass_Internal` computes children before caching the parent desired size and clearing the flag (`SWidget.cpp:1811-1844`).

`FSlateInvalidationRoot` inserts invalidated proxies into pre-update, prepass, or post-update heaps (`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp:299-340`). `ProcessPrepassUpdate` consumes proxies in sort order and skips descendants already covered by a processed ancestor range (`:1160-1218`), while `PaintFastPath` consumes the retained final update list (`:723-835`). Slow-path rebuilding is an explicit fallback, not normal pointer-event behavior.

The transferable rule is persistent identity plus invalidation-root-local work, not a literal port of Slate's proxy types.

### Fyrox, Rust comparison

Fyrox keeps per-widget `measure_valid`, `arrange_valid`, previous measure input, and previous arrange rectangle. `measure_node` returns immediately when both validity and available size match (`dev/Fyrox/fyrox-ui/src/lib.rs:1830-1843`), and `arrange_node` applies the same contract to the final rectangle (`lib.rs:1745-1758`). Invalidation is split into measure and arrange events (`dev/Fyrox/fyrox-ui/src/widget.rs:882-935`).

The transferable rule is exact input/result validity at the layout-owner boundary.

### Slint, Taffy comparison

Slint builds a local Taffy flex product in `FlexboxTaffyBuilder` (`dev/slint/internal/core/layout.rs:1404`) and returns compact geometry through `SharedVector`; `solve_flexbox_layout_with_measure` is at `layout.rs:1813`. This supports Zircon's current local-parent ownership boundary, but it does not prove that rebuilding the local product on every high-frequency refresh is acceptable.

### Bevy, retained Taffy comparison

Bevy owns a persistent `UiSurface` containing one `TaffyTree`, an entity-to-`NodeId` map, and child scratch storage (`dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs:67-75`). `upsert_node` updates style or creates a node only on first insertion (`:113-147`), `update_children` calls `set_children` on stable identities (`:157-175`), and removal explicitly retires the corresponding Taffy nodes (`:277-288`). This is direct evidence that stable Taffy identity is practical. Zircon should transfer that property at its existing per-parent solver boundary rather than copy Bevy's global tree ownership.

### Taffy 0.10.1 capability

The exact dependency used by Zircon exposes stable `NodeId` values and the operations required for retention: `set_children`, `set_style`, `mark_dirty`, `dirty`, and `compute_layout`. `mark_dirty` clears cached compute state and propagates to ancestors. The current `clear()` call discards both node identity and cached compute state before those facilities can help.

## Rejected Designs

### One global Taffy tree

Rejected because Zircon recursively owns content measurement, clipping, visibility, virtual materialization, and fallback layout. Current Taffy usage is deliberately a direct-child allocation solver. A global mirror would duplicate tree authority, complicate mixed Zircon/Taffy containers, and make partial fallback unsafe.

### Cache only final child frames

Rejected as the primary design. A final-frame cache can skip an exactly identical solve, but it still requires a collision-safe comparison of parent style, available size, ordered children, child styles, slot contracts, desired sizes, and visibility. It also does not reuse Taffy's internal sub-computation when only one child changes.

### Hash-only validity

Rejected. Hashes may accelerate lookup, but acceptance must compare the exact retained contract before returning geometry. A collision must not publish stale layout to render or hit testing.

### Skip all ancestor solves after a leaf change

Rejected. Auto-layout sibling placement and parent desired size can change. Dirty propagation may later be narrowed using dependency proof, but retained topology alone does not authorize it.

## Target Ownership

Each live Taffy-owned parent has one retained product:

```text
UiNodeId(parent)
  -> TaffyTree<()>
  -> stable parent NodeId
  -> ordered [(UiNodeId(child), Taffy NodeId, exact child style snapshot)]
  -> exact parent style snapshot
  -> last available size
  -> last child frame output
  -> last-used layout generation
```

The product belongs to the retained surface/layout owner, not a thread-local scratch pool and not the renderer. It is synchronized after responsive style and slot-order indexes are current, and before frame publication.

### Update protocol

1. Resolve the current visible ordered direct children using `UiLayoutSlotIndex`.
2. If the child identity/order contract changed, reconcile nodes and call `set_children`; remove retired nodes from the product.
3. Build the exact parent style. Call `set_style` only when it differs from the retained snapshot.
4. For each live child, build the exact child style. Call `set_style` only for changed children.
5. If the product is dirty or available size changed, call `compute_layout`; otherwise reuse the retained output.
6. Read child frames only when the solve ran or the caller requires fresh materialization.
7. Publish geometry through the existing Zircon layout cache and surface-frame path. Input never queries the Taffy cache directly.

### Lifecycle protocol

- Remove a parent product when the parent leaves the tree, changes to a Zircon-owned container, or enters an unsupported fallback contract.
- Treat node removal and re-insertion as distinct topology generations even when a numeric `UiNodeId` is reused.
- Bound retained products by live eligible parents; no time-only unbounded cache.
- On reconciliation or Taffy error, discard only the affected parent product and use the current Zircon fallback. Never publish a partial product.
- Keep scratch vectors for temporary ordered-child and frame materialization work; product retention does not justify new per-frame vectors.

## Required Observability

Existing counters remain authoritative:

- `layout_visited_node_count`
- `layout_measure_probe_node_count`
- `layout_arrange_probe_node_count`
- `layout_taffy_tree_build_count`
- `layout_taffy_tree_node_build_count`
- `layout_elapsed_micros`

The retained product needs additional counters before product acceptance:

- parent-product lookup hit/miss
- topology create/reconcile/remove counts
- child style update count
- parent style update count
- Taffy compute count and compute-cache reuse count
- child layout read count
- fallback/discard count by reason
- live retained parent and Taffy node high-water marks
- allocation count/bytes for the layout stage in the profile capture

These counters must be emitted per surface rebuild and aggregated by the existing source-bound profile pipeline. A deterministic work-count model is useful for algorithmic bounds but is not CPU, allocation, RSS, or input-to-present evidence.

## Acceptance Matrix

| Scenario | Required behavior |
| --- | --- |
| Stable frame | Zero layout visit, zero Taffy build, zero Taffy compute. |
| One child style change in a wide parent | One retained parent hit, zero topology node creation, one child style patch, no unrelated-parent access. |
| Parent available-size change | Zero topology creation, parent style/available-size update as needed, solve only affected parents. |
| Child insert/remove/reorder | Reconcile only the owning parent product; exact parity with the current bridge. |
| Nested auto-layout leaf change | Ancestor solves remain allowed; topology creation is zero after warmup. |
| Independent parent forest | Work is invariant as unrelated parent count grows. |
| Unsupported child/slot/value | Product is discarded or bypassed and existing fallback diagnostics remain exact. |
| Node removal or identity reuse | No stale Taffy node or frame can be observed. |

Parity tests must cover flex row/column, wrap, grid, block, hidden/collapsed children, slot padding/alignment/sizing, constraints, responsive changes, root resize, virtual materialization, fallback, and error recovery. Frame, clip, arranged tree, render extraction, and hit testing must observe the same published geometry.

## Milestone Plan

### M0: source-bound structural evidence

- Add a deterministic pressure tool guarded by the current bridge source.
- Bind the retained ordered-child authority separately from the Taffy bridge.
- Report order lookup/sort, topology node creation, solve count, and child-frame read count separately.
- Cover wide parent, nested auto-layout, independent forest, and resize scenarios.
- Write artifacts only to D:, E:, or F: and label them as non-product timing.

### M1: retained product data structure

- Add the per-parent product behind the existing Taffy bridge boundary.
- First preserve current behavior exactly: every visited parent may still solve and read all children, but topology creation becomes warm-cache work.
- Add lifecycle, fallback, and parity regressions before enabling compute reuse.

### M2: exact style/topology patching

- Reconcile child order only on structural generation changes.
- Patch exact changed parent/child styles.
- Prove unrelated-parent work remains constant at 64, 1,000, and 10,000 unrelated parents.

### M3: compute and output reuse

- Skip `compute_layout` only when Taffy's product is clean and available space is identical.
- Reuse child frames only under the same exact contract.
- Capture product CPU p50/p95/p99, allocations, RSS high-water, and input-to-present timing from a managed source-bound editor binary.

### M4: invalidation-root refinement

- Compare nested auto-layout propagation with Slate's invalidation-root range collapsing.
- Narrow ancestor work only where parent desired size and sibling placement dependencies prove it safe.
- Keep a conservative slow path for structural mutation and unsupported contracts.

## M0 Evidence

The current source-guarded work-count artifact is:

- `E:\zircon-profiles\runtime-ui-taffy-parent-product-pressure-20260831-r2.json`
- artifact SHA-256: `03CF55E7C53BBA4FBFAD5F0CF53CB75D67950BD7FF9DA6C1FB7D456B81489694`
- schema: `zircon.runtime.ui_taffy_parent_product_pressure.v2`
- critical source-set SHA-256: `11775770D3A7C3F1FA8404C768FF4F512611429A79E9198E0870611F9C97C34E`
- critical sources: five, including the retained order index
- source guard: ready
- product timing: false

Modeled measured-phase work after a successful warm product build:

| Scenario | Warm order sorts | Current Taffy node creates | Retained creates | Conservatively retained solves | Conservatively retained child reads |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1,000 single-child changes in one 1,024-child parent | 0 | 1,025,000 | 0 | 1,000 | 1,024,000 |
| 1,000 leaf changes through eight 8-child auto-layout ancestors | 0 | 72,000 | 0 | 8,000 | 64,000 |
| 1,000 changes in one 64-child parent with 10,000 unrelated parents | 0 | 65,000 | 0 | 1,000 | 64,000 |
| 120 resizes across 100 visible 16-child parents | 0 | 204,000 | 0 | 12,000 | 192,000 |

The aggregate avoided topology creation count is 1,366,000. This establishes a structural opportunity, not a latency result. M1 acceptance still requires allocation and CPU evidence from the actual retained implementation.

M0 validation on the bound source:

- focused pressure-tool unit tests: 9/9 passed
- Runtime UI performance contract suite: 191/191 passed
- Python bytecode compilation: passed
- owned-file trailing whitespace and final-newline checks: passed
- no Cargo command was run

## Current Gate

Production layout files remain heavily modified by concurrent owners in this shared worktree. M0 is therefore isolated to new tooling and documentation. M1 is deferred while work continues on other non-overlapping UI authorities; it must begin with lower-layer parity tests and retain the current slot-order, incremental routing, fallback, and surface publication contracts.

## Current-source static revalidation (2026-08-31)

The M0 pressure suite passes 9/9 and its source guard is ready against current
HEAD `f31fd06f69fdaedb70a0a56fe6d0268de1af83a6`. The exact five-source set is
`11775770D3A7C3F1FA8404C768FF4F512611429A79E9198E0870611F9C97C34E`.
The current artifact is
`E:\zircon-profiles\runtime-ui-taffy-parent-product-pressure-20260831-r2.json`
with SHA-256
`03CF55E7C53BBA4FBFAD5F0CF53CB75D67950BD7FF9DA6C1FB7D456B81489694`.

Current source still clears the Taffy tree and recreates leaf and parent nodes
for every visited auto-layout parent before calling `compute_layout`. Across
the four canonical scenarios, a retained parent product would avoid 1,366,000
node creations after warmup. The model deliberately retains all required
compute calls and child-layout reads: it does not infer cache hits or latency
improvement from topology retention.

Current source also proves that stable parent order does not sort on each
visit. The pressure model records 1,000 retained-order lookups and zero sorts
for the wide-parent scenario, followed by 1,025,000 Taffy node creations. This
separates the completed order-cache work from the remaining topology defect.

Together with the slot, edge-evidence and incremental-layout suites, the
focused static layout total is 45/45. M1-M3 remain unimplemented and blocked by
shared production ownership, not by missing architecture: per-parent product
lifecycle, exact style/topology patching, compute reuse, parity tests and
managed product CPU/allocation/RSS/input-to-present evidence are still needed.
