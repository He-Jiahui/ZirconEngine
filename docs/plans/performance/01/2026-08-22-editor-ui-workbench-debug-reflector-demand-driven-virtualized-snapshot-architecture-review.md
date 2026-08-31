---
title: Editor workbench debug reflector demand-driven virtualized snapshot performance review
date: 2026-08-22
module: zircon_editor/src/ui/workbench/debug_reflector
priority: MVP-P2 diagnostic path
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate Widget Reflector and virtualized tree view
---

# Goal

Make UI reflection an explicitly requested, generation-owned diagnostic product. A normal editor
frame must not construct reflector data. A visible diagnostic consumer may capture one committed
all-node surface snapshot per changed generation, and its tree must materialize only visible rows.
Do not rebuild a synthetic surface merely to reflect the Runtime Diagnostics pane itself.

## Reviewed source

- folder: `zircon_editor/src/ui/workbench/debug_reflector`
- Rust files: 9/9
- lines: 1,923
- bytes: 68,867
- joined current source-bytes SHA256:
  `ec1c13fecd20d41953986b65773bf79b343abafda9a0831146d13169843b4eda`
- joined pre-change source-bytes SHA256:
  `e26b63abd223950f042e1aef359ff2ae19b90f5a4deb3c2d9200dc1b152e0197`
- owning commit before review: `08094b9b9e17f6c80372e15c17b01204038b305b`

| File | Lines | SHA256 |
| --- | ---: | --- |
| `export.rs` | 34 | `5d8e0ee08b1f400e9758757cdc0bc10f578ec303fc423d92ab9ab1aae62a72c8` |
| `mod.rs` | 19 | `f7f8b0b274a1c0b81709b4ad68881198706cf440fbf25c8f88001eaa42d39697` |
| `model.rs` | 502 | `21645e89881481fd2019fd8b55a86e4d166eeb62a97357b51dfbec5381f0b51b` |
| `overlay.rs` | 138 | `a2945b3cd3bd8780348ff9931c04212c738ddebf67353983af550673974866b3` |
| `schedule_sections.rs` | 192 | `206a4efc24d0022dc635f9c2a91348a1bc3ff262a4e5297e43b36cdce7e98e37` |
| `schedule_sections_tests.rs` | 119 | `8c8a86afc1d2747e5d9b9b955174f0935e307f3c4cd50667d23d8dcf4cef791e` |
| `selection.rs` | 26 | `c5b773bcf6c368600b1f780f5aa702127459d0b65dfafb6abbf9f35c4ec2181f` |
| `tests.rs` | 740 | `c273225655a63dba0ad5b981626f2d73ef8b200aefc18e3362c10261b9897f62` |
| `timeline.rs` | 153 | `f02ced940f306d68f71ca1beb62e612653baa23fb5183ab868e54e998cf9e740` |

All nine files were read in full. The production call chain was traced through pane payload build,
retained pane conversion, pane hit-artifact rebuild and retained-host full/shell-content recompute.
`export.rs`, `selection.rs`, `timeline.rs` and their consumers are test-only today; production cost is
owned by `model.rs`, `overlay.rs` and
`ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs`.

## Result

### Correct foundations to retain

Runtime diagnostics collection is already visibility-gated by active Runtime Diagnostics or
Performance Timeline content. Hidden normal rendering does not execute this reflector conversion.
Snapshot section previews for layout selections and canvas layers are bounded to eight records.
Overlay filtering is linear in the captured overlay records and preserves explicit category gates.

These gates are important, but they do not make the visible diagnostic path structurally sound.

### P1: one pane conversion performs up to three full-tree transactions

The production `PanePayloadBuildContext::with_active_ui_debug_snapshot` setter is `#[cfg(test)]`.
Therefore a production Runtime Diagnostics payload cannot carry the real committed UI debug
snapshot and `ui_debug_reflector_has_active_snapshot` is false. Retained conversion then:

1. projects the pane body and rebuilds its hit artifacts;
2. copies every current pane node into a new `UiSurface`, allocating path and metadata strings;
3. calls full `surface.rebuild()` and derives a wide debug surface snapshot;
4. formats that snapshot into reflector rows and recreates pane label nodes; and
5. rebuilds pane hit artifacts again.

This is the `PERF-MVP-143` structural failure. The dispatch/body surface is not the source UI tree,
so the extra work also reports the diagnostics pane reflecting itself instead of one authoritative
captured surface. It must not be optimized into a faster second truth.

### P2: the entire tree is eagerly converted to long owned strings and nodes

`EditorUiDebugReflectorModel::from_snapshot` formats one long label for every node. The retained
adapter then turns every label, detail and flattened section line into a real
`TemplatePaneNodeData`, regardless of viewport height. `section_display_lines` duplicates every
section string, and the payload adapter currently makes another three temporary full string-vector
copies before each line is copied again into its host node.

For N captured nodes and L total text bytes, model projection is at least O(N + L) allocation and
the host has O(N) retained rows even when only tens are visible. The current selected-node check also
builds a `BTreeSet` of all N node IDs only to answer one membership query, paying O(N log N) time and
O(N) memory even when no node is selected. Root marking performs `roots.contains` for each node,
which is O(NR) for R roots.

M1 removes the redundant payload vector clones and replaces the single-membership `BTreeSet` with a
borrowed scan. Per payload projection this removes three vector allocations and D + S + N temporary
string clones, where D, S and N are detail, section and node row counts. Selection validation removes
N ordered-set insertions and N stored IDs; when no node is selected it now visits zero snapshot
nodes. The final host-node text copy and eager all-row architecture remain and are not accepted by
this milestone.

### P2: overlay publication clones all enabled records

`primitives_from_snapshot` clones all matching captured primitives and creates owned primitives for
all enabled render-visualizer overlays. The default enables every category. This is acceptable only
for an explicit capture generation with bounded record counts; it must not be rebuilt by UI refresh
frequency or copied independently for multiple consumers.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Widgets/SWidgetReflector.cpp`
- `dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Widgets/SWidgetReflectorTreeWidgetItem.cpp`
- `dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Models/WidgetReflectorNode.cpp`

Unreal binds `SWidgetReflector` to an `STreeView` using `TreeItemsSource`, `OnGenerateRow` and
`OnGetChildren` (`SWidgetReflector.cpp:1079-1088`). Rows are `SMultiColumnTableRow` instances and
generate cells per requested column (`SWidgetReflectorTreeWidgetItem.cpp:34-66`), so the view owns
visible-row materialization rather than a flat list of every preformatted row.

Unreal's ordinary `Tick` only services an explicitly pending delayed snapshot
(`SWidgetReflector.cpp:1394-1405`). Live tree replacement occurs when
`SetWidgetsToVisualize` receives a picked widget path (`1425-1438`). A full snapshot is initiated by
the Take Snapshot action, then one captured tree is assigned and the tree view is refreshed
(`1947-1969`, `2030-2055`).

The transferable invariant is demand-driven capture plus a persistent tree item model. Opening or
refreshing a diagnostic pane is not itself a reason to run another complete UI surface pipeline.

## Target architecture

1. Runtime UI publishes one immutable `Arc<UiSurfaceDebugSnapshot>` with surface ID, committed
   generation, capture options and bounded detail sections. Hidden diagnostics publishes no wide
   snapshot and does no reflector work.
2. Runtime Diagnostics subscribes by generation. The same generation is shared by reflector tree,
   details, overlays, JSON export and timeline; repeated presentation refreshes are no-ops.
3. Replace flat `Vec<String>` node labels with a typed hierarchical item source keyed by stable node
   ID and parent/child ranges. Format cells lazily for visible rows; selection and expansion do not
   rebuild unaffected rows.
4. Remove `runtime_diagnostics_debug_surface_frame`. Pane dispatch hit artifacts remain a normal
   pane concern and are never used as the all-node reflector source.
5. Publish overlay records by shared generation handle. Apply category filters and spatial/visible
   bounds without cloning every disabled or off-screen record.
6. Keep the current JSON schema and selected-node/details text as compatibility tests only until the
   typed item source is accepted, then delete redundant flat projection APIs in one hard cutover.

Complexity target:

- diagnostics hidden: zero snapshot, reflector, overlay and row work;
- unchanged visible generation: O(1) handle comparison, zero rebuild/format/allocation;
- changed generation: one O(N) capture owned by Runtime, not another Editor surface rebuild;
- display: O(V) row/cell materialization for V visible rows, O(1) indexed selection lookup;
- overlays: O(P_enabled_visible) publication with bounded records and one shared owner.

## Instrumentation and acceptance

Add counters before the structural cutover:

| Evidence | Target |
| --- | --- |
| source surface rebuilds per editor recompute | 0 extra for diagnostics |
| synthetic diagnostics surfaces/rebuilds | 0 |
| snapshot builds per surface generation | at most 1 when subscribed |
| reflector model/row/string builds per unchanged generation | 0 |
| visible rows materialized | proportional to V, not N |
| copied/cloned text bytes | 0 on unchanged generation; no full-tree host copy |
| pane hit-artifact rebuilds per conversion | at most 1 |
| overlay records/bytes and dropped-by-budget count | bounded and explicit |

Matrix: hidden/visible diagnostics; capture off/on; 1/100/1,000/10,000 nodes; depths 1/16/64;
viewports `640x420/1280x720/1920x1080`; stable 300 frames; selected node, expand/collapse, search,
overlay categories and export. Report median/p95/max CPU, surface/snapshot/row counts, allocation and
clone bytes, RSS, input-to-paint latency and package energy on one source/executable fingerprint.

Use the current editor profiler and WPR/ETW with all targets and artifacts on D/E/F. RenderDoc is
only for final overlay draw-call, overdraw and pixel parity on a launchable current-source GPU path;
it cannot prove removal of CPU surface rebuilds or string allocations.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Capture current visible/hidden diagnostic counters and WPR baseline. | source-bound trace artifacts |
| M1 | Remove redundant full string-vector clones and one-query node-ID index. | static contracts plus focused tests |
| M2 | Publish and subscribe to one generation-owned real surface snapshot. | synthetic surface/rebuild count = 0 |
| M3 | Replace flat labels with a typed virtualized tree item source. | materialized rows proportional to V |
| M4 | Share bounded overlays/export/timeline from the same generation. | unchanged generation clone/build = 0 |
| M5 | Run WPR, power, product interaction and RenderDoc pixel acceptance. | quantified before/after evidence |

## Validation state

- Full folder source review: passed, 9/9 files.
- Related production call chain and Unreal reference functions: read and recorded.
- M1 source implementation: complete. The RED-to-GREEN static performance contract is 2/2; targeted
  `rustfmt`, source accounting and `git diff --check` pass.
- Managed Cargo and current-source profiler: pending while shared Cargo lanes are active.
- M0-M5 dynamic and product acceptance: pending.

The folder remains in `pending.md` until M0-M5 pass on one fingerprint. Static review and a simple
M1 allocation reduction must not be recorded in `review.md` or claimed as end-to-end performance
acceptance.
