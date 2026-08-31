---
title: Editor diagnostics, timeline and plugin retained-row virtualization performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/pane_data_conversion/{runtime_diagnostics.rs,performance_timeline.rs,module_plugins.rs}
priority: MVP-P0 runtime diagnostics, performance timeline and plugin manager
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate SListView visible-row generation and reuse
---

# Goal

Keep runtime diagnostics, profiling and plugin data in one immutable generation and materialize only
the rows visible in the active pane. Stable generations must reuse retained rows; diagnostic
inspection must not execute a second UI pipeline during ordinary pane conversion; hotspot analysis
must avoid per-sample key allocation and full per-group sorting when only percentile statistics are
required.

## Reviewed source

- owner files: runtime diagnostics, performance timeline and module-plugin final pane conversion
- Rust files: 3/3
- current lines: 1,477
- current bytes: 52,483
- joined current source-bytes SHA256:
  `b98b2d27de2eae8124a39d07848982ba0af85e6f276e50117a26bb4d54469476`
- joined pre-M1 source-bytes SHA256:
  `f8a8eae6e53146044a3803e8e7d343c193bcef46a1a052645f226c782d5607a4`
- owning commit before review: `4d5f52aa2b76a3a877aabdd47b01a98dcdd59493`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `runtime_diagnostics.rs` | 487 | 15,897 | `aafa5c7305c5aca89e7697cc91fed6ccb0fbc646b87862a9b432e44618c599bb` |
| `performance_timeline.rs` | 503 | 18,151 | `0e7a812274cc713c40937ac3bdaed833094ee2570b07480ea788841f10199337` |
| `module_plugins.rs` | 487 | 18,435 | `f5f2a251d9602a81dc2b87b94c8e0be26ceaf40f863577dc936aa50508623659` |

All three files were read in full. Production ownership was followed through timeline payload
construction, runtime hotspot aggregation, plugin status-report generation/cache, pane projection,
debug-reflector schedule sections and final pane hit-artifact rebuilding. These related files are not
counted in the 3/3 owner total.

## Existing foundations to retain

Pane-variant projection now builds only the active typed payload. Timeline payload display rows are
bounded to 12 frames, 12 spans and 12 hotspots, so presentation growth is bounded. Plugin view rows
are cached by immutable `Arc<EditorPluginStatusReport>` identity, making stable production reads
O(1). `ModelRc` and `SharedString` give a viable shared-generation owner. These foundations should be
extended through final host rows rather than replaced.

## Structural findings

### P0: ordinary diagnostics conversion runs a second UI pipeline

After the pane is converted and hit artifacts are rebuilt, `pane_conversion.rs` calls
`refresh_runtime_diagnostics_debug_reflector_from_body_surface`. Unless an externally captured
reflector snapshot is marked active, this constructs a new `UiSurface`, copies every diagnostic node
into a `UiTreeNode` with new paths, attributes and state, calls `surface.rebuild()`, snapshots the
surface, formats a new reflector model, replaces all diagnostic nodes and rebuilds pane hit artifacts
a second time.

This is not retained diagnostics. It is a self-reflecting full UI pipeline inside normal pane apply.
The work scales with all template/status/detail/section/node rows and runs on the main presentation
path. Debug reflection must be driven by an explicit capture generation and consume the already
committed body surface receipt. Ordinary conversion must execute one layout/hit pipeline.

### P0: plugin rows are cached before, but fully copied and expanded after, the cache

The source status report and first `ModulePluginsPaneViewData` are correctly cached by `Arc` identity.
The final host conversion nevertheless maps every plugin to a second 26-field DTO and separately
generates visual nodes. Each plugin produces a row, title and metadata node, zero to two detail nodes,
and up to six action buttons: 3 to 11 wide nodes per plugin. The row also owns a second action list,
so labels and IDs are allocated again for row actions and buttons.

There is no visible-range calculation. At the existing 1/100/1,000-plugin validation scales, one
changed final apply visits every plugin, copies about 26/2,600/26,000 fields and creates about
3-11/300-1,100/3,000-11,000 nodes even if only a few rows fit in the pane. The plugin generation
cache must publish stable item identities through a virtualized retained list, not stop at the
intermediate DTO.

### P0: timeline display is bounded, but hotspot analysis is full and allocation-heavy

The final timeline duplicates its bounded typed rows into host DTOs and visual nodes. Each frame
becomes four nodes (track, fill, budget marker and formatted label); each span/hotspot becomes another
formatted node. At the current 12/12/12 row caps plus three controls this is up to 75 dynamic nodes
and four copied row models per conversion.

Before that bounded view is built, `analyze_hotspots` processes every recorded span. It clones
`stream`, `category`, `name` and `path` into a key for every span, stores frames in a `BTreeSet` even
though only the count is used, fully sorts every group's durations to read p95, then sorts all groups.
For N spans split into groups of sizes m_i this is O(total key bytes) allocation,
O(sum(m_i log m_i)) duration work plus ordered-set insertion, even though one percentile can be
selected without a full sort. The public all-hotspot report may remain complete, but its aggregation
algorithm must borrow sample keys and use expected O(N) grouping/frame membership plus selection.

### P1: diagnostics status and reflector rows repeatedly copy strings

Status ordering scans `detail_items` once for each of three priority prefixes, again for active
probes and again for the remainder, cloning matching strings before `push_label` copies them again.
The reflector-model path first allocates all selected labels and section display lines, then copies
them into wide template nodes. This local duplication is safe M1 work, but it does not replace the P0
explicit-capture boundary.

### P1: row identity is discarded at the final ABI boundary

Timeline and plugin `map_model_rc` calls borrow source rows but build entirely new models. Dynamic
nodes use row indices in IDs for the timeline and plugin IDs for plugins, yet neither conversion
receives a prior generation or visible-window receipt. Stable row identity, formatted labels and
geometry are therefore lost on each accepted shell generation.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Views/STableViewBase.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`

Unreal `SListView` observes stable item references and invokes `OnGenerateRow` only when it needs a
visual row (`SListView.h:233-252`). Its regeneration starts at the scroll offset, generates until the
available view area is filled, and stops (`SListView.h:1520-1628`). For an already visible item it
reuses the prior row widget and optionally refreshes it instead of creating a new widget
(`SListView.h:1664-1688`). `STableViewBase::RequestLayoutRefresh` coalesces the pending refresh flag
and invalidates the list layout (`STableViewBase.cpp:1393-1407`). The invalidation root separately
retains cached element data and updates only explicit invalid widgets on the fast path.

The transferable invariant is a stable data item owner plus a visible row window and exact refresh
request. A changed catalog or profile generation does not justify constructing every offscreen row,
and debug introspection does not justify executing another ordinary UI tree pipeline.

## Target architecture

1. Publish `RetainedPaneItemGeneration<T>` for timeline, plugins and reflector data with stable item
   IDs, source receipt and shared typed fields. Remove the intermediate-to-host duplicate row model.
2. Add one retained list-window owner containing scroll offset, viewport extent, overscan and visible
   item IDs. Generate/reuse only visible rows; stable rows keep identity and formatted text.
3. Drive debug reflection from an explicit `UiDebugCaptureReceipt` over the committed body surface.
   Normal pane apply never constructs a temporary surface or performs a second hit-artifact rebuild.
4. Cache hotspot analysis by profile generation. Borrow span strings while grouping, use hash frame
   membership and percentile selection; publish the completed immutable report once to timeline,
   diagnostics and export consumers.
5. Split row semantic data from geometry/style patches. Scrolling changes the visible window and
   geometry only; capture/catalog changes replace exact item rows.
6. Remove final full-model maps and all-node expansion after every consumer uses shared item
   generations and virtualized rows.

Complexity targets:

- unchanged pane generation: O(1), zero row/string/node construction;
- changed list with V visible rows and N total items: O(changed data + V), not O(N) host nodes;
- scroll-only update: O(V entering/leaving rows), retained overlap;
- diagnostic ordinary apply: one UI/hit pipeline, zero temporary surfaces;
- hotspot aggregation: expected O(N + G log G), no per-span key-string clones and no per-group full
  duration sort; G is unique hotspot count;
- final plugin/timeline duplicate row models: zero.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| temporary diagnostic surfaces/rebuilds per ordinary apply | 0 / one pane hit rebuild total |
| hotspot key string clones | 4G output strings, not 4N grouping strings |
| percentile duration comparisons | linear selection, no full per-group sort |
| host DTO rows/fields copied | visible changed rows only; stable = 0 |
| visual nodes generated/reused | O(V); overlapping rows reused |
| main-thread CPU and input-to-paint latency | report median/p95/max |

Matrix: diagnostic rows 0/1/100/1,000/10,000; profile spans 0/1/1,000/100,000/1,000,000 with
1/100/10,000 groups; plugins 0/1/100/1,000/10,000; viewport rows 1/8/32; stable applies and scrolls
1/1,000; source, capture, scroll, geometry, style and render-only changes. Capture aggregation,
conversion and node-build counts, strings/bytes, allocations, main-thread CPU, interaction latency,
RSS and package energy on one source/executable fingerprint.

Use managed Windows validation and WPR/ETW with artifacts only on D/E/F. RenderDoc is reserved for
current-source draw/pixel parity after a launchable editor exists; it cannot prove CPU aggregation,
temporary-surface or virtualization cost.

## M1 result

`analyze_hotspots` now groups with borrowed `&str` keys, accumulates total/max/over-budget values in
the input pass, uses hash frame membership and selects the p95 order statistic without sorting every
duration group. It still emits the complete public report and preserves the final total/p95/path
ordering. For N spans and G distinct hotspot keys, grouping key-string allocations fall from 4N to
the required 4G output strings: exactly `4(N-G)` grouping allocations are removed. Ordered frame
membership changes from O(log F) per insert to expected O(1); per-group duration work changes from
full O(m log m) sorting plus finish scans to selection and input-pass accumulation.

Runtime status projection now classifies each detail once and returns borrowed text. For D detail
items it removes D detail clones plus the render/physics/animation clones, `D+3` temporary `String`
allocations, while preserving the priority order. Plugin actions now borrow up to six label/ID pairs
and return compact labels by reference. For A active actions it removes 2A source-field temporary
strings and A compact-label strings, up to 18 temporary strings per plugin; the required final row
action and button owners remain unchanged.

The related runtime implementation is
`zircon_runtime/src/core/runtime/diagnostics/profiling/hotspot.rs` (203 lines, 6,260 bytes after M1;
SHA256 `d90e9441c6991694848c83b88ac6edec47fa4c78c1dc5c06f30a3cfdfa3f3d17`). It is a related
algorithm owner, not part of the 3/3 pane conversion count. M1 does not solve the ordinary diagnostic
second pipeline, offscreen plugin nodes or duplicate final row models; those remain M2-M4.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add diagnostic pipeline, hotspot key/sort, row copy/build/reuse and visible-window counters; capture baseline. | scale-bound evidence on one fingerprint |
| M1 | Remove avoidable hotspot/status/action temporary allocation without changing ABI. | focused RED-to-GREEN contracts and behavior tests |
| M2 | Publish cached hotspot report and explicit debug-capture receipt. | one analysis per profile generation; ordinary temporary surfaces = 0 |
| M3 | Publish stable typed pane item generations and visible row windows. | node/build cost O(V), stable rows reused |
| M4 | Remove duplicate final DTO maps and all-offscreen node generation. | one item authority and virtualized presenter |
| M5 | Run managed scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full owner source review: passed, 3/3 Rust files.
- Payload producers, hotspot aggregation, plugin cache/row producer, debug schedule sections, final
  pane conversion and Unreal references: read.
- M1 source implementation: complete. Its allocation contract moved RED 3/3 to GREEN 3/3; the new
  percentile reference behavior test is present but cannot run while managed Cargo is unavailable.
- Combined related static performance contracts: passed, 19/19.
- Changed Rust `rustfmt` and scoped diff check: passed.
- M2-M5 remain structural work.
- Managed command
  `.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter hotspot -VerboseOutput`
  was rejected before Cargo launch with `cargo_session_not_executable`: Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`.
- WPR and RenderDoc remain pending a current-source launchable editor.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.
