---
title: Editor asset content virtualized item source performance review
date: 2026-08-22
module: zircon_editor/src/ui/workbench/asset_content_layout
priority: MVP-P0
status: source_reviewed_static_pass_dynamic_pending
reference_engine: Unreal Engine Content Browser and Slate virtualized table views
---

# Goal

Replace full-catalog template-node materialization and control-ID-driven layout with one stable typed
asset item source. Only visible list/tree/reference rows or thumbnail cards may be materialized for
paint and hit testing. Filtering and refresh work must be coalesced and time-sliced; projection,
layout, paint, scrollbar and pointer routing must consume the same generation artifact.

## Reviewed source

- folder: `zircon_editor/src/ui/workbench/asset_content_layout`
- Rust files: 8/8
- lines: 1,638
- bytes: 55,538
- joined UTF-8 SHA256: `e2cc45fbf3e4c381bc31422a380bdf03453ec394b264839a0c8555e46a898667`
- owning commit before review: `08094b9b9e17f6c80372e15c17b01204038b305b`
- local Rust tests identified: 12

| File | Lines | SHA256 |
| --- | ---: | --- |
| `controls.rs` | 108 | `1c9efd0c2e10ce139e1516e9fe310b387c664d61bf546e22860d12f17c790af4` |
| `metrics.rs` | 100 | `fd9dc14134df92add9ca534880d6483cef91b7d03961eb492c40abc3861869db` |
| `mod.rs` | 34 | `76df720c8cb829915cd18f0ab80b0dfb91dd9c748563cab279ac0e62c6e9532f` |
| `paint_metadata.rs` | 817 | `005731ccc1cdbea5a3b4be506db35dcece759d1774663356f052593f2120d0c9` |
| `profile.rs` | 15 | `85a90fc969e3ad428f12062f54280f450eac99060f3e6cadc6b9b2e46b6bead6` |
| `tests.rs` | 272 | `86d3319155bdb5e293b5d51425b0d8374e78c68285bcb043143385cbd45bb888` |
| `text.rs` | 127 | `46e0efa86e767022139e5faa49c12a9f41e1ab6e22167ea43af9590f4523160d` |
| `thumbnail_grid.rs` | 165 | `3bab34897f23052a03e089b320718786e4d265a13356828b2c25378ab173a4c0` |

All eight files were read in full. The review also traced the current production owner chain through
`assets_activity.rs`, its content node/layout builders, `asset_browser.rs`, table/thumbnail/source
tree/reference builders, composition caching, template-node draw/transform, native pointer routing
and scrollbars. The existing Editor09 asset projector failure record was read in full.

## Result

### Correct foundations to retain

`AssetThumbnailGridMetrics::item_index_at_point` and `content_extent` are allocation-free O(1)
geometry. Filename compaction uses binary search over prefix length, not a linear prefix trial loop.
Paint metadata is attached to a cached `ModelRc` composition generation, and visible group selection
uses `partition_point`, so stable paint no longer scans every asset node. The 10k metadata fixture
correctly proves that only intersecting groups are returned.

These are useful local mechanisms, but the 10k fixture starts after full node generation and layout.
It therefore does not cover the dominant current cost.

### P0: full node materialization followed by quadratic layout

Assets Activity appends five owned `ViewTemplateNodeData` values per visible folder or asset. Its
layout loops over every item and, per row, performs one full-vector lookup for meta width and five
full-vector frame lookups; assets perform a seventh lookup for filename compaction. With `A` assets,
the model has about `5A` dynamic nodes and layout performs `7A` linear searches: Theta(A squared).
At 10k assets the static upper bound is 3.5 billion node/control-ID comparisons before paint.

Browser thumbnail mode appends eight nodes per asset plus an optional selection marker. Card count
repeatedly searches the whole vector for successive IDs. Each card then performs about twelve more
whole-vector lookups for text, frame writes and compaction. This is also Theta(A squared); a 10k
catalog produces roughly 80k dynamic DTOs and a static upper bound above ten billion ID comparisons.
Preview image loading is requested while all thumbnail nodes are materialized, even though only a
small viewport can be visible.

Browser list mode creates all rows, then uses per-asset `.any` during sync and per-asset `.find`
during selection and cell application. Those three phases remain Theta(A squared). Paint
virtualization cannot recover main-thread time and memory already spent on this projection.

### P0: generation key and geometry lifetime are inconsistent

The Browser has an outer cache keyed by snapshot identity, exact viewport bits, resource generation
and text metrics generation. Assets Activity instead keys composition with
`AssetWorkspaceProjectionGeneration::from_snapshot`, which does not include viewport size. A same
snapshot resize can patch base projection rows while preserving the old metadata and dynamically
appended content geometry. Adding size to the key alone would restore correctness but expose the
quadratic rebuild on every resize event. Correct invalidation and virtualized materialization must
land in the same milestone.

### P1: the remaining paint path is visible-set bounded but still transient

`paint_metadata.rs` builds five `BTreeMap` indices, performs seven later linear `.find` passes and six
group sorts per invalidated generation. It owns a `BTreeMap<String, AssetContentNodeIdentity>`, so
recognized IDs are cloned. All three visible-row APIs clone `fixed_node_rows`, append visible groups
and sort a fresh vector on every pane draw.

The Activity projector parses reference control IDs for each visited node, then performs identity
lookup; scrolling content performs the identity lookup again through `is_scroll_node`. Browser nodes
may successively parse source-tree and reference IDs before the identity map lookup. This cost scales
with visible nodes rather than total assets, but it is still avoidable hot-path string work.

### P2: duplicate text policy

Assets Activity uses `asset_content_layout/text.rs`; Asset Browser carries a second filename
compaction implementation. Both perform glyph-width-aware binary search, but their invalid-width
semantics and internal allocation shapes differ. One Runtime Text-owned compaction/cache policy
should replace both only after virtual row materialization establishes the correct generation key.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Views/STableViewBase.cpp`

`SAssetView::RequestSlowFullListRefresh` and `RequestQuickFrontendListRefresh` only set pending flags
(`2132-2140`). `Tick` coalesces those requests, separates source refresh from frontend filtering and
amortizes pending filter tasks; its configured `MaxSecondsPerFrame` is 0.015 seconds. List and tile
views consume `FilteredAssetItems` as a stable item source and `RefreshList` requests layout refresh
instead of eagerly building every row (`3044-3073`, `5279-5287`).

Slate `SListView::ReGenerateItems` begins at the scroll-derived start index and generates until the
available view is filled (`1524-1660`). `FWidgetGenerator` maps typed items to current widgets,
reuses visible widgets and releases widgets not seen in the generation pass (`978-1080`,
`1664-1695`, `3308-3325`). `STableViewBase::Tick` regenerates only when items or panel geometry need
refresh (`424-524`). The transferable standard is stable typed item data, explicit invalidation,
visible widget generation and reuse. It is not UE's exact widget classes or pixel values.

## Target architecture

1. Publish a stable `AssetViewGeneration` from the asset workspace/catalog owner. It contains typed
   folder, asset and reference item sources, filter/sort generation, selection identity and preview
   readiness; it does not contain one generic template DTO per catalog item.
2. Coalesce catalog/search/filter/selection invalidations. Expensive filtering is cancellable and
   time-sliced with an explicit per-frame budget no larger than UE's 15 ms ceiling; current-source
   measurement must set the lower Zircon budget.
3. Give list/tree/reference/thumbnail surfaces one typed virtual layout each. Row/card extent and
   visible range are derived directly from item count, viewport, scroll and density. Materialize only
   visible items plus a small measured overscan.
4. Emit a `VisibleAssetNodeBatch` with typed route metadata and stable item identity. Paint,
   scrollbar, hover and pointer consume borrowed row/card slots. No consumer reparses control IDs,
   rebuilds a string map, clones fixed rows or sorts a visible index vector.
5. Cache preview decode/upload by asset revision and requested thumbnail class. Only visible plus
   bounded prefetch items may request thumbnails; stale generation work must be cancellable.
6. Include viewport/layout tier, Runtime Text metrics, density/style and item-source generation in
   the virtual layout key. Resize updates visible geometry without rebuilding the catalog item
   source.
7. Delete full-catalog dynamic template-node builders and their control-ID parsers after all
   consumers use typed visible batches. Do not keep a compatibility path.

Complexity target:

- item source/filter invalidation: O(A), time-sliced or worker-produced where thread-safe;
- viewport/scroll update: O(log A + V), where `V` is visible plus bounded overscan;
- node materialization, layout, paint and hit test: O(V), independent of total catalog size;
- stable frame with no damage: zero asset projection/layout/thumbnail work;
- retained UI memory: O(A) compact item records plus O(V) presentation nodes, not O(A) large DTOs.

## Instrumentation and acceptance

Add attributable spans/counters before the hard cutover:

| Evidence | Required result |
| --- | --- |
| item source/filter visits and slice duration | O(A), no main-thread slice above 15 ms |
| dynamic DTOs/materialized cards | O(V), not O(A) |
| control-ID comparisons/parses in layout/paint/hit | 0 after cutover |
| visible-index allocations/sorts per paint | 0 after cutover |
| preview requests/decodes/uploads | visible plus bounded prefetch only |
| stable redraw projection/layout builds | 0 |
| input-to-damage and damage-to-submit p95 | at most repository gates of 1 ms and 8 ms |

Matrix: assets `1/100/1k/10k/100k`, folders `1/100/10k`, list/thumbnail, references `0/100/10k`,
viewport `320/640/900/1260/1920`, scale `1/1.25/1.5/2`, idle/scroll/resize/search/filter/selection/
refresh and cold/warm preview cache. Record median/p95/p99 main-thread time, comparisons, visits,
allocations/bytes, materialized nodes, decoded bytes, RSS, frame pacing and package energy before and
after on the same machine and content fingerprint.

Use the current UI profiler plus WPR/ETW CPU sampling and allocation evidence, with all artifacts and
targets on D/E/F. RenderDoc is deferred until a launchable current-source GPU UI path exists and is
used only for thumbnail draw/upload and pixel parity; it cannot prove CPU projection complexity or
Softbuffer painter allocation removal.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add counters and capture 1/100/1k/10k current-source baselines. | WPR, allocation and UI trace artifacts |
| M1 | Introduce stable typed item sources and coalesced/cancellable refresh. | O(A) source build; bounded slices |
| M2 | Implement list/tree/reference/thumbnail virtual layouts and visible batches. | O(log A + V) scale tests |
| M3 | Move paint, pointer, scrollbar and preview requests to typed visible slots. | zero string parse/map/sort hot path |
| M4 | Delete full-catalog DTO/control-ID layout paths and duplicate text policy. | exact owner/deletion contracts |
| M5 | Run real-window interaction, WPR, power and GPU/pixel parity where applicable. | quantified before/after evidence |

## Validation state

- Full owner-folder review: passed, 8/8 files.
- Related generation/layout/painter/pointer call chain: reviewed for the stated algorithms.
- Static Editor09 and pointer contracts: 7/7 passed after updating one stale ownership assertion to
  the current composition-metadata contract.
- Production source edit: none; a local metadata micro-optimization would preserve the dominant
  Theta(A squared) model and is therefore rejected.
- Managed Cargo, current-source real-window profiling and power evidence: pending.

The folder remains in `pending.md`. It must not enter `review.md` until M0-M5 pass on one source and
asset fingerprint and the existing Editor09 failure record is returned as fixed.
