---
title: Editor profile hit-route single-authority performance review
date: 2026-08-23
module: zircon_editor retained-host profiling_hit_routes
priority: MVP-P0 input evidence integrity and capture scale
status: source_reviewed_m0_allocation_cleanup_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate persistent hit-test grid and real bubble path
---

# Goal

Make performance capture validate the exact retained-host pointer route used by the editor, using one
generation-owned spatial/control index. Profiling must not maintain a second string-dispatched router,
linearly rediscover controls for every sample, or report self-consistency as product-route parity.

## Reviewed source

- owner Rust files: 18/18
- lines: 553
- bytes: 16,674
- source-only SHA256 over lexicographically sorted owner files:
  `c1222deab0d3255812a48b8108b4fd896846c6fbc4d26e581cd817da7235e9be`
- post-M0 owner files/lines/bytes/SHA256: 19 / 566 / 17,013 /
  `96376ee1d9300bd8ea306176e61ead319988c02bc58568ae8db62555e0c9298c`
- owning commit at review: `0d70d1ac6499abcf56c3f6c3ef43cb3a7502a249`

| Owner group | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `host_contract/profiling_hit_routes.rs` | 1/1 | 8 | 175 |
| `host_contract/profiling_hit_routes/*.rs` | 6/6 | 244 | 7,506 |
| `host_contract/profiling_hit_routes/tabs/**` | 5/5 | 138 | 4,046 |
| `host_contract/profiling_hit_routes/template/**` | 3/3 | 79 | 2,427 |
| `host_contract/profiling_hit_routes/viewport_toolbar/**` | 3/3 | 84 | 2,520 |

All owner files were read in full. Direct consumers were read through profile geometry/sample
generation; actual product routing was compared through `native_pointer/routing/**`,
`surface_hit_test/**`, `HostWorkbenchHitIndex`, pane surface frames and the runtime cell hit grid. The
cited Unreal `FHittestGrid` and `FSlateApplication` call path was read directly. Supporting files are
not counted as owner coverage.

The 2026-07-17 combined artifact/route report covered an older 526-line route owner. This current
record supersedes its coverage and retains the still-valid scale findings.

## Correct foundations to retain

1. Profile samples include center and two outside points, so capture can detect containment, clipping
   and surface-routing errors instead of only exporting decorative rectangles.
2. Template and viewport-toolbar checks ultimately query the retained `UiSurfaceFrame` hit grid and
   inspect its top hit. They do not linearly walk every arranged node for the final point query.
3. Route order respects docked surfaces before floating windows and requires content/header/rail
   containment. Invalid kinds and surfaces fail closed.
4. Floating-tab lookup checks the requested window identity before scanning its tabs; fixed drawer
   surfaces short-circuit before touching unrelated models.
5. Product routing already has typed `ChromePointerRoute`, `PanePointerRoute`,
   `TemplateNodePointerHit`, surface-frame hit entries and generation-owned template hit indexes. The
   required single authority exists in adjacent code; it does not need to be invented in profiling.

## Structural findings

### P0: profile evidence uses a duplicate router instead of the product pointer route

`route_contains_profile_frame` dispatches on schema strings and calls manually maintained tab, rail,
template and toolbar routines. Actual editor input uses `native_pointer/routing/**` and
`surface_hit_test/**`, returning typed route targets with product ordering, popup, close-button,
workbench and pane semantics. The profile router does not call that route and cannot prove that an
actual click reaches the expected target.

This is an evidence-integrity defect before it is a micro-performance issue. A duplicated route can
remain internally consistent with duplicated exported geometry while the product path changes. M1
defines a stable typed route identity and lets both input dispatch and capture consume one actual
route result. Profiling compares expected identity to the product result; it does not reimplement the
decision.

### P0: three samples per control repeat collection-scale row searches

Every clickable frame creates three samples, and each calls this router independently. Document,
drawer and page tabs scan their row models to find an id. Activity rails scan every button and format
an expected id per row. Floating tabs scan windows then tabs. With C controls of one kind, validating
all samples can approach 3*C*C row visits. Template/toolbar paths avoid full-tree scans at the final
point because the runtime grid is cell-indexed, but they still retry fixed panes/floating windows and
allocate surface prefixes.

M1/M2 route each sample once through the actual generation index. Expected target identity is already
known while geometry is collected, so no subsequent id-to-row rediscovery is allowed. Acceptance is
near-linear control scaling with exact route parity.

### P1: simple route checks allocate strings and transform nonmatching frames

Activity rail, template and viewport-toolbar checks use `format!` for ids/prefixes in their hot
capture loops. Template/toolbar format once per attempted pane and again after the hit. Shared tab
routing translates or clones every row's frame before comparing its control id, even when most ids do
not match.

M0 now replaces formatted identity checks with allocation-free borrowed prefix/surface parsing and
checks tab id before frame transformation. The shared parser is a temporary M0 mechanism, not a new
route authority; M1 still replaces the complete string protocol with typed product-route identity.

### P1: profile identities are untyped concatenated strings

Kinds, surfaces and controls cross module boundaries as concatenated strings such as
`template.{surface}.{control}`. Every consumer must parse or rebuild the convention, and a surface or
control containing separators can make ownership ambiguous. Product routes already carry enum
variants and separate fields.

M1 exports a serialized typed identity derived from the product route. Human-readable ids remain
schema output, not the lookup key or routing protocol.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`

Unreal's `FHittestGrid` partitions the hit area into 128x128 cells and retains a widget map, sparse
widget array and ordered per-cell indices (`HittestGrid.cpp:109-160`; `HittestGrid.h:260-355`). Widgets
are added/updated/removed in that persistent index (`HittestGrid.cpp:859-976`). A point query selects
the relevant cell, obtains the best real widget and builds its actual bubble path
(`HittestGrid.cpp:189-264,986+`).

`FSlateApplication::LocateWidgetInWindow` calls that window's `GetHittestGrid().GetBubblePath` and
returns the resulting product event path (`SlateApplication.cpp:1988-2014`). The transferable rule is
one spatial index and one real routed identity. Debugging/automation observes that result; it does not
keep a parallel string router for each control family.

## Target architecture

1. Define a stable serializable `HostPointerRouteIdentity` alongside typed product routes, with
   variant, surface/window, control/node and action identity as separate fields.
2. Route a point once through the product generation's chrome/pane/workbench/surface hit indexes and
   derive the identity from that result.
3. Carry expected typed identity from geometry collection into each sample. Never rediscover a frame
   by scanning row models from a string id.
4. Keep one generation-owned spatial/control index for capture and input; retain cell-bounded queries,
   z-order, clipping, popup projection and bubble ancestry.
5. Export human-readable ids only after comparison and outside product routing.
6. Delete `profiling_hit_routes/**` after all capture consumers use the product route receipt; do not
   preserve a compatibility router.

## Instrumentation and acceptance

| Evidence | Acceptance |
| --- | --- |
| actual product route calls | exactly one per sample |
| row/window/control visits | near O(C), no id rediscovery scans |
| hit-grid cells/candidates/path depth | bounded and separately reported |
| string allocations/formats | zero in route decision |
| expected vs actual typed identity | exact variant/surface/control/node/action match |
| popup/clip/z-order/disabled/custom-path behavior | exact parity with product input |
| capture CPU/allocated bytes | scale slope reported for 1/100/1K/10K controls |
| actual click dispatch | automation confirms the same target used by evidence |

Matrix: control kind `splitter/tab/rail/template/toolbar/workbench`; surface
`document/left/right/bottom/floating`; point `center/edges/outside/overlapped/clipped`; state
`enabled/disabled/hidden/popup`; controls `1/100/1K/10K`; windows `1/10/100`; generation
`stable/changed`; capture `off/on`.

WPR owns CPU, allocations and context switches for capture scale. RenderDoc is not a route authority;
it is used only when the same sample also needs GPU pixel/draw parity. All artifacts stay on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Remove route-loop string formatting and nonmatching tab frame transforms. | applied; static contract GREEN, Rust/dynamic pending |
| M1 | Add typed route identity and route capture through the actual product path. | duplicate router no longer authoritative |
| M2 | Carry expected identity from generation collection and delete row rediscovery. | near-linear 1/100/1K/10K scale |
| M3 | Hard-delete `profiling_hit_routes/**` and run parity/dispatch/WPR matrices. | exact actual-route evidence and dynamic acceptance |

## Validation state

- Owner source review: passed, 18/18 Rust files.
- Profile geometry, product chrome/pane/workbench routes, surface hit grid and generation indexes: read
  and mapped.
- Unreal persistent hit grid and real bubble-path routing: read and mapped.
- M0 static contract moved RED 0/3 to GREEN 3/3. Together with the adjacent artifact gate contract,
  the focused set passes 5/5.
- Eight changed/new Rust files pass independent `rustfmt --check`; scoped `git diff --check` passes
  with line-ending warnings only.
- Performance-contract discovery passes 121/126. The five failures are unchanged: two missing test-
  support files, missing `available_slots`, preview resize `.roots.clone()` and UI-asset root helper
  `.roots.clone()`.
- Managed Rust tests and WPR remain pending because the managed Cargo Session is terminal `archived`
  with `cargo_session_not_executable`. No raw Cargo bypass or dynamic complexity claim is permitted.
- M1-M3 remain structural/dynamic work. This owner stays in `pending.md` until the duplicate router is
  deleted and the current-source matrix passes.
