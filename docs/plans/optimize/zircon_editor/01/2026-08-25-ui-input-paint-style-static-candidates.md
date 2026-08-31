---
title: Retained UI Input Paint and Style Static Candidates
category: zircon_editor
report_id: Editor01-ui-input-paint-style-static-candidates-2026-08-25
date: 2026-08-25
session_id: runtime09-ui-layout-validity-20260825
implementation_status: static_candidate
validation_status: managed_validation_pending
---

# Retained UI Input Paint and Style Static Candidates

## Scope

This slice continues the retained-UI architecture review without treating a local source change as
product acceptance. It covers clipped editor table paint, bounded runtime table-cell
materialization, text-layout cache ownership, generic menu and command input filtering, a common
runtime pseudo-state probe, arranged input patch ownership, and a read-only SVG/GPU cache audit.
It also covers the rich-table column-width constraint solver, duplicated tree-selection flattening
algorithms, geometry-only workbench layout commands, Performance Timeline clip materialization and
static-key reuse in virtual-window state updates. It also covers generation-keyed reuse of the menu
pointer chrome stencil during pointer-surface recomputation and a borrowed Asset Browser folder
index for breadcrumb construction. The input-route diagnostics path is now reviewed separately,
with an implementation-ready execution/trace authority split, but its production migration has not
started beyond the first route-ownership transfer. Rich-link release and editable-text pointer hit
testing now use the retained per-node render-command range instead of scanning every render command. It does not claim that
button response, native resize, popup routing, Inspector
virtualization, or current-source product latency is fixed.

The canonical parent report remains
`docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md`. An external
Windows mmap writer currently prevents a scoped append to that file, so this child record preserves
the evidence without rewriting or truncating the parent.

## Current-source Candidates

| Path | Previous work | Candidate behavior | Complexity boundary |
| --- | --- | --- | --- |
| Template table paint | Offscreen rows could reach cell extraction and text normalization | Reject empty or clipped row geometry before reading cells; inspect archived cell shape with a borrowed token probe | Offscreen row materialization becomes `O(1)` and performs no cell-sized allocation |
| Runtime collection table | The renderer converted every metadata cell into `Vec<String>` and only then retained four columns | Lazily convert at most four non-empty cells into a fixed stack array; parse the compact label fallback without a token vector | Owned cell materialization becomes `O(min(C, 4))` strings with no cell-container heap allocation; scanning may pass empty values but never owns more than four |
| Text layout cache | Stable lookup owned a text key; capacity pressure cleared all 2,048 entries; shaped advance validation built two vectors | Borrow `&str` for lookup, own only on miss, evict one entry, and validate adjacent advances in constant scratch space | Stable hit has no text-key allocation; eviction becomes `O(1)` entry work; validation stays `O(G)` time and `O(1)` scratch |
| Command palette | ASCII fields allocated lowercase strings; navigation reparsed commands and scanned disabled IDs per candidate | Borrowed ASCII prefix/window matching, one filter projection per transaction, and set-backed disabled membership | Worst disabled filtering changes from `O(F*D)` to expected `O(F+D)` |
| Command palette filtered-row rendering | The renderer reparsed the command list, then linearly searched every command for each published filtered ID | Build one borrowed ID/label-to-index map in source order and resolve filtered rows through it | Lookup changes from `O(F*C)` to expected `O(C+F)` time and `O(C)` borrowed index scratch while preserving first-match semantics |
| Generic menu | Typeahead and directional navigation repeatedly scanned disabled/filter lists; filter sync parsed two top-level option projections | One borrowed eligibility/filter index per input transaction and one recursive search model | Candidate eligibility becomes expected `O(1)` membership after one `O(D+F)` build |
| Runtime pseudo-state descendant probe | A changed node cloned component, ID, and classes before checking ancestor pseudo-state selectors | Validate the node, return immediately when the ancestor-pseudo index is empty, and otherwise match Type/ID/Class/Host directly against borrowed tree metadata | Probe performs no selector-fact allocation; non-empty work is `O(A*T*C)` comparisons for ancestor segments/tokens/class membership without copied metadata |
| Arranged input patch | Parent input-policy or pointer-events changes expanded to descendants, then reconstructed every affected `UiArrangedNode`, cloning path, children, control ID and replacing slot/geometry values that did not change | Validate structure by borrowing the tree; patch only the actually changed node's input scalars and changed control ID; retain descendants only in the returned hit-grid affected set | Owned structure work changes from `O(A)` complete-node clones to `O(K)` scalar patches, where `K` is directly changed nodes and `A` includes affected descendants; validation remains `O(A*H_clip)` without owned node clones |
| Rich-table column shrink | A single proportional scale was followed by an independent per-column minimum clamp, so a feasible width budget could still overflow | Solve the proportional scale with every minimum lower bound inside the budget equation; preserve minimums explicitly when the budget itself is infeasible | Fixed 24-pass search is `O(C)` time with a constant factor and `O(1)` scratch; no temporary column container |
| Tree selection/expansion | Component-state and surface-default paths each flattened all tree IDs into owned strings; linear `push_unique` made flattening `O(N^2)`, while range selection rescanned all disabled IDs for every candidate | Preserve first-seen tree order in a borrowed ID vector with hash deduplication; build disabled membership once per range transaction; clone only selected output IDs | Current fallback changes to expected `O(N + R + D)` time and `O(N + D)` pointer/index scratch, versus `O(N^2 + R*D)` and `O(N)` owned ID strings |
| Drawer/split geometry command | Every changed extent/ratio globally normalized drawers, cloned the active drawer map, collected all instance placements, cloned all open instances and rebuilt window/session registries; left/right drawer resize dispatched two complete events | Classify geometry-only commands once; patch target plus legacy mirror locally; skip session-structure metadata; atomically resize a drawer region through one journaled command | Drawer mutation is `O(1)` over at most two slots and one event instead of two; split mutation remains `O(path)`; structure-metadata work is zero for these commands |
| Performance Timeline visual rows | Every logical frame, span and hotspot row produced host template nodes and formatted strings even when the list clip could not display them; each frame produced four nodes | Derive the exact intersecting row range from the list clip, then fetch only those indexed rows from `ModelRc` | Dynamic visual-node construction becomes `O(V)` rather than `O(N)`; frame-node output is bounded by `4V`, where `V` is the clip-intersecting row count |
| Virtual-window state update | Every visible-range or page event recreated owned `String` keys for all canonical and compatibility fields before replacing values already present in component state; changed metadata fields repeated the same allocation in the batched surface write | Update existing static-key slots through borrowed lookup and allocate a key only on first insertion at both layers; preserve reference-source clearing, reflected updates and dirty merging | A repeated reducer event performs zero state-key allocations instead of up to 17 (visible range) or 6 (page); each changed metadata field also removes one temporary key allocation; map work remains `O(K log S)` for `K` published fields and state size `S` |
| Menu pointer chrome stencil | Every pointer-surface recomputation projected `workbench_menu_chrome.zui` again using the current shell dimensions, even though only its fixed origin, height and gap stencil is consumed | Cache one normalized stencil by `ViewTemplateResourceGeneration`; translate it to the live shell and continue measuring each label width from runtime glyph metrics | Stable resource generations perform `O(1)` cache lookup plus `O(M)` translation/width work for `M` menus and zero ZUI projection; resize is not a cache key; hot reload replaces the single entry |
| Asset Browser breadcrumb | After selecting a folder, every parent step linearly scanned a copied vector of the complete tree/visible folder collection and cloned each display-name segment | Build one borrowed folder-ID index with tree-first parent precedence, borrow path segments and stop cycles by borrowed folder ID | Parent resolution changes from `O(F*D)` to expected `O(F+D)` time for `F` folder records and path depth `D`; transient ownership is `O(F+D)` references with no per-segment `String` clone |
| Asset reference row layout | Every References/Used By layout inserted each row-kind width into a `BTreeMap<usize, f32>` and then performed ordered lookup for all four row nodes | Store contiguous generated row indices in a bounded `Vec<Option<f32>>`; reject suffix indices at or beyond node count and lazily compute one malformed-row fallback | Existing full-row layout changes from `O(R log R)` index work to `O(R)` time and `O(R)` contiguous scratch; it does not change the still-unbounded `4R` physical-node topology |
| Input route diagnostics (first ownership slice) | Every pointer event materializes owned preview/focus/popup/route-step diagnostics; bubble/root vectors were cloned and focus-route construction performed a linear arranged-node search for every ancestor | Move bubble/root vectors into the trace and resolve focus ancestry through the retained arranged-node index; terminal design gates serialized trace materialization under Disabled/ErrorsOnly/Sampled/Full policy | Implemented slice removes two vector clones and changes focus lookup from `O(H*N)` to `O(H log N)`; terminal normal diagnostic overhead targets `O(1)` scalar policy work and zero diagnostic allocation |
| Rich-link click command lookup | A primary release with an already resolved target reverse-scanned the complete render command list, and the owned link href was cloned again into the effect | Query the retained per-node command range, preserve reverse paint order and fall back only for deliberately unindexed extracts; move the owned href into the effect | Indexed product path changes from `O(R)` command visits to `O(log N + C_target)` for `N` rendered nodes and `C_target` target commands, with one fewer `String` clone |
| Pointer input DTO ownership | Normal pointer dispatch cloned metadata, then cloned the complete pointer input, duplicating owned window/surface IDs before helpers usually returned without text/link work | Borrow the original metadata/input through routing and the optional text probe, retain pointer ID/source/click count as Copy scalars, and move the original pointer once into the result | Normal non-text pointer events perform zero complete input/metadata clones, eliminating up to four ID-string clone allocations; one scalar-field pointer-event clone remains |
| Pointer dispatcher ownership | Every preview/target/bubble node with handlers cloned the complete pointer route; candidate dispatch cloned stacked/root vectors, allocated a captured vector and used ordered visited membership; result construction cloned the complete reply | Reuse one owned handler context, traverse an optional target plus borrowed candidate slices, use a pre-sized unordered visited set, and move the reply into the final result | Route ownership changes from about `O(H^2)` copied IDs to two fixed `O(H)` copies; candidate-vector allocations become zero, visited membership changes from `O(V log V)` to expected `O(V)`, and one effect-vector clone is removed |
| Pointer hover membership | Every changed hover stack computed entered and left with nested slice `contains`, making dense overlapping hit stacks quadratic | Keep allocation-free linear membership while `C*P <= 64`; above that budget, build one capacity-sized hash membership table and reuse it for the reverse difference | Common small paths retain their previous allocation profile; dense paths change from `O(C*P)` to expected `O(C+P)` while preserving current/previous route order |
| Navigation dispatch/input ownership | Every navigation cloned its complete bubbled/root candidate vector, used ordered visited membership, cloned the complete route once per handler-bearing node, then cloned the event and route vectors again for input diagnostics | Move the source route into result, borrow its candidates, use a capacity-sized hash set and one handler context; move result vectors into trace, retain only the required bubbled-to-focus copy, and move reply/event ownership | Candidate-vector allocations become zero, visited membership changes from `O(V log V)` to expected `O(V)`, dispatcher route copying becomes one fixed context copy, and the input tail removes one event, one bubbled and one root-vector clone |
| Routed diagnostic step capacity | Every bubble/focus diagnostic route started from `Vec::new()` even though preview, active path and possible terminal upper bounds were already known | Preallocate the exact safe upper bound; retain `Vec::new()` when an empty unrouted result cannot append a terminal step | Non-empty routed step materialization performs at most one vector allocation; empty unrouted work remains zero-allocation |
| Editable-text pointer command lookup | Each caret/selection pointer press or drag already knew its target node but linearly scanned the complete render command list for its text layout | Query the same retained per-node command range used by rich links and keep the legacy full-list fallback only for unindexed lower fixtures | Indexed product path changes from `O(R)` command visits to `O(log N + C_target)` while preserving first-layout lookup order |
| Popup ID uniqueness lookup | Every popup candidate created an owned path/fallback ID string only to compare it with the requested ID | Borrow node paths directly and parse the canonical `node:<u64>` fallback without formatting or cloning | Retains `O(N)` popup-node traversal but removes `O(P)` transient ID-string allocations for `P` popup candidates; terminal target is a generation-owned popup ID index |
| Popup tree-to-stack reconciliation | Each retained stack entry scanned all authored popup records, then every open authored popup rescanned the retained stack | Index open records by node with borrowed `(popup_id, owner)` facts, retain in stack order, then track retained popup nodes in a set before adding missing records | Reconciliation changes from `O(P*S)` to expected `O(P+S)` time and `O(P+S)` index scratch for authored popup count `P` and stack size `S` |
| Popup dependency impact | Render full-extract and stack-reconciliation decisions repeated nearly identical popup/changed-node analysis; each control popup also walked the owner ancestor chain once per changed node | Produce both domain flags in one analysis, pre-index changed control IDs/missing nodes, and intersect each popup/owner chain with the changed-node set once | Shared layout branch performs one analysis; common dependency work changes from about `O(P*C*H)` to expected `O(C + P*H)` for changed count `C` and tree height `H` |

ASCII matching keeps the prior Unicode fallback: non-ASCII values still use Unicode lowercase
matching. Empty-query behavior and the previous trim rules are preserved. Menu grouping behavior is
also preserved: descendants of an ID-less grouping map remain searchable but are not promoted into
default top-level focus candidates when there is no query.

## Withdrawn Candidate

A terminal style-rule multiway merge was implemented and then removed during review. Its bucket and
heap scratch was reused only within one subtree application. The single-node hover/press path would
still construct a fresh bucket vector and binary heap per event, potentially increasing allocator
pressure over the existing one-vector concatenate, sort, and deduplicate implementation.

`zircon_runtime/src/ui/v2/style/rule_index.rs` is therefore restored to HEAD. Any future rule-index
merge must use scratch that survives across input events, or publish generation-owned selector facts,
and must win an allocator plus input-p95 comparison before integration.

## Deferred Pointer-state Candidate

The current hover/press state path still sends a one-node change through the generic batch routine.
For one changed node it constructs a one-entry `BTreeSet`, a second descendant-affecting
`BTreeSet`, a `Vec` of minimal roots, and a third root-membership `BTreeSet`. It also performs an
ancestor walk while minimizing roots and another ancestor walk while deciding whether the changed
node is covered. With tree height `H` and `K` changed nodes, the bookkeeping is bounded by roughly
`O(K * H * log K)` before any required subtree restyle; ordinary single-node hover therefore pays
allocator and parent-chain costs intended for batch mutations.

The target design is a specialized one-node path: validate and query the node once; when its state
cannot affect descendants, apply only that node's runtime state style and mark it render-dirty with
no ordered-set allocation or root minimization. When descendant selectors can match, reuse the
existing subtree path. The batch entry point remains for genuinely multi-node changes. This should
ultimately consume generation-owned selector facts so the descendant-affecting decision does not
clone component, ID, or class data on every pointer event.

The generic pointer-state batching path is externally modified and was not edited by this slice.
The style probe it calls is owned by this slice and now performs its descendant-affecting test with
borrowed tree metadata, including the non-empty ancestor-selector case. Acceptance still requires a 1,000-event
alternating hover/press regression that records zero bookkeeping allocations in the no-descendant-
selector case, constant node visits for a fixed-depth target as unrelated tree size grows, identical
computed style/damage for descendant-selector cases, and an improved input-to-damage p95 on the
current-source editor binary.

## SVG and GPU Cache Audit

Current source has distinct cache levels rather than one cache:

- The CPU pixel cache is bounded by 4,096 entries and 64 MiB and is queried before candidate path
  construction or filesystem probing.
- The SVG parse-tree cache is bounded by 1,024 entries, indexes path aliases, caches missing results,
  and uses targeted content-fingerprint invalidation.
- Async cold loads deduplicate identical cache keys and reject stale binding/cache epochs.
- Vector raster targets quantize 1 through 512 square sizes from 512 exact identities to 64 buckets.
- WGPU keeps textures by `(resource_key, resource_generation)` and skips writes for resident
  generations. Whole image preparation is `O(1)` only for an unchanged draw-list generation with no
  staged resource or unversioned external provider.

The remaining unproven risks are raw locator aliases producing separate CPU raster entries before
content-addressed GPU identity, and changed draw-list generations walking all image sources for
resident hits. Visual and RHI paths contain external in-progress changes, so this slice does not edit
or absorb them.

## Arranged Input Patch Evidence

`zircon_runtime/src/ui/surface/arranged.rs` was clean before this slice. Its prior
`patch_arranged_tree_input` called `arranged_node_from_tree` for every affected descendant and then
replaced the complete arranged node. This was unnecessary for a parent input-policy change:
descendants need to be returned so the hit index can recompute inherited policy, but their node
path, children, layout, clip, z/order and slot are not input mutations.

The patch now uses a two-pass transaction. `affected_node_ids` still includes every descendant
required by `UiHitTestIndex::patch_arranged_geometry`; the first pass compares structure by
reference and returns a fallback before any write, while the second pass commits six copyable input
facts and owns a control ID only when it actually changed. There is no complete-node replacement or
additional patch vector. A lower Rust test records the path-string and children-vector pointers for
a three-level tree, changes root input policy/pointer events plus a descendant input fact, and
asserts that all descendants remain affected, inherited policy changes, and the recorded
allocations remain identical.

This removes complete-node allocations and replacement from the input patch, but it does not yet
remove the descendant validation and hit-entry work required for inherited policies. A future
generation-owned inherited-input epoch could reduce that cost further, but only if hit testing can
consume the epoch without scanning ancestors on the event hot path.

## Rich-table Width Solver Evidence

`zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs` previously computed one shrink ratio
from all shrinkable preferred widths and then independently applied `.max(minimum)` to each result.
For preferred widths `[20, 190]`, minimum `20`, and shrink budget `100`, that procedure returned
approximately `[20, 90.5]`: the total still exceeded the budget even though `[20, 80]` is feasible.
This creates avoidable table overflow and extra clipping/paint pressure during narrow-window layout.

The candidate now solves `sum(max(preferred_i * scale, minimum)) <= shrink_budget` directly. A
fixed 24-step monotonic search avoids allocation and has bounded linear work in column count. Fixed
columns are subtracted before solving; when fixed widths plus minimum shrinkable widths cannot fit,
the documented result retains the minimum widths and allows overflow rather than producing invalid
geometry. Lower Rust regressions cover a column reaching the lower bound, a mixed fixed/shrinkable
case, and an infeasible budget. They are present but remain unexecuted until managed Cargo is
authorized.

## Tree-selection Fallback Evidence

Both `component/state_reducer/tree_view.rs` and
`surface/surface/default_interactions/tree_view_support.rs` independently flattened a complete
tree for selection, expansion, rename and reorder. Each encountered ID was cloned before a linear
duplicate check against all prior IDs. Range selection then called a recursive disabled-option
scan once per row. Large Hierarchy interactions could therefore pay `O(N^2 + R*D)` even before the
subsequent binding and presentation work.

The fallback collectors now retain ordered `&str` views and use a hash set only for uniqueness.
Range selection builds one borrowed disabled-ID set and clones only the IDs that become the output
selection. Direct single-node disabled checks retain the allocation-free recursive probe because
one scan is cheaper than constructing a set. Lower Rust tests verify first-occurrence ordering,
pointer identity into the source strings, `UiValue::Flags`, and TOML table identity aliases.

This is deliberately not the terminal Hierarchy design. It still builds an `O(N)` pointer view for
an event. The generation-owned collection/index authority specified in
`2026-08-25-pane-projection-generation-cache-design.md` must eventually supply stable ID-to-row,
disabled and visible-range indexes so ordinary selection becomes `O(1)` target lookup plus `O(R)`
range output. `tree_view_support.rs` also contains pre-existing, uncommitted component-to-role
semantic changes that are not owned by this slice; a managed integration must attribute hunks and
must not absorb or revert that external work.

## Geometry-only Layout Command Evidence

`LayoutManager::apply` previously ran `normalize_drawer_active_selection` across every activity
window and cloned the active drawer map into the legacy mirror after any changed command, including
`ResizeSplit` and `SetDrawerExtent`. `EditorUiHost::apply_layout_command_inner` then cloned that
drawer map again and called `recompute_session_metadata`. That routine collects all layout
placements, retains and updates every open instance, clones all open instances, rebuilds the window
registry, scans active drawers, prunes animation/UI-asset sessions and synchronizes layout windows.
None of that structure work is required for an extent or split-ratio change.

`LayoutCommand::is_geometry_only` is now the single classifier consumed by manager and host. A
single drawer extent updates the active target and matching legacy entry directly. The new
serialized `SetDrawerRegionExtent` command validates all region slots before writing any, then
updates the left/right pair in one event; the old retained-host loop emitted two complete event
transactions and could partially mutate the first slot before a second-slot error. `ResizeSplit`
and both extent commands skip global drawer repair and session metadata reconstruction.

This does not yet make resize geometry-only end to end. The event still publishes
`LayoutChanged`, `PresentationChanged` and `ReflectionChanged`, so downstream host invalidation can
still enter the broad layout/presentation path described in
`2026-08-25-window-metrics-geometry-publication-design.md`. Acceptance must prove one event per
pointer sample, zero session-metadata rebuilds, zero structure-generation changes and bounded
geometry publication in the current-source product profile.

## Performance Timeline Clip Materialization Evidence

`pane_data_conversion/performance_timeline.rs` was clean before this slice. The previous frame,
span and hotspot builders enumerated their complete `ModelRc`, created owned control IDs and labels,
and appended nodes regardless of the clipped list viewport. A 10,000-frame payload therefore
created approximately 40,000 dynamic frame nodes before later clip rejection.

The candidate computes the half-open index range whose row rectangles intersect the list clip and
calls `row_data` only for those indices. It preallocates at most four nodes per visible frame and one
per visible span or hotspot. A lower Rust regression keeps 10,000 logical frames in the item source
while requiring fewer than 100 projected nodes for a 240 by 160 list and rejects frame-label nodes
below the clip. Source contracts also reject a return to complete `.iter().enumerate()` traversal.

This is a bounded visual-materialization slice, not the terminal retained collection design. The
pane conversion still maps the complete logical timeline into host row DTOs before node projection,
and the dynamic-node builder has no authoritative scroll-window or retained row-identity input.
Interactive scrolling therefore still requires the generation-owned item source and visible-range
authority specified in `2026-08-25-pane-projection-generation-cache-design.md`; this candidate does
not satisfy the full host-model-build or scrolling-virtualization acceptance gates.

## Virtual-window Static-key Evidence

`component/state_reducer/windowing.rs` was clean before this slice. Both `SetVisibleRange` and
`SetPage` publish a fixed schema, but each event previously called `property.to_string()` for every
field before `BTreeMap::insert`. Existing map keys are retained by value replacement, so those
temporary strings were allocator work with no change in state identity.

The candidate uses one `set_static_value` helper. It clears the field's drag/drop reference source
exactly as the generic reducer did, updates an existing slot through borrowed `get_mut`, and allocates
the static key only when the slot is missing. The first event still creates the required canonical
and compatibility fields; subsequent visible-range events allocate zero property keys for 17 writes,
and subsequent page events allocate zero for 6 writes. Source contracts require the conditional
insert shape and reject an unconditional return to `super::set_value` in both reducers.

The clean lower `property_mutation/metadata_batch.rs` path now applies the same rule to changed
metadata attributes. It continues to skip unchanged aliases before mutation, emits exactly one
reflected update per changed property, merges dirty domains and marks the owner once. The reflected
update intentionally still owns the property name because it survives the mutation call; only the
redundant temporary key used to replace an existing metadata slot is removed. Existing Rust
virtualization and metadata-batch tests cover the complete alias/value, change-count and dirty result
but remain pending under managed Cargo.

## Menu Pointer Stencil Evidence

`menu_pointer/build_host_menu_pointer_layout.rs` previously called
`build_view_template_node_projection` on every pointer-surface layout build and supplied the current
shell width and height. The projected asset contributes a fixed seven-slot stencil: origin, row
height and inter-slot gap. Live widths already come from runtime glyph measurement, so rebuilding the
asset projection during a window-size or pointer-state transaction did not add layout authority.

The candidate keeps one thread-local normalized stencil keyed by the complete
`ViewTemplateResourceGeneration` (compiled template, design tokens and font generation). A stable
generation returns the copied seven-frame array; a generation change projects the ZUI once at a
fixed reference size and replaces the entry. The live path translates the stencil by the current
menu-bar origin and retains the existing glyph-aware width and extension-menu behavior. ZUI remains
the geometry authority, hot reload remains generation invalidated, and shell dimensions cannot grow
cache cardinality. `ui.menu_pointer.stencil_projection_build_count` records cold rebuilds for the
product profile.

This removes redundant template projection from stable resize and pointer-surface recomputation, but
does not claim that the surrounding bridge or surface rebuild is incremental. Acceptance requires a
200-step resize and repeated menu hover/click trace with one cold stencil projection per resource
generation, unchanged hit frames, and no regression in input-to-damage p95.

## Asset Breadcrumb Index Evidence

`layouts/views/asset_browser/selection_text.rs` previously copied references for every folder and,
for each ancestor, scanned that full vector to resolve the next ID. A depth-`D` breadcrumb over `F`
folder records therefore performed `O(F*D)` comparisons after the initial selected-folder scan and
cloned every display-name segment before joining it.

The candidate builds one `HashMap<&str, &AssetFolderSnapshot>` from the folder tree followed by the
visible folder list, preserving the existing tree-first parent lookup. The selected folder still
uses the existing visible-first rule. Parent traversal uses expected constant-time borrowed lookup,
borrows display names and terminates repeated IDs with a borrowed `HashSet`. Source contracts reject
the old per-parent `.find` scan and owned segment clones; a Rust regression preserves the asymmetric
selection/parent precedence. The resulting per-call boundary is expected `O(F+D)` time and `O(F+D)`
borrowed index/path scratch.

This remains transaction-local scratch, not the terminal Asset Browser collection authority. A
future generation-owned catalog should publish the folder-ID index once and let breadcrumb, tree,
selection and pointer paths share it. Product acceptance requires a large-catalog selection profile
with folder lookup visits, allocation bytes and input-to-damage p95 bound to the current binary.

## Asset Reference Layout Index Evidence

`layouts/views/asset_reference_rows.rs` materializes References and Used By rows as four template
nodes each. During every layout, it previously scanned the kind nodes into a
`BTreeMap<usize, f32>` and then performed an ordered lookup for every panel/name/locator/kind node.
For `R` generated reference rows, this added `O(R log R)` index work to resize after the already
expensive `4R` physical-node construction.

Generated row suffixes are contiguous and zero-based after parsing, so the candidate uses a
`Vec<Option<f32>>`. It grows only to a seen index smaller than `nodes.len()`, preventing a malformed
large suffix from causing sparse allocation. Normal rows perform direct indexed lookup. The
`"Unknown"` width is initialized lazily at most once and is not measured at all when every row has
its expected kind node. Source contracts require the linear container and reject the ordered map;
a lower Rust regression covers dense rows and the sparse-suffix bound.

This is an interim algorithm correction, not collection virtualization. The source still deletes
and recreates every dynamic row on data sync and lays out every physical node. The terminal design
must retain logical references separately, materialize only the scroll-visible window plus overscan,
and make resize work proportional to visible rows. Acceptance therefore needs both an index-focused
large-row layout profile and a 10K-reference product test proving physical nodes remain bounded.

## Input Route Diagnostics Migration

`surface/input/pointer.rs` unconditionally builds route diagnostics after pointer dispatch.
`route_policy.rs` creates preview, focus and popup-owned products, while
`route_steps.rs` creates another route-step vector. `input_manager/manager.rs` then treats the
serialized trace as live state and clones its bubble path into the active pointer table. Disabling
trace construction in isolation would therefore corrupt hover/capture behavior.

The first production slice is complete: pointer dispatch transfers its completed route by value to
the trace materializer, which moves the existing bubble and root-target vectors rather than cloning
them. Focus trace construction and the public `UiSurface::focus_path()` frame-publication path now
use the surface-owned arranged-node index. The previous `UiArrangedTree::get` performs
`nodes.iter().find`, so walking a focus route of depth `H` over `N` arranged nodes cost `O(H*N)`;
the indexed path is `O(H log N)` with the existing `BTreeMap` authority. Focused source contracts
are GREEN 5/5 and a lower-layer Rust regression preserves legacy route and missing-node fallback
semantics; scoped rustfmt/diff-check pass, while Cargo execution of the Rust regression remains
pending. This slice still materializes the public diagnostics on every event and does not change
product trace policy.

Pointer hover diff now retains the allocation-free nested membership path only while the explicit
comparison budget is at most 64. Above that threshold it builds one `HashSet<UiNodeId>`, emits
entered nodes in current-stack order, clears and reuses the same allocation, then emits left nodes
in previous-stack order. This bounds dense overlapping-hit work at expected `O(C+P)` without taxing
the normal tiny path. Navigation dispatch applies the same ownership rules already established for
pointer dispatch: borrowed candidates, capacity-sized unordered membership, one reusable public
handler context and no cloned terminal effect. The input tail then moves that route into diagnostics,
cloning bubbled IDs only once because the public trace still owns both bubble and focus paths, and
does not clone the complete Navigation event or root targets. Focused source contracts are GREEN
11/11 and both Rust behavior regressions are staged for later managed execution.

Routed reply-step diagnostics now compute their safe capacity from the existing preview tunnel,
selected bubble/focus path and whether a handled/blocked out-of-route terminal could be appended.
This is an intermediate allocation fix, not a substitute for the terminal diagnostic capture policy:
it bounds Vec growth to one allocation but still materializes every public step on normal input.

The implementation-ready authority split is specified in
`2026-08-25-input-route-diagnostics-materialization-design.md`. It moves the already-built routed
path into an internal execution envelope, updates hover/capture/cross-surface routing from compact
facts, and only then materializes the public serde diagnostic DTO under an explicit capture policy.
Unreal Slate supplies the reference boundary: `FEventRouter` iterates a borrowed `FWidgetPath`,
while debugging is compile-time gated and trace emission is runtime-channel gated.

The terminal migration remains open because it crosses the surface dispatcher, input manager,
dynamic session and public diagnostics contract. Acceptance requires Disabled/Full semantic parity, exact physical
virtual-pointer preservation for projected popup hits, zero route diagnostic allocations across
10,000 normal pointer moves, and a managed current-source Editor p95/allocator/RSS comparison.

## Command Palette Filtered-row Index Evidence

`surface/render/command_palette.rs::command_rows` receives both the complete command metadata and
the already ordered `filtered_commands` projection. It previously resolved each filtered ID with a
fresh linear search through every parsed command. A palette with `C` commands and `F` filtered rows
therefore performed `O(F*C)` string comparisons during each render extraction.

The candidate builds one `HashMap<&str, usize>` over borrowed command IDs and labels, then resolves
the filtered order with expected constant-time lookups. Insertion uses `or_insert` while walking the
commands in source order, preserving the previous rule that the first command wins when an ID and a
label collide. Unknown non-empty IDs still materialize placeholder rows. The source contract was RED
for the nested search and missing first-match index, then GREEN 2/2; nine existing command-palette
contracts remain GREEN. Expected complexity is `O(C+F)` time with `O(C)` borrowed scratch. Managed
render behavior and large-palette allocator/p95 validation remain pending.

## Rich-link Command-range Evidence

`surface/input/rich_link.rs` already knows the final click target, but previously reverse-scanned
`surface.render_extract.list.commands` and rejected every command owned by other nodes. The retained
render cache publishes contiguous command ranges by node and exposes `commands_for_node`; popup
projection already consumes that same authority. Reusing it keeps the reverse-paint-order lookup but
bounds candidates to the target node.

The candidate now queries that range first and retains a full-list fallback for explicitly
unindexed extracts used by lower fixtures. The first existing Rust behavior test constructs the
range through `UiSurfaceRenderCache::update`, while the second still exercises the fallback. The
owned `UiTextLinkHit.href` is moved into the activation effect instead of cloned. A focused source
contract was RED for the full scan and href clone and is GREEN 2/2; scoped rustfmt/diff-check pass.

With `R` total commands, `N` rendered nodes and `C_target` commands for the clicked node, the normal
lookup changes from `O(R)` to `O(log N + C_target)`. The fallback remains `O(R)` by design and must
be counted if it ever occurs on a rebuilt product surface. Managed behavior and click-p95 validation
remain pending.

## Pointer Input Ownership Evidence

`surface/input/pointer.rs` previously cloned `pointer.metadata`, then cloned the complete pointer as
`pointer_for_text`, then cloned the scalar pointer event for the lower dispatcher. Metadata can own
both window and surface ID strings, so the first two clones could duplicate four ID strings per
event. The text and rich-link helpers usually reject ordinary button/move events immediately, but
the ownership cost was paid before those checks.

The candidate borrows the original metadata for cursor state and routing, borrows the original
pointer for the optional text edit, and copies only pointer ID/source/click count. It then moves the
original pointer once into the public result. Rich-link activation accepts click count rather than a
complete input; trace construction accepts pointer source/ID plus the owned route. Ordinary non-text
events now contain zero `pointer.clone()` or metadata clones in this function. The remaining event
clone contains only kind/button/point/scroll/click scalars.

The lower pointer dispatcher also used to clone the complete route once for every routed node that
had a phase or unqualified handler. A dispatch now creates one reusable owned handler context and
changes only its `node_id` and `phase` before invoking handlers. The returned dispatch result keeps
its independent route as required by the public contract, so the retained lower bound is two route
copies per dispatch rather than one copy per handler-bearing node. For route depth `H`, copied route
IDs therefore change from about `O(H^2)` to fixed `O(H)` work. Final input-result construction now
moves `UiDispatchReply` after applied-effect diagnostics are built instead of cloning its effect
vector again, and preallocates the diagnostic effect vector to the exact reply effect count.
Captured, stacked and root candidate routing now traverses an optional injected target
chained with a borrowed slice; it preserves the prior target-first rule and stacked/root order
without allocating or cloning a candidate vector. Cross-candidate visited membership has no ordering
contract, so it now uses a capacity-sized `HashSet` rather than a `BTreeSet`.

The source contract is GREEN 7/7; scoped rustfmt/diff-check pass. Existing text behavior still owns
a cloned event only when it creates a separate text result for merge. Managed behavior and
allocator/p95 validation remain pending.

## Editable-text Pointer Command-range Evidence

`surface/input/text_pointer.rs::text_pointer_hit` already receives the routed target node, but it
previously searched every command in `surface.render_extract.list.commands` before reading that
node's text layout. Text selection drags therefore paid `O(R)` command visits per move even when the
retained render cache had already indexed the target's contiguous command range.

The candidate queries `commands_for_node` and searches only the target slice in the original forward
order. A full-list fallback remains for deliberately unindexed lower fixtures; production profiling
must count fallback use because a fallback on a rebuilt surface would restore the old complexity.
The focused source contract was RED on the full scan and is GREEN 1/1 after the change. The indexed
path is `O(log N + C_target)` for `N` rendered nodes and `C_target` target commands. Managed text
selection behavior, allocator and pointer-drag p95 validation remain pending.

## Popup ID Lookup Evidence

`surface/popup_stack.rs::unique_popup_state_for_id` must reject ambiguous IDs, so it currently walks
eligible popup metadata until a second match is found. The old predicate called
`popup_stack_id_for_node` for each candidate, cloning every non-empty node path or formatting
`node:<id>` into a fresh `String`, then immediately discarded it after comparison.

The candidate compares non-empty paths by borrowed `&str` and validates the numeric fallback without
formatting. It preserves the exact canonical-string contract: `node:7` matches node 7, while
`node:07`, `node:+7` and non-digits do not. A Rust unit regression records those cases. The source
contract was RED for the allocating predicate/missing helper and is GREEN 2/2. This removes `O(P)`
temporary strings but deliberately leaves the uniqueness traversal at `O(N)`. The terminal design
is a popup ID index rebuilt with the tree/control-index generation and storing ambiguity explicitly.

Tree-to-stack seeding previously compared every retained stack record with every authored popup and
then repeated the inverse scan for each open authored popup. It now builds a borrowed node-to-
`(popup_id, owner)` map, preserves stack order through `retain`, drops that borrowed map before any
surface mutation, and uses a node set to add only missing open records. ID and owner equality remain
mandatory before a stack record enters the node set. Expected reconciliation complexity changes
from `O(P*S)` to `O(P+S)`.

The render full-extract and stack-reconciliation predicates also duplicated their common popup
trigger/owner traversal. They now return one `UiPopupDependencyImpact`. The analysis scans changed
nodes once to collect borrowed control IDs, missing-node state and direct popup facts; each retained
control popup then checks its popup and owner ancestor chains once. This removes the nested
changed-node-by-owner walk and changes the common upper bound from about `O(P*C*H)` to
`O(C + P*H)`. The layout rebuild branch consumes both flags from one result; the non-layout branch
reuses it only when render-patch and final dirty sets are the same. A Rust regression preserves the
independent domain truth table for missing, closed-popup, open-control-popup and ordinary nodes.

## Current Structural Inventory

`E:\zircon-profiles\ui-structural-audit-20260825-133934` is bound to HEAD
`1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8` plus 1,216 dirty UI paths. It inventories 4,740
production Rust files and 466,813 lines, with 5,734 clone calls, 2,570 vector-materialization
signals, 132 sort calls, 7,624 string-allocation signals and 2,944 traversal signals. The JSON
SHA-256 is `9F5590749932311FC202ABD72CE4C09914ADD6FB8A9BBE11D9F08D1EB870FA53`; the CSV SHA-256 is
`F19402D4BDD70E496A34FCA8288947F7D1DF636B03EE370FFDD9D456396BBBBC`.

Compared with the prior same-HEAD snapshot, the current UI tree has 925 more production lines, four
fewer clone signals, one more vector-materialization signal and seven more traversal signals. The
top 29 hotspot ranks are unchanged: `pane_payload_projection.rs` remains first, the Asset Browser
assembly remains fourth and workbench pane projection remains twelfth. This stability supports the
existing structural priority, but the inventory remains heuristic source evidence, not CPU or
latency evidence.
The highest-risk file, `pane_payload_projection.rs` (616 points), is externally modified and was
therefore reviewed but not edited. The next clean, event-relevant candidate was `arranged.rs` (88
points), which led to the input patch above.

## Historical Priority Evidence

The following E-drive artifacts predate current HEAD and fail the current source-binding/sample-
integrity gate. They are priority evidence only, not acceptance:

| Historical session | Relevant observation |
| --- | --- |
| `runtime09-button-hover-click-20260809-124940` | 8 click counter frames had 10.018 ms p95, 2 slow paths, and 4 presentation rebuilds. The enclosing capture had 5/5 over-budget frames and one 2,666.78 ms `recompute_if_dirty` span. |
| `20260810-180518-window-resize` | 2/2 frames were over budget; the captured recompute was 3,056.67 ms, presentation was 1,673.89 ms, host-scene build was 1,634.88 ms, and pointer bridges were 1,221.80 ms. |
| `20260811-201002-click-dock-patch-spaced` | The click scenario still recorded 12 presentation rebuilds, 6 slow paths, 75,520,896 retained-cache copy bytes, and damage-to-submit p95 of 2,412,531.5 us. |

These results support keeping presentation publication, host-scene conversion, pointer bridges, and
resize geometry-only publication above matcher micro-optimization in the P0 order.

## Validation

- Focused source contracts: 75/75 pass for editor/runtime table, text, menu/command, pseudo-state,
  arranged input-patch, rich-table sizing, tree-selection, geometry-command and Performance Timeline
  clip-materialization, virtual-window static-key, menu pointer stencil-cache and Asset Browser
  breadcrumb/reference-layout changes, pointer-route trace ownership transfer, command-palette
  filtered-row indexing, rich-link/editable-text command-range lookup, borrowed pointer-input and
  fixed-cost handler-route/reply ownership, allocation-free popup ID comparison, indexed popup
  reconciliation, fused popup dependency analysis, indexed surface focus-path publication, hybrid
  hover membership, fixed-cost navigation dispatch/input ownership, and bounded routed-step
  allocation.
- Full `test_*performance_contract.py` discovery: 553 tests, 551 pass, 2 foreign-owner failures,
  and 0 errors. The remaining failures are the existing Asset Content borrowed-identity assertion
  and bounded-damage counter file-split assertion; neither references this UI candidate set.
- Scoped `rustfmt --check` and `git diff --check` pass for the retained candidates.
- The seven delegated Runtime/Runtime Interface ownership errors are absent from current source:
  pointer routing borrows its route, navigation copies scalar route facts before consuming it, IME
  input matches the normalized event directly, and no IME pattern invokes `Vec::new()`. Source
  guards are 7/7 and scoped rustfmt passes; this is not a managed compile result.
- Rust behavior tests are present for ASCII/Unicode/empty search, text cache ownership/eviction,
  paint behavior, allocation-preserving parent input patches, rich-table lower bounds and borrowed
  tree ID ordering/disabled membership, atomic geometry-command behavior, Asset Browser breadcrumb
  precedence and dense/bounded reference-width indexing, but have not run under managed Cargo in
  this slice.
- No current-source `zircon_editor.exe` exists in the approved D-drive target pool, so no valid
  current-source CPU, RSS, allocator, input-to-damage, damage-to-submit, SVG parse/raster/upload, or
  resize p95 result is available.
- The current profile manifest cannot yet prove exact dirty-source provenance and has no general
  allocator collector. The evidence and schema-v3 repair are recorded in
  `2026-08-25-profile-source-binding-and-allocation-evidence-audit.md`; product captures remain
  provisional until those gates bind a managed build closure to the launched binary.

## Next Acceptance Gate

After managed validation is authorized and a current-source editor binary exists, run three repeated
source-bound profiles for idle hover, button click, 200-step native resize, 10K menu options, and
visible SVG resize. Require complete PID-bound samples and report at least:

- input-to-damage and damage-to-submit p50/p95/p99/max;
- CPU and quiescent/peak RSS;
- presentation, host-scene, pane-projection, pointer-bridge, and structure-generation counts;
- layout visited nodes and explicit responsive fallback reasons;
- visual pixel/SVG tree hits and misses, candidate builds, async queue age, GPU image-source visits,
  texture writes, upload bytes, and resident bytes;
- allocator deltas for menu input, text layout hits, and clipped table paint.
- menu-pointer stencil projection builds, requiring one cold build per resource generation and zero
  rebuilds caused only by shell size or pointer-state changes.
- Asset Browser folder-index builds/lookups and breadcrumb allocation bytes under a large catalog.

Ordinary pointer move must request no present. Stable button hover must perform no presentation or
structure rebuild. Ordinary resize must preserve structure generation and pane model identity, with
responsive breakpoint fallback measured separately. Only those results can advance this slice from
`static_candidate` to a validated integration milestone.
