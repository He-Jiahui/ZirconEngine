---
title: Editor surface hit and paint generation index performance review
date: 2026-08-23
module: zircon_editor retained-host surface_hit_test
priority: MVP-P0 editor input latency and damage-bounded paint
status: source_reviewed_m0_single_pass_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate persistent hit-test grid and invalidation fast path
---

# Goal

Preserve the current generation-owned point-hit index while separating hit routing from paint
invalidation/candidate ownership. Stable pointer input must remain cell-bounded, and damage-bounded
paint must not allocate, deduplicate and sort candidates on every multi-cell clip or rebuild every
pane model when only one owner changes.

## Reviewed source

- owner Rust files: 18/18
- lines: 2,653
- bytes: 92,829
- source-only SHA256 over lexicographically sorted owner files:
  `8e3f2268495ff189b99c05fd2867db6757adf624bd97dd609506331d8e509b5a`
- post-M0/M0b owner files/lines/bytes/SHA256: 18 / 2,644 / 92,575 /
  `d95d74fc1566b754eac0b086c489a5f3e05458b02bd1529afd72779b9f3db827`
- owning commit at review: `0d70d1ac6499abcf56c3f6c3ef43cb3a7502a249`

| Owner group | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `surface_hit_test/*.rs` | 4/4 | 1,012 | 35,837 |
| `surface_hit_test/template_node/*.rs` | 7/7 | 1,285 | 44,918 |
| `surface_hit_test/template_node/popup_rows/**` | 4/4 | 201 | 7,010 |
| `surface_hit_test/template_node/surface_frame_builder/**` | 3/3 | 155 | 5,064 |

All owner files were read in full. Direct consumers were inspected through host state replacement and
patching, presentation paint scopes, pane conversion/scoped patches, native pointer routing and
template-node painting. The cited Unreal hit-grid and invalidation-root sources were read directly.
Supporting files are not counted as owner coverage.

The 2026-07-17 surface-hit report covered 16 files and 923 lines. This current record supersedes that
owner coverage: the current tree has added persistent workbench hit/paint indexes, generation
rebinding and stronger scale tests.

## Correct foundations to retain

1. `HostWorkbenchHitIndex` is generation-owned. Product pointer routing borrows the committed index;
   it does not rebuild a surface on every pointer move.
2. Point hit testing selects one 64 px cell and visits candidates in reverse paint order. The existing
   10,000-node regression accepts no more than two candidate visits for the tested point.
3. Popup rows have an indexed path; uniform popup rows are selected in O(1) with at most two boundary
   candidates. Closed/no-popup panes skip popup scans.
4. Same-membership workbench patches reuse hit buckets and parent rows. Dock patches can replace only
   changed paint models while retaining the root hit cells.
5. Paint order, z-order, clipping, popup precedence/blocking, console scrolling, table identity and
   virtualized extents have explicit tests. These semantics are constraints on any later cutover.
6. M0b exposes the current generation's popup candidate rows as a borrowed crate-local slice for
   keyboard discovery and removes the now-redundant boolean query. Keyboard input no longer discards
   the index and returns to a full workbench-node scan.

## Structural findings

### P0: one owner conflates input hit membership and all presentation paint indexes

`HostWorkbenchHitIndex::from_presentation` builds workbench hit buckets, parent rows, popup rows and
extension-workspace metadata, then discovers every distinct template-node model in the root, menu,
page, status, docks, panes and floating windows and builds a paint index for each. Generic structure
replacement calls `indexes_presentation`; any changed model identity can synchronously rebuild this
combined owner even when the workbench hit membership is unchanged.

The existing workbench and dock rebind paths are valuable partial fast paths, not a complete ownership
model. M1 splits generation artifacts into a persistent hit-route index and independently replaceable
paint-range indexes keyed by stable model identity/version. A paint-only change must not advance or
rebuild input hit authority; a hit-only change must not rebuild unrelated pane paint indexes.

### P0: multi-cell damage queries allocate, deduplicate and sort on every paint request

Each `HostTemplateNodePaintIndex` sorts its full order and every populated bucket at build time, but
`rows_for_clip` only reuses that order for a full-model clip or one cell. A clip spanning multiple
cells constructs a new `HashSet`, constructs a new `Vec`, visits all bucket entries, deduplicates rows
and sorts candidates again. Template drawing calls this through `paint_workbench_row_indices` for its
effective clip; extension workspaces additionally retain candidates by walking parent chains.

This turns damage frequency into allocation and O(K log K) sort frequency even though buckets already
carry paint order. M2 uses an order-preserving k-way merge or a generation scratch/mark table with
monotonic paint-order emission. It must report cells, bucket entries, unique rows, duplicates,
allocations, sort count and ancestry visits before choosing the representation. A global full scan is
not an acceptable replacement.

### P1: fixed-cell insertion can amplify memory for large frames without telemetry

Every node row is copied into every covered 64 px cell. Full-pane containers and large clipped nodes
therefore consume area/cell-size-squared bucket entries. The current counters report query and
candidate counts, but not build rows, populated cells, total bucket entries, maximum bucket depth,
duplicate ratio, build bytes or build time. M1 adds these counters before changing cell size or adding
a large-frame tier; no cell-size guess is accepted without the 1/100/1K/10K and viewport-scale matrix.

### P1: pane surface construction tests dispatchability twice and materializes owned paths

`build_template_surface_frame` first scans until any dispatchable node is found and then
`template_nodes_surface_frame` scans all nodes again, repeating component-family classification for
the prefix. Every dispatchable row also owns component/control strings and a formatted node path.
Pane conversion builds this artifact for every converted pane and may build it a second time after
runtime-diagnostics reflection; scoped UI-asset patching rebuilds it for each changed presentation.

M0 now folds dispatch selection and insertion into one lazy iterator pass while preserving the empty
result, row-to-node id mapping, build counter and surface semantics. The new contract moved RED 0/2
to GREEN 2/2. String/path ownership remains for M1, where the runtime surface contract and stable
node identities must be reviewed together.

### P1: special-case and fallback paths remain outside one arranged generation authority

Console hit testing uses visible-line metadata for scrolled rows but linearly scans remaining static
nodes. Pane popup rows have a separate index and a reverse-scan fallback when the index is invalid.
The runtime `UiSurfaceFrame`, workbench hit buckets, popup row index and paint index are four spatial
representations with different rebuild rules.

M3 publishes one arranged generation artifact with explicit hit, popup and paint projections. The
product may retain specialized representations, but their lifetime/version source and fallback
policy must be common and observable. A stale index must request a generation rebuild or fail closed;
normal current-generation input must not silently return to a collection-scale scan.

### P2: model discovery and subtree filtering have avoidable scale terms

Presentation paint-model discovery deduplicates each model with a linear `iter().any`, producing O(M2)
identity checks across panes/windows/plugins. Extension subtree queries retain K clip candidates and
walk parent links up to tree depth for each. These are below the current input P0 but must be replaced
by stable model keys and precomputed subtree intervals/ancestry once M1 defines the generation index.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h`

Unreal's `FHittestGrid` uses 128x128 cells, a persistent widget map/array and ordered per-cell widget
indices (`HittestGrid.cpp:109-179`). A point query selects the cell, finds the best real widget and
then builds the actual parent bubble path (`HittestGrid.cpp:189-264`). The public/private source also
provides explicit add/update/remove operations; hit membership is maintained as widget state changes
rather than reconstructed per pointer event.

Painting has a separate invalidation authority. `FSlateInvalidationRoot` clears and rebuilds the full
widget list only on the slow path, then repopulates cached elements (`SlateInvalidationRoot.cpp:389-
423`). The fast path processes an ordered invalidation/update list and reindexes only required ranges
(`SlateInvalidationRoot.cpp:723-849`). `ProcessInvalidation` separately times pre-update, attributes,
prepass and post-update and falls back to a slow rebuild only when required (`SlateInvalidationRoot.cpp:
1281-1405`).

The transferable rule is not Unreal's exact 128 px constant. It is persistent hit membership plus a
separate cached/invalidation paint lifecycle, with full rebuilds observable as a slow path. Zircon's
point grid follows the first half; M1-M3 close the paint-ownership gap.

## Target architecture

1. `HostPresentationGeneration` publishes separate `HostHitRouteIndex` and
   `HostPaintRangeIndexSet`, both tied to the same structure generation but independently reusable.
2. Stable model identity/version selects a paint index in O(1); presentation discovery does not
   linearly deduplicate `ModelRc` values on every comparison.
3. Paint indexes retain global paint rank, cell ranges and subtree intervals. Multi-cell queries emit
   unique rows in paint order without per-query sorting or heap allocation after warm-up.
4. Large frames use measured policy: normal cell insertion, coarse/large-frame tier, or interval
   coverage chosen from bucket-entry and query evidence.
5. Pane surface frames, popup rows and workbench routes are projections of one arranged generation.
   Current-generation routing has no linear fallback.
6. Full rebuild, partial rebind and slow fallback have separate counters/timings and reason codes.

## Instrumentation and acceptance

| Evidence | Acceptance |
| --- | --- |
| hit queries/cells/candidate visits/path depth | one cell for point hit; bounded candidates; no build on stable pointer move |
| hit-index full builds/rebinds/build CPU/bytes | paint-only updates do not rebuild hit membership |
| paint-index models/full builds/partial replacements | only changed model indexes rebuild |
| cells/bucket entries/max depth/large-frame rows | memory slope reported; no unexplained area amplification |
| paint clip cells/raw entries/unique rows/duplicates | output matches full ordered reference |
| paint query allocations/sorts | zero after warm-up; zero query-time sort |
| subtree candidates/ancestry visits | interval-bounded, no K*depth parent walk |
| surface build row visits/owned bytes | one dispatch classification per row; stable-generation reuse |
| UI correctness | exact clip/z/popup/disabled/console/table/input parity |

Matrix: nodes `1/100/1K/10K`; panes `1/10/100`; windows/plugins `1/10/100`; frame shape
`small/full-width/full-pane/nested`; clip `one-cell/multi-cell/full/disjoint`; update
`semantic/geometry/order/model/popup`; input `stable/moving`; damage `1/10/100 regions`; scale
`1x/1.5x/2x/4K`.

WPR owns CPU, allocation, context-switch and power evidence. RenderDoc is used only after a current-
source GPU presenter is launchable, and only for submitted draw/resource/pixel parity. Artifacts stay
on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Make template surface dispatch/build selection a single pass. | applied; static contract GREEN, Rust/dynamic pending |
| M0b | Reuse borrowed generation popup candidates for native keyboard discovery. | applied; combined static contracts GREEN, Rust/dynamic pending |
| M1 | Split hit and paint generation owners; add build/memory/reason telemetry and O(1) model identity. | paint-only updates never rebuild hit index |
| M2 | Replace multi-cell allocate/dedup/sort and subtree parent walks with ordered ranges/scratch marks and subtree intervals. | zero warm query allocation/sort; parity passes |
| M3 | Converge pane surface, popup and workbench representations under one arranged generation; remove normal linear fallbacks. | scale, input, WPR and UI parity acceptance |

## Validation state

- Owner source review: passed, 18/18 Rust files.
- Host state, generation paint scope, pane conversion/scoped patch, pointer route and paint consumers:
  read and mapped.
- Unreal persistent hit grid and invalidation slow/fast paint paths: read and mapped.
- M0 static contract moved RED 0/2 to GREEN 2/2. M0b's native-keyboard contract moved RED 0/3 to
  GREEN 3/3. Together with adjacent popup, hit-index, presentation-generation, preview-index and
  paged-keyboard contracts, the focused set passes 18/18.
- The changed Rust file passes independent `rustfmt --check`; scoped `git diff --check` passes with a
  line-ending warning only.
- Performance-contract discovery passes 136/142. Five failures are the known two missing test-support
  files, missing `available_slots`, preview resize `.roots.clone()` and UI-asset root helper
  `.roots.clone()`. The sixth is adjacent Runtime 07 source/telemetry/owner-gate document drift; it is
  not caused by this owner and remains routed to its owning plan.
- Existing source tests provide strong static/unit contracts, including a 10,000-node point-hit case
  with at most two candidate visits. They are source evidence only in this session.
- Managed Rust tests, current-source launch, WPR and RenderDoc remain pending because the managed Cargo
  Session is terminal `archived` with `cargo_session_not_executable`. No raw Cargo bypass or dynamic
  complexity/power claim is permitted.
- M1-M3 and M0 dynamic acceptance remain pending; this owner stays in `pending.md` until current-
  source dynamic acceptance.
