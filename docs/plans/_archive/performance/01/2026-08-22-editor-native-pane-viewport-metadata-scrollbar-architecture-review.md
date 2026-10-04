---
title: Editor native-pane viewport, metadata and scrollbar performance review
date: 2026-08-22
module: zircon_editor retained-host paint_workbench_renderer native_panes
priority: MVP-P0 hierarchy, asset, viewport and diagnostics pane paint
status: source_reviewed_m2_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Scene Outliner and Content Browser retained item sources and Slate virtualization
---

# Goal

Make one pane source/layout generation own typed viewport, row, content-extent and scrollbar metadata.
Hierarchy and asset paint must not rediscover anchors/counts by scanning template control ids. Visible
rows, interaction overlays, viewport images and scrollbars must consume the same indexed generation
and retained command ranges, with exact damage routing before content preparation.

## Reviewed source

- Rust files: 20/20
- lines: 2,020
- bytes: 63,706
- joined path-and-raw-source-bytes SHA256:
  `91dde901f6ac9840634a0dfaadfc38478161db0769b90f7b6c87a5667797f9a3`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

Scope: `paint_workbench_renderer/native_panes.rs` and
`paint_workbench_renderer/native_panes/**`, including all production and inline/test files.

Supporting owners traced/read where behavior-defining: hierarchy pointer metrics, asset pointer
extents, `AssetContentPaintMetadata`, pane content fan-out, template projection metadata and viewport
image primitive paths.

## Correct foundations to retain

1. Native content selects one pane-kind branch; inactive pane backends return without work.
2. Hierarchy rows already compute and visit a bounded strict visible range O(V), including damage clip.
3. Asset content/reference viewports and extents use typed `AssetContentPaintMetadata` without model
   scans; tests explicitly protect this path.
4. Scrollbar geometry is O(1), clamps invalid/fitting content and derives a proportional bounded thumb.
5. CPU/GPU viewport resources are selected without copying the resource key or image owner here.
6. Diagnostics overlays use an iterator rather than first materializing another primitive Vec.

## Structural findings

### P0: hierarchy anchor and metrics are rediscovered by sibling layers

Hierarchy row paint scans `pane.hierarchy.nodes` to find the list anchor. Immediately afterward its
scrollbar calls the same viewport scan again. Row paint obtains hierarchy metrics once to derive the
visible range, but each visible row geometry calls `current_hierarchy_row_metrics` again; scrollbar
also reads it for content extent.

For `T` template nodes and `V` visible hierarchy rows, this was two O(T) anchor scans and `V + 2`
metrics reads per hierarchy paint. M1 made the content owner compute viewport and metrics once and
pass them to rows, row geometry and scrollbar. The indexed-anchor follow-up now publishes the small
candidate row set at generation time, eliminating the remaining paint scan.

### P0: asset-tree row count and hover address still scan the complete template model

Activity/browser content and reference lists use metadata, but `asset_tree_row_count` filters every
template node by control-id suffix each paint. Activity hover repeats a linear scan until the kth
matching row and parses the leaf id for each candidate. An Assets pane can therefore perform a full
count scan plus a second hover-address scan even though its model already carries paint metadata.

M2 extends `AssetContentPaintMetadata` with activity/browser tree viewport, row count and indexed row
addresses/node groups. Source generation builds the index once; scrollbar count is O(1), while hover
is O(1) for contiguous models and O(log T) for persistent row overlays, with no control-id parsing in
paint. Browser's existing source-tree groups are the foundation, not a second painter cache.

### P0: pane scrollbar selection repeats independent metadata/damage work

Assets can call tree, content, References and Used By scrollbar functions in one paint. Typed metadata
lookups are cheap, but each route separately selects viewport/extent, computes style/geometry and only
clips at primitive emission. Damage intersecting one subviewport still visits all scrollbar routes.

M3 compiles the pane generation's scrollbar descriptors and damage bounds. Paint selects only
intersecting descriptors and reuses retained track geometry; scroll changes patch thumb instance data.

M3a now publishes the bounded typed descriptor set in `AssetContentPaintMetadata`, performs one
metadata read per asset pane paint, intersects effective frame damage before native kind work and
rejects each disjoint descriptor before interaction/extent/style/geometry preparation. It deliberately
   does not cache theme-dependent pixel geometry in workbench metadata. Current-source reassessment
   found that Runtime `UiSurface` already owns per-node render command ranges. M3b therefore maps
   scrollbar track/thumb roles into that Runtime authority; it must not add an Editor-owned payload
   cache. Track commands can be retained and thumb instances patched only after the Runtime range owner
   supplies one layout/style generation and explicit invalidation boundary.

### P0: hierarchy row text and interaction remain immediate artifacts

Visible-row virtualization bounds work, but every visible stable row rebuilds surface, border and text
bars. Selection/hover/inline rename affect one addressed row. M3 retains row command/text artifacts by
source/layout/style generation and patches only changed row state.

### P0: viewport image presentation remains tied to immediate pane paint

Scene/Game image selection dispatches CPU RGBA or GPU resource drawing from the pane paint path. The
separate render-command review owns removal of CPU image resampling/copy and convergence on shared
prepared image draw instances. This module must supply exact viewport/resource generation and damage,
not add another image cache.

### P1: diagnostics and kind routing are string/immediate paths

Diagnostics walks all overlay primitives when its pane is painted, and content routing matches string
pane kinds. M4 compiles typed backend kind and retains overlay command ranges by diagnostics
generation. String identifiers remain serialization/protocol data, not hot paint dispatch.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SSceneOutliner.cpp`
- `dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`

Scene Outliner retains root tree items, binds them as `TreeItemsSource`, generates rows through
`OnGenerateRow` and uses explicit full/column/selection refresh flags. Content Browser retains item
collections, supports recycling/incremental data updates and maintains a lookup; its large lookup
refresh uses `ParallelFor`. Slate list views generate rows from a retained item source, while widget
proxies repaint from explicit update flags.

The transferable constraints are typed retained sources, source-owned indices, visible-row
generation, incremental refresh domains and no control-id model scan in paint. Zircon should not copy
Unreal pointer lifetimes or background-thread APIs blindly; current-source captures remain mandatory.

## Target architecture

1. Each pane generation publishes typed backend kind plus hierarchy/asset viewport, row groups/counts,
   content extents, resource generations and damage bounds.
2. Hierarchy and asset visible ranges use indexed source metadata; hover/selection/rename address exact
   stable row ids.
3. Scrollbar descriptors and retained track geometry are built once per layout/style/content
   generation; scroll updates compact thumb state only.
4. Visible row text/commands and diagnostics overlay ranges are retained and patched by granular state.
5. Viewport image draw instances share Runtime/editor prepared resources and carry resource/damage
   generations without CPU ownership conversion.
6. Typed pane plans replace string paint routing and feed one prepared render-list authority.

## Instrumentation and acceptance

Matrix: hierarchy/asset nodes `0/1/1k/10k/100k`, visible rows `1/16/64`, asset subviews
`tree/content/references/used-by`, hover `none/stable/move`, scroll `stable/change`, damage
`outside/one subview/full`, CPU/GPU viewport, diagnostics primitives `0/1k/100k`.

| Evidence | Acceptance |
| --- | --- |
| template anchor/count/control-id scans | zero in accepted paint path |
| metrics/source metadata reads | one generation-qualified view per pane paint |
| logical/visible/visited rows and overlays | work proportional to visible/changed rows |
| scrollbar descriptor/track/thumb rebuilds | only intersecting; thumb-only on scroll |
| row/text/overlay command rebuild/reuse bytes | proportional to dirty rows/ranges |
| CPU/allocation/RSS/latency/context switches/power | same executable/workload before and after |
| RenderDoc draw/batch/GPU and pixel/text parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add anchor/count/id scan, metadata, row, scrollbar, text/overlay/range counters; capture. | attributable baseline |
| M1 | Compute hierarchy viewport/metrics once and share with rows/geometry/scrollbar. | scans `2 -> 1`; reads `V+2 -> 1` |
| M2 | Extend source-owned asset metadata with exact tree counts/frames/groups for both surfaces. | zero asset paint scans/id parsing |
| M3 | Retain visible row/text and scrollbar descriptors/ranges with granular damage patches. | rebuild proportional to dirty rows/subviews |
| M4 | Compile typed pane backend and retained diagnostics/viewport resource ranges. | no string dispatch/full overlay rebuild |
| M5 | Converge all native pane ranges with shared prepared render list and hard-cut duplicates. | one presentation authority |
| M6 | Run managed scale/input/WPR/power and RenderDoc/pixel/text parity matrix. | quantified accepted milestone |

## M1 implementation result

The Hierarchy branch in `content.rs` now computes one viewport and one `HierarchyRowMetrics` value.
Both row paint and scrollbar consume those values. The visible-range function and every visible row
frame use the same metrics snapshot; neither child re-enters global metric selection or scans template
anchors. Metrics are a `Copy` structure of five `f32` values, so the change adds no heap/shared owner.

For `T` hierarchy template nodes and `V` visible scene rows:

| Static paint work | Before | After | Change |
| --- | ---: | ---: | ---: |
| template anchor scans | 2 x O(T) | 1 x O(T) | -50% scans |
| hierarchy metric snapshots | `V + 2` | `1` | visible-row independent |
| visible scene-node row reads | `V` | `V` | preserved O(V) foundation |
| scrollbar geometry evaluations | 1 | 1 | behavior preserved |

These are source-path counts, not elapsed-time claims. The indexed-anchor follow-up below eliminates
the remaining hierarchy scan; M2 eliminates the asset-tree count/hover scans with source-owned metadata.

## M1 indexed-anchor follow-up

Both hierarchy model construction paths now attach `HierarchyPaintMetadata`, containing source-order
row addresses for `HierarchyListPanel` and `HierarchyTreeSlotAnchor`. Paint visits only these
candidates, resolves their live row geometry and selects the first visible frame. This preserves the
old hidden-first-candidate behavior without scanning or parsing the complete template model.
Generation publishes at most the first row for each allowed identity, so malformed duplicate anchors
cannot expand A beyond two or reintroduce an O(T) paint path.

Stable row addresses deliberately avoid cached-rect invalidation under metadata-preserving geometry
patches. A contiguous model costs O(A); a persistent row overlay costs worst-case O(A log T), where A
is the two allowed anchor identities, rather than O(T).

Default deterministic pressure model: T=10,000, A=2, 2,000 paints, one generation and a conservative
15 trie node visits per candidate. Its legacy side is explicitly the worst case where the first visible
anchor is last or absent; work changes from 20,000,000 template visits to 76,000
generation/index/trie units, a 263.1579x ratio. This is not typical-path, elapsed-time or resource evidence.

Artifact: `E:\zircon-profiles\editor-hierarchy-anchor-index-20260828.json`

SHA-256: `18B70F6F24C879FF7820EB69C03AB51CC4F57371A4C2AC93224BB5DB831B6788`

## M2 implementation result

`AssetContentPaintMetadata` now publishes Activity tree row addresses in source order and exposes one
surface-aware tree row count. Browser uses its existing source-tree logical groups; no second painter
cache was added. Activity tree rows remain in `fixed_node_rows`, preserving current paint selection
while scrollbar count becomes O(1) and hover lookup becomes O(1) for a contiguous model or O(log T)
for a persistent row overlay.

The row address deliberately resolves the live model row frame at query time. Same-generation
composition can patch row geometry while preserving metadata, so caching the rect itself would be a
stale-geometry bug after incremental resize/layout updates.

The Activity/Browser scrollbar paths no longer carry row-control strings or filter the complete
template model. Activity hover no longer scans to the kth matching control id. Control-id parsing is
confined to the metadata generation classifier.

Default deterministic structural model: 10,000 template nodes, 2,000 paints per surface, 1,000
Activity hover paints and one generation. Modeled node/query work changes from 50,000,000 to 30,000
units, including a conservative 15 trie node visits per overlay row lookup, a 1666.6667x ratio. This is not
CPU, allocation, memory, latency or GPU evidence. Full evidence,
limits and dynamic gates are recorded in
`docs/plans/optimize/zircon_editor/01/2026-08-28-asset-tree-generation-metadata.md`.

## M3a implementation result

Asset metadata now publishes at most four style-independent scrollbar descriptors: tree, content,
References and Used By. The native asset painter reads metadata once and evaluates the bounded set.
Pane clip is intersected with frame damage before native pane dispatch. Pane layers report O(1) logical
content presence rather than whether the current damage happened to receive pixels, so partial paint cannot
spuriously enter fallback while a genuinely empty full frame still renders `No actors` / `No assets`.
Each descriptor is rejected by viewport/damage before dynamic interaction, extent conversion, theme reads
or geometry calculation.
The typed-kind index uses inline capacity four and resolves viewport/extent from the existing metadata
fields and row groups, so it adds neither a heap allocation nor a copied geometry/count authority.

The default deterministic single-subview-damage model uses 4,000 pane paints, four descriptors and two
metadata generations. Metadata lookups, style reads and geometry evaluations change from 16,000 to
4,000 each, a 4x structural ratio. Full damage intentionally retains all four bounded evaluations. This
is not elapsed-time, allocation, memory, latency, power or GPU evidence.

Artifact: `E:\zircon-profiles\editor-native-pane-scrollbar-damage-20260828.json`

SHA-256: `6E5412A867CB26BC06611A503CCFF77C28C4BF99B41424E01B086A940D93C9C6`

Historical post-M1 direct-owner snapshot (retained for provenance; M2/M3a later changed some paths,
so these are not current-candidate fingerprints):

- Rust files: 20/20
- lines: 2,035
- bytes: 64,003
- joined path-and-raw-source-bytes SHA256:
  `b0dfc673b87cb9777c00c5019ab14b1108cee200ededdf62fcb83c350c68d225`
- unchanged direct owner files: 15 retain their pre-M1 fingerprints inside the joined hash above

| Changed direct owner file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `native_panes/content.rs` | 89 | 3,004 | `f559f6bef142c2f82b8ada1508bc9c418b4bb5974322336752e51bdec95e221e` |
| `native_panes/hierarchy.rs` | 180 | 5,484 | `0b7fbae4b3fc9247fd659aeeba5df00d684b35d9d6556f02132d5fe484bec21a` |
| `native_panes/hierarchy/row.rs` | 42 | 1,344 | `fc01980663ad3ec34791d40c3a6975b50d947ae63b8ca36e5c13cf191af92f65` |
| `native_panes/hierarchy/row/frame.rs` | 19 | 559 | `dc0181f1de5d813b46e9b237acb6beb8eb53913d25cbd8901b83bf3c965ca180` |
| `native_panes/scrollbar.rs` | 266 | 7,708 | `02e61baa3ca524aba0693c3ef662e226a6cda2e9398fb1e5cfded5cfe8ac5bbb` |

Focused static contract:
`tools/tests/test_editor_native_pane_shared_hierarchy_view_performance_contract.py`, 58 lines,
2,406 bytes, SHA256
`a5b797acd3a9774415c0be4dd9c4bbc4f11349b2961877512e6169a1a5ddf1ba`.

## Validation state

- Full owner review: passed, 20/20 Rust files.
- Asset metadata and pointer/extent owners: traced; relevant current production paths read.
- Relevant Unreal sources above: read and mapped to retained/indexed source constraints.
- M1 focused contract: RED 4/4 before the change, GREEN 4/4 after the change.
- M1 indexed-anchor contract: RED 3/3 before the change, GREEN 3/3 after the change; combined current
  M1 viewport/metrics/anchor pressure contracts GREEN 11/11.
- Current owned editor performance-contract set: GREEN 69/69.
- Current focused M1-M3a combined Python contract set: GREEN 24/24; the M3a source contract was 4/4 RED
  before production migration, then its fallback/presence extension was 2/5 RED before repair and is now
  5/5 GREEN.
- M3a exact nine-file Rust `rustfmt --check` and scoped candidate `git diff --check`: passed.
- Independent M3a review found and drove repair of one empty-state/fallback Important; lower tests now cover
  non-empty disjoint damage plus empty full/disjoint paint. Its pressure-input-boundary Minor is also fixed.
  Final incremental re-review found no Critical/Important/Minor.
- Existing hierarchy visible-range, rename, row geometry and scrollbar Rust behavior tests remain
  present, but are not claimed passing until managed Cargo is executable.
- M0, M3b.3-M3b.5 and M4-M6 remain pending. M3b.1 removes one redundant post-compaction
  command probe and M3b.2 carries the coherent presentation cursor through both presenter backends;
  neither claims retained-range convergence. M2/M3a dynamic acceptance remains
  pending, and no elapsed-time, GPU or power claim is made from static counts.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived` and rejects Cargo
  launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a launchable current-source editor. RenderDoc cannot validate
  template scans, metrics reads, CPU text preparation or source-index reuse.

The module remains in `pending.md` until M0-M6 pass on one source/executable/workload fingerprint.
