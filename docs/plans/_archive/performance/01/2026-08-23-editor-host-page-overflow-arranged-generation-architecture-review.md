---
title: Editor host page overflow arranged generation performance review
date: 2026-08-23
module: zircon_editor retained-host host_page_overflow_menu
priority: MVP-P1 editor page-tab overflow input and paint latency
status: source_reviewed_m0_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate arranged docking tab well and dirty-driven virtualized list
---

# Goal

Publish page-overflow popup geometry, visible rows, hidden-page identity and lookup maps once with the
committed page-strip generation. Paint, pointer, keyboard and scroll consumers must share it. Stable
input must not rescan even the visible window, linearly search hidden pages, rebuild keyboard rows or
recompute independently versioned popup geometry.

## Reviewed source

- owner Rust files: 4/4
- lines: 936
- bytes: 30,445
- source-only SHA256 over lexicographically sorted owner files:
  `9e10f52ae0c539ee11cbffc16ddfb4ad2158341dd940d899dcaef595f9b1d45e`
- post-M0 owner files/lines/bytes/SHA256: 4 / 989 / 32,095 /
  `909732dbeb0cae7c753517d5bf6c85f886d5bdb673852db9dcec8c2d3d38e661`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`

| Owner group | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `host_page_overflow_menu.rs` | 1/1 | 478 | 16,311 |
| `host_page_overflow_menu/tests/**` | 3/3 | 458 | 14,134 |

All owner files were read in full. Page-overflow pointer dispatch, keyboard target/actions, scene-layer
paint, presentation generation/state and the existing page-strip owner report were inspected as
direct boundaries. Unreal docking tab-well and table-view regeneration sources were read directly.
The 2026-07-17 combined record treated this as one small file; this current record supersedes that
stale coverage and includes the split geometry/scrolling tests.

## Correct foundations to retain

1. Popup width consumes `overflow_widest_title_width_px` projected by page layout; it does not measure
   every hidden title on input or paint.
2. Content extent, scroll clamp and visible row range are closed-form O(1) math. Paint consumers can
   iterate only strict viewport intersections rather than all hidden pages.
3. Popup placement handles above/below space, shell offsets, narrow shells, non-finite anchors and
   scrollbar reserve with explicit tests.
4. Hit containment excludes the scrollbar gutter and clipped row portions. Scroll and keyboard reveal
   use the same row height/stride constants.
5. Page overflow state is an immutable generation snapshot and overlay dispatch gives it precedence.

## Structural findings

### P0: equal-height row hit still loops over the visible window

`host_page_overflow_row_hit_in_popup_for_scroll` computes the strict visible range, then builds/tests
each visible row frame until one contains the point. Since rows are uniform, one Y/stride calculation
identifies at most one candidate; the frame check is still required to reject the row gap and strict
viewport boundaries.

M0 now replaces the loop with one candidate calculation plus range/frame containment. Complexity
becomes O(1) independent of viewport height while preserving gutter, clipping, gap and boundary
semantics.

### P0: keyboard reconstructs all hidden rows per event outside this owner

`native_keyboard/target/page_overflow.rs` builds a new row vector for every key/text event, clones tab
titles/ids, formats each page index and recomputes every row frame. Reveal then calls this owner and
linearly searches `overflow_hidden_tab_indices` for page-to-row mapping. These costs scale with hidden
pages and repeat frequency despite a stable popup.

M1 adds page-to-hidden-row O(1) identity and immutable visible navigation rows to the committed
overflow artifact. It is coordinated with the native-keyboard popup navigation plan; this owner does
not add a second cache.

### P1: paint, pointer and keyboard recompute one popup from raw presentation

Each consumer calls popup-frame, viewport, scrollbar, visible-range, clamp and row-frame helpers
independently. The math is cheap, but multiple owners can observe different structure/interaction
generations or drift in clipping/selection policy. The existing page-strip review already found
duplicate projection/pointer layout owners before this overflow stage.

M1/M2 extend the immutable page-strip generation with an overflow artifact containing popup/viewport/
scrollbar frames, content/max scroll, visible row range, hidden page identities, page-to-row map and
row geometry parameters. Only scroll/hover interaction projections change on stable structure.

### P1: repeated helper composition hides actual work and invalidation reasons

One popup-frame query calls content extent more than once through scrollbar/max-scroll helpers and
reads theme metrics in multiple helpers. There are no counters for layout builds, geometry queries,
row candidates, page searches, row-vector builds, title/id clones or invalidation reason. M1 records
these before deciding whether any further arithmetic consolidation matters.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabWell.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Views/STableViewBase.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`

`SDockingTabWell::OnArrangeChildren` computes child sizing once and performs one ordered arrangement
pass (`SDockingTabWell.cpp:100-189`); paint consumes arranged children, while desired size reads
cached child desired sizes (`268-295`). `STableViewBase::Tick` regenerates list items only when items
need refresh or panel geometry changes, then retains the result for scrolling/scrollbar state
(`STableViewBase.cpp:424-510`). `SListView` supplies that visible-item regeneration implementation.

The transferable rule is arranged geometry plus dirty-driven visible-row generation shared by input
and paint. Zircon's closed-form equal-row math can be more compact than Slate widgets, but it needs the
same committed lifetime and invalidation authority.

## Target architecture

1. The page-strip generation owns `HostPageOverflowLayoutArtifact`, keyed by page topology/title/
   active state, strip/shell frames, layout tier, text/theme metrics and scale.
2. It stores popup/viewport/scrollbar frames, content/max-scroll, hidden page identities, page-to-row
   lookup and uniform row origin/stride/height.
3. Pointer hit maps Y to one candidate; keyboard uses retained visible descriptors and O(1) page-row
   lookup; paint iterates only the artifact's visible range.
4. Scroll/hover update a small interaction projection without rebuilding stable structure or rows.
5. Projection, pointer, keyboard and paint reject generation mismatch rather than reconstruct raw
   presentation fallbacks.

## Instrumentation and acceptance

| Evidence | Acceptance |
| --- | --- |
| overflow layout builds/rebinds/reasons | zero build for stable pointer/key/paint |
| geometry queries/content calculations | consumers borrow one artifact after M1 |
| hit row candidates/frame probes | at most one candidate and one row-frame probe |
| keyboard row builds/page visits/cloned bytes | zero stable-event row build; O(1) page-to-row |
| painted/visible/hidden rows | paint visits only strict visible range |
| input CPU p50/p95/p99 | slope independent of total hidden pages after artifact build |
| correctness | placement, gutter, gaps, strict clipping, scroll/reveal/selection and width parity |

Matrix: pages/hidden pages `0/1/2/8/100/1K/10K`; viewport rows `1/12/100`; width
`96/320/640/1260/1920`; press `row/gap/gutter/top/bottom/outside`; scroll `0/fractional/max/NaN`;
input repeat `1/10/30/60 Hz`; update `none/hover/scroll/active/title/topology/resize/theme`; scale
`1x/1.25x/1.5x/2x/4K`.

WPR owns CPU, allocation, context-switch and power evidence. RenderDoc is used only after a current-
source GPU presenter is launchable and only for overflow draw/resource/pixel parity. All artifacts
remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Replace visible-row hit loop with one Y/stride candidate and exact checks. | applied; static contract GREEN, managed Rust/dynamic pending |
| M1 | Extend committed page-strip generation with overflow artifact, lookup and telemetry. | one build per invalidated generation |
| M2 | Make pointer, keyboard and paint borrow the artifact; delete reconstruction and linear reveal lookup. | zero stable-event rebuild/search |
| M3 | Run page/viewport/repeat/WPR/power/UI and GPU parity matrices. | quantified acceptance and milestone closeout |

## Validation state

- Owner source review: passed, 4/4 current Rust files.
- Pointer, keyboard, paint, generation/state and page-strip boundaries: read and mapped.
- Unreal arranged tab well and dirty-driven virtual list sources: read and mapped.
- M0 static performance contract moved RED 0/1 to GREEN 1/1. Together with paged keyboard, native
  keyboard/dismiss and presentation-generation contracts, the focused set passes 14/14.
- Rust regressions record row-gap rejection and strict viewport-bottom rejection; execution remains
  pending with managed Rust validation.
- The changed Rust files pass independent `rustfmt --check`; scoped `git diff --check` passes with
  line-ending warnings only.
- Performance-contract discovery passes 142/148. The six unrelated failures remain the known two
  missing test-support files, missing `available_slots`, preview resize `.roots.clone()`, UI-asset
  root helper `.roots.clone()` and Runtime 07 source/telemetry/owner-gate document drift.
- Managed Rust tests, current-source launch, WPR and RenderDoc remain pending because the managed
  Cargo Session is terminal `archived` with `cargo_session_not_executable`. No raw Cargo bypass is
  allowed.
- M0 dynamic acceptance and M1-M3 remain pending; this owner stays out of `review.md` until dynamic
  acceptance.
