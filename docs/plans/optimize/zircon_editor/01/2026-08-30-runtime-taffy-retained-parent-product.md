# Runtime UI Retained Taffy Parent Product Review

Date: 2026-08-30

Status: M1 retained topology, M2 exact child-contract receipts, and M3 exact solve/output reuse implemented; managed Rust and product acceptance pending

Source binding used for this review:

- revision: `9963f8eb72e2d725d2536eb50b393b30387a1ffa`
- worktree: dirty, with the scoped M1/M2 layout candidate and unrelated concurrent owner changes
- validation restriction: no raw Cargo; product timing remains gated on a source-bound managed binary

## Outcome

Zircon does not rebuild one global Taffy tree on every input event. Before M1 it rebuilt a small Taffy tree for every visited auto-layout parent: one leaf per visible direct child plus one parent. The M1 candidate now retains one independent Taffy product per supported parent in the surface-owned `UiLayoutSlotIndex`. Stable parents reuse all Taffy node identities, style changes call `set_style` only for exact differences, and insert/remove/reorder reconciles only the owning product.

For a set `P` of visited Taffy-owned parents, the removed pre-M1 topology construction work was:

```text
tree_builds = |P|
taffy_nodes_created = sum(parent in P, visible_children(parent) + 1)
layout_reads = sum(parent in P, visible_children(parent))
```

The M1 candidate reduces this topology creation to zero after warmup when identity/order is stable, and to only newly inserted leaves during local structural reconciliation. M2 consumes the incremental layout context's exact required direct-child receipt: stable non-grid parents validate and rebuild styles only for those `K` children. Root resize with a stable child contract uses `K=0`. M3 now skips `compute_layout` only when the retained parent is clean and its exact available-size key matches. When parent frame and inherited clip also match the last publication, it publishes only the `K` required child frames; translation or clip changes still publish all `C` frames. These are source-bound structural claims, not product latency claims.

The ordered-child lookup is no longer part of this defect. `UiLayoutSlotIndex` owns an `Arc<[UiNodeId]>` per parent, patches it from `layout_order_generation`, and returns the same allocation while order is stable. The warm-path model therefore records one retained-order lookup and zero child sorts per visited parent. The remaining topology cost begins after that lookup, inside the Taffy bridge.

The implementation preserves Zircon's existing recursive measure/arrange authority and uses Taffy only for the same direct-child allocation boundary that existed before M1. A single global Taffy mirror remains rejected.

## Current Source Evidence

The current retained bridge behavior is explicit:

- `zircon_runtime/src/ui/layout/taffy_bridge/product_cache.rs` owns the per-parent `TaffyTree`, stable parent node, ordered-child revision, `node_id -> child` index, and exact style snapshots.
- `product_cache.rs` removes only the requested parent product from the map while updating it, so a failed partial update is discarded rather than published.
- Stable exact receipts look up only changed child IDs and call `set_style` only when an exact style differs.
- The retained product stores exact available size plus parent-local child frames; solve reuse requires both an unchanged available-size key and `TaffyTree::dirty(parent) == false`.
- Exact output reuse additionally matches the last parent frame and inherited clip. A translated parent reuses Taffy solve work but republishes every child in the new absolute coordinate space.
- Structural changes preserve matching child `NodeId` values, create only new leaves, call `set_children`, and remove retired leaves.
- `zircon_runtime/src/ui/layout/taffy_bridge/compute.rs` materializes either the complete child contract or only the exact receipt into scratch buffers and delegates solve/output to the retained product.
- `zircon_runtime/src/ui/layout/pass/taffy_arrange.rs` admits exact receipts only for stable non-grid membership and order revision; membership, order, Grid, missing-product and eligibility changes use the complete path.

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
5. If the product is dirty or available size changed, call `compute_layout`; otherwise reuse the retained solve.
6. Read all Taffy child layouts only after a solve. With identical parent frame/clip, publish only the exact child receipt; otherwise synthesize every absolute child frame from retained parent-local output.
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

- Implemented in the current candidate behind the existing Taffy bridge boundary.
- Every visited parent still solves and reads all children, while stable topology creation is now warm-cache work.
- Lower regressions cover stable reuse, one-child style patching, and inserted-child-only creation. Fallback and lifecycle cleanup are encoded in the production owner; managed execution remains pending.

### M2: exact style/topology patching

- Topology reconciliation and exact `set_style` suppression are implemented.
- Exact direct-child contract/style materialization is implemented from the incremental required-child authority.
- Lower source regressions cover `K=0` root resize, `K=1` child change, active-membership fallback and order-revision fallback; managed execution remains pending.
- The deterministic forest scenario keeps 10,000 unrelated parents at zero visits.

#### Accepted M2 receipt algorithm

M2 must remove the two full direct-child contract walks that currently precede
the retained Taffy update; merely suppressing `set_style` calls is insufficient.
The incremental layout context already owns the required authority:

- `layout_source_node_ids` identifies nodes whose layout inputs actually changed;
- `required_children_by_parent` projects a descendant mutation to the exact
  direct child whose desired-size contract may have changed;
- `pending_mutation_node_ids` closes the lower-level mutation gap by projecting
  every still-live mutation through the same direct-child receipt; an unknown
  mutation on the parent itself rejects the fast path;
- the retained ordered-child index publishes a parent-local monotonic revision
  whenever child identity/order or slot order changes.

For a retained non-grid parent, the fast path is admitted only when the parent
is not itself a layout source, structure is stable, and the cached product's
ordered-child revision equals the current revision. It validates visibility and
eligibility only for the exact required direct children, rebuilds only those
styles, and patches the matching retained Taffy leaves. An empty receipt during
a root-size change performs zero child-contract/style visits; the parent style
and available size still update and Taffy still solves and reads every child
layout.

Parent/container changes, Grid child-placement changes, order-revision
mismatch, missing products, or a changed child's active/hidden membership select
the existing full contract scan before mutation. The fallback is observable and
must not silently publish a partially updated product. This preserves M1's
conservative solve/output boundary while changing child-contract preparation
from `O(C)` to `O(K log C)` for a stable parent, where `K` is the exact changed
direct-child set. Taffy solve and output remain `O(C)` until M3 evidence proves a
safe reuse condition.

### M3: compute and output reuse

- Implemented in the current static candidate: skip `compute_layout` only when Taffy's parent is clean and available space is identical.
- Exact K-frame publication additionally requires unchanged parent frame and inherited clip; translation/clip changes retain full child publication.
- Lower source regressions cover solve reuse with an unchanged child style, conservative full publication under translation, and mandatory recompute under resize.
- Product CPU p50/p95/p99, allocations, RSS high-water, and input-to-present timing still require a managed source-bound editor binary.

### M4: invalidation-root refinement

- Compare nested auto-layout propagation with Slate's invalidation-root range collapsing.
- Narrow ancestor work only where parent desired size and sibling placement dependencies prove it safe.
- Keep a conservative slow path for structural mutation and unsupported contracts.

## M0-M3 Evidence

The current source-guarded work-count artifact is:

- `E:\zircon-profiles\runtime-ui-taffy-parent-product-pressure-20260901-r6.json`
- artifact SHA-256: `B69FB16AF4EF13EC4D64F4120CCBB43A1DE7051BCFAC94785F0004FE49287B02`
- schema: `zircon.runtime.ui_taffy_parent_product_pressure.v5`
- critical source-set SHA-256: `8BA3F101E1AF843C1A9374E1369813A2EDFCCC5CEF47F687AA93FB28384F0D44`
- critical sources: eight, including incremental receipt routing, retained order revision and parent-product implementation
- source guard: ready
- product timing: false

Modeled measured-phase work after a successful warm product build:

| Scenario | M1 contract visits | M2 contract visits | Creates | M3 solves | Taffy reads | Published child frames |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 single-child style changes in one 1,024-child parent | 1,024,000 | 1,000 | 0 | 1,000 | 1,024,000 | 1,024,000 |
| 1,000 leaf changes through eight 8-child auto-layout ancestors | 64,000 | 8,000 | 0 | 8,000 | 64,000 | 64,000 |
| 1,000 changes in one 64-child parent with 10,000 unrelated parents | 64,000 | 1,000 | 0 | 1,000 | 64,000 | 64,000 |
| 120 resizes across 100 visible 16-child parents | 192,000 | 0 | 0 | 12,000 | 192,000 | 192,000 |
| 1,000 subtree invalidations with unchanged child style in one 1,024-child parent | 1,024,000 | 1,000 | 0 | 0 | 0 | 1,000 |
| 1,000 translations of one 1,024-child parent | 1,024,000 | 0 | 0 | 0 | 0 | 1,024,000 |

The aggregate avoided topology creation count is 3,416,000 for the six modeled warm scenarios. This describes the implemented structural delta, not a latency result. Acceptance still requires managed Rust execution plus allocation and CPU evidence from the actual editor binary.

M0 validation on the bound source:

- focused pressure-tool unit tests: 12/12 passed
- seven scoped Rust files pass standalone `rustfmt --check`
- the ten owned source/report paths pass `git diff --check`
- the repository UI architecture audit reports Taffy production hits/files
  `242/16`, zero baseline mismatches, zero missing document anchors, and zero risks
- the lower Rust and product suites have not run against M1-M3; earlier Runtime UI
  results are not treated as current acceptance evidence
- no Cargo command was run

## Current Gate

The M1-M3 source candidate is implemented without touching UI12's shared DPI work in `surface/rebuild.rs` or `runtime_window_input_pump/metrics_dirty.rs`. Seven owned Rust files pass standalone formatting, the combined source/report diff is whitespace-clean, and the source-bound Python pressure suite passes 12/12. The shared architecture-contract file has a pre-existing external rustfmt delta outside this candidate's local ownership assertion. No Cargo command was run.

Managed lower-layer Rust tests, editor build, allocation/CPU/RSS capture, and input-to-present evidence remain gated on explicit managed validation authorization and a stable shared closure. The M3 candidate remains unaccepted until its exact dirty, available-size, frame, and clip contract runs in lower-layer and product tests.

## Current-source static revalidation (2026-09-01)

The pressure suite is ready against current HEAD `9963f8eb72e2d725d2536eb50b393b30387a1ffa` plus the scoped dirty M1-M3 candidate. Its eight-source set is `8BA3F101E1AF843C1A9374E1369813A2EDFCCC5CEF47F687AA93FB28384F0D44`.

The current implementation and model agree on the conservative boundary: stable warm parents create zero Taffy nodes and exact receipts remove unrelated child-contract preparation. A changed child style still retains 1,000 solves and 1,024,000 child-layout reads in the wide-parent case; an unchanged style contract reduces both to zero and publishes only the 1,000 exact child frames. Translation also reduces solve/read work to zero but retains 1,024,000 frame publications because absolute geometry changed. Resize retains 12,000 solves and 192,000 reads/publications. Unrelated parents remain unvisited by the existing incremental routing authority.
