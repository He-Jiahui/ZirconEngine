---
title: Editor workbench menu visible-row and state-generation performance review
date: 2026-08-22
module: zircon_editor retained-host paint_workbench_renderer menus
priority: MVP-P0 basic editor menu bar and popup interaction
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate list-view visible row generation, invalidation and menu anchor
---

# Goal

Make one menu-state/source/layout generation own menu bar, popup and submenu paint. Scrolled popups
must visit only rows intersecting their viewport; stable labels/shortcuts must use retained text-layout
artifacts. Paint geometry must consume an already selected state view instead of rematerializing menu
state for every row.

## Reviewed source

- direct Rust files: 12/12
- lines: 805
- bytes: 26,346
- joined path-and-raw-source-bytes SHA256:
  `3566187d19e9eebc03c234ebb1080a0200ef5af2e09c0a5864d47d03b7313f50`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

Direct scope: `paint_workbench_renderer/menus.rs` and
`paint_workbench_renderer/menus/**`.

Supporting current-source owners read/traced in full where changed or behavior-defining:

- `host_contract/menu_popup_metrics.rs`: 38 lines, SHA256
  `427e4b0b6e12f0e883f43fc8078bd830a0cc3b7c034a4a35126c49de3f442ff7`
- `host_contract/host_page_overflow_menu.rs`: 506 lines, SHA256
  `b33dba33e9959814588c3bfd0a00beb10b9777f3396783831441a6412c8e9fdd`
- all three overflow-menu test files, including exact scroll/intersection boundaries
- menu-state paint override owner, Runtime text measurement and popup viewport contract

## Correct foundations to retain

1. Closed menus return before menu/frame/model lookup.
2. Root and submenu frames are constrained to the current workbench shell and flip around anchors.
3. Window-menu scroll is clamped to content extent; ordinary menus ignore stale window scroll state.
4. Row text frames and label clips are finite, bounded and use Runtime text with ellipsis.
5. Template-backed popup controls retain their separate template-node path.
6. Page-overflow menus already prove strict visible-row intersection and use that range for hit tests.

## Structural findings

### P0: scrolled menu rows iterate and lay out the complete item model

`draw_menu_popup_rows` loops `0..items.row_count()` for root and every open submenu. For each logical
row it reads data, builds geometry, checks hover, computes label/shortcut columns and emits up to two
Runtime text calls. Popup clipping rejects pixels only after this work. A window menu with `N` items
and `V` visible rows therefore performs O(N) row/text preparation instead of O(V).

M1 moves the proven strict-intersection formula into shared `menu_popup_metrics` with explicit
viewport height, scroll and first-row offset. Main menus and page overflow consume the same owner;
root/submenu rows iterate only that range. This preserves partially visible rows and excludes a row
that only touches the viewport boundary.

### P0: menu frame geometry rematerializes paint state per menu row

`draw_menu_bar_labels` calls `paint_menu_state` once, but `scrolled_menu_frame` calls it again for each
bar row. Without an active paint override, every call clones `HostMenuStateData` and allocates a new
`Arc`. Root popup similarly selects state once and then rematerializes it for anchor scrolling.

M1 changes geometry to accept the already selected `menu_bar_scroll_px` scalar. For `M` bar entries,
fallback state materializations become `M + 1 -> 1`; root popup becomes `2 -> 1`. The remaining one
fallback state clone is owned by the broader retained presentation-state plan.

### P0: stable row text is measured and emitted on every repaint

Every non-empty shortcut calls `menu_popup_text_width`, which invokes Runtime measurement. Runtime
text draw may also shape/clip label and shortcut strings. The item source already owns label,
shortcut, enabled and child data, so M2 must retain per-row text columns and shaped text by source,
width, font, locale and style generation. Hover changes only the row surface/text style patch.

### P0: submenu paint complexity is the sum of every open branch's complete children

The open path is followed by index, which is appropriate, but each level paints all children and
clones the child `ModelRc` handle for the next level. The handle clone is cheap; full row preparation
is not. M1 makes each level O(visible children); M3 retains the open popup stack and exact visible
range/layout generations so stable pointer movement does not rebuild unaffected levels.

### P1: menu-bar paint scans all entries after horizontal scrolling

The bar loops every menu frame and relies on primitive clipping. Menu counts are normally small, so
this is lower priority than popup rows. M3 should publish a visible bar range or retained command
ranges for extension-heavy menus, preserving open-menu identity and ordering.

### P1: immediate popup/chrome commands have no retained range owner

Surface, border, row backgrounds, texts and scrollbars are re-issued for every intersecting popup.
M4 converges menu ranges with the shared prepared render list; template-backed and fallback rows must
not become two persistent presentation authorities.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Views/SListPanel.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Input/SMenuAnchor.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`

`SListView` owns an item source and generates rows through `OnGenerateRow` when needed; `SListPanel`
arranges the generated children and invalidates item dimensions as layout changes. `SMenuAnchor`
separates popup placement from content ownership. Slate widget proxies repaint from explicit update
flags rather than rematerializing state for every row geometry query.

The transferable constraints are visible-row generation, one item/source owner, retained generated
rows, explicit layout invalidation and separate popup placement. Zircon should not copy Unreal pointer
lifetimes or assume Unreal timings; current-source Zircon captures remain mandatory.

## Target architecture

1. A menu generation owns source rows/tree, open path, hover/selection, scroll, shell/layout, text and
   style sub-generations.
2. Root and submenu viewports derive strict visible ranges from the shared menu metric owner.
3. Retained row artifacts store text columns/shaping and typed command ranges; hover patches one row.
4. The open popup stack retains per-level bounds, visible range and source generation; unchanged
   levels reuse their artifacts.
5. Menu bar and popup geometry consume one selected state view, never creating per-row state owners.
6. Prepared render-list consumers reuse canonical menu ranges and remove duplicate immediate routes.

## Instrumentation and acceptance

Matrix: bar items `1/8/64/1k`, popup items `0/8/1k/100k`, open depth `0/1/8/64`, viewport rows
`1/8/32`, scroll `top/middle/bottom/non-finite`, shortcuts `0/50%/100%`, stable/hover/source/width
changes, one/eight surfaces.

| Evidence | Acceptance |
| --- | --- |
| logical/visible/visited rows per popup level | visited equals strict visible range |
| paint-state materializations/cloned bytes | one per draw fallback; zero per row |
| text measure/layout/shaping and cloned bytes | zero for unchanged retained rows |
| popup-level/range/command rebuild and reuse | proportional to changed levels/rows |
| CPU/allocation/RSS/latency/context switches/power | same executable/workload before and after |
| RenderDoc draw/batch/GPU and pixel/text parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add state materialization, logical/visible/visited row, text and retained-range counters. | attributable baseline |
| M1 | Share strict visible-row math; window root/submenus; pass selected scroll to geometry. | O(V) rows; no per-row state clone |
| M2 | Retain row text columns/shaping by source/layout/style generation. | unchanged text work zero |
| M3 | Retain open popup stack and visible menu-bar ranges with addressed hover patches. | rebuild proportional to changed levels/rows |
| M4 | Converge menu command ranges with shared prepared render list. | one presentation authority |
| M5 | Hard-cut fallback state clones after retained presentation-state view migration. | zero paint-time state ownership copies |
| M6 | Run managed scale/input/WPR/power and RenderDoc/pixel/text parity matrix. | quantified accepted milestone |

## M1 implementation result

`menu_popup_metrics` now owns one strict visible-row range function parameterized by item count,
viewport height, scroll and the first row's offset from the viewport. It preserves partially visible
rows, excludes rows that only touch a boundary and rejects non-finite/empty inputs. Page overflow now
delegates its existing behavior to this owner. Root and submenu fallback rows iterate the same range
with the popup edge inset.

`scrolled_menu_frame` now accepts the already selected scroll scalar. It no longer reads
`HostWindowPresentationData` or calls `paint_menu_state` from row geometry.

| Static paint work | Before | After | Change |
| --- | ---: | ---: | ---: |
| rows prepared per popup level | logical `N` | visible `V` | `O(N) -> O(V)` |
| total rows across open levels | `sum(N_level)` | `sum(V_level)` | viewport bounded |
| shortcut column/runtime width work | up to `N` | up to `V` | off-viewport zero |
| menu-bar fallback state materializations | `M + 1` | `1` | per-row ownership removed |
| root-popup fallback state materializations | `2` | `1` | duplicate geometry read removed |
| strict visible-range formula owners | 2 | 1 | shared metric owner |

These are source-path counts, not elapsed-time claims. Template-backed popup rows remain governed by
the template-node plan; M0/M2-M5 own measurements and retained artifacts.

Post-M1 direct owner scope:

- Rust files: 12/12
- lines: 811
- bytes: 26,426
- joined path-and-raw-source-bytes SHA256:
  `d70536b29773ff952ac1befca1592027064628a0a73d3f5848cf1424c3c1c349`
- unchanged direct owner files: 8 retain their pre-M1 fingerprints inside the joined hash above

| Changed direct owner file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `menus/bar.rs` | 186 | 6,034 | `1af5360756d7cd030c5068654e1e2850fcc6ae1d73f5e29ee6d3deae9348d237` |
| `menus/geometry/scroll.rs` | 13 | 359 | `238b7dec066f6975c936ee9ce93acd33f17c4e06cdf8363b21afe89e3bd31bb1` |
| `menus/popup.rs` | 114 | 3,666 | `ef6efc3ef503a1a7ba3ed6a6e33e06efa09ab9d407c808db300a6029ee3e7071` |
| `menus/rows.rs` | 168 | 5,579 | `726d6e19550775d5fac8bccabedd585fffb4b11d42520e0069e517f67adac9b3` |

Changed supporting shared owners:

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `host_contract/menu_popup_metrics.rs` | 98 | 3,453 | `872bc41fa403ec122f442eb0cfc192c22d2e6ba633f590d375cdb5c482f207aa` |
| `host_contract/host_page_overflow_menu.rs` | 478 | 16,311 | `3b2e3ebb6bff803697a983e2160ce2acdec7b6d9ed32d34f5cc4296a67b8f464` |

Focused static contract:
`tools/tests/test_editor_workbench_menu_visible_row_performance_contract.py`, 60 lines, 2,579
bytes, SHA256
`24fc873df6bbef3f5df2bf7ee1d7f0bfcdd9dbe40a8ac23b4be909555c32ac69`.

## Validation state

- Direct owner review: passed, 12/12 Rust files.
- Shared overflow visible-range owner and all direct tests: read; current behavior mapped.
- Relevant Unreal list/menu/invalidation sources above: read.
- M1 focused contract: RED 4/4 before the change, GREEN 4/4 after the change.
- Current owned editor performance-contract set: GREEN 65/65.
- `rustfmt --check` for the six changed Rust files and scoped `git diff --check`: passed.
- Shared Rust range tests cover strict boundary, popup inset and invalid input; existing overflow tests
  cover exact intersecting rows and top-edge contact. They are present but not claimed passing until
  managed Cargo is executable.
- M0 and M2-M6 remain pending; no elapsed-time, GPU or power claim is made from static counts.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived` and rejects Cargo
  launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a launchable current-source editor. RenderDoc cannot validate
  row-model traversal, state cloning or text measurement.

The module remains in `pending.md` until M0-M6 pass on one source/executable/workload fingerprint.
