---
title: Editor inspector, hierarchy and console retained-item generation performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/pane_data_conversion/{inspector_fields.rs,inspector_projection.rs,hierarchy_projection.rs,console_projection.rs,inspector_pane_tests.rs}
priority: MVP-P0 inspector, scene hierarchy and console interaction
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate details/outliner tree rows and output-log marshaller
---

# Goal

Keep inspector properties, hierarchy items and console messages as stable typed generations from
snapshot through retained presentation. Selection, filter, scroll and appended-log changes must
patch exact rows; only visible rows may become paint/hit widgets; raw field strings must not be
reclassified and re-encoded on every host conversion.

## Reviewed source

- owner files: inspector field/node construction, inspector projection, hierarchy projection,
  console projection and inspector pane behavior tests
- Rust files: 5/5
- current lines: 1,633
- current bytes: 57,697
- joined current source-bytes SHA256:
  `f9db737e991335dc9607fb126ada305e050655335028e6bfedf4d439d01a9fe6`
- joined pre-M1 source-bytes SHA256:
  `512d03ec6c415d87bd3d31a3a1cae40a443c9acfd22e1cb3f8b583618db72df7`
- owning commit before review: `4d5f52aa2b76a3a877aabdd47b01a98dcdd59493`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `inspector_fields.rs` | 735 | 23,739 | `3de0c31f218cc853240f5fbc4530bbbdbffb45d0703f3b3a3225cdf4edab0385` |
| `inspector_projection.rs` | 168 | 6,356 | `637a0043396ba89ea150112d874a8e3e001ce8a6f71d6b62baa9af030aa14cb1` |
| `hierarchy_projection.rs` | 162 | 5,540 | `7f8cbd735f076a11e25d8dd312e8ae65c1761cb163d5904971f3d358aaf1ca1e` |
| `console_projection.rs` | 294 | 11,266 | `db789f2a8c7c07341ef74a6fdd1c56b7d224a001ffe74341ba0176fd9e279639` |
| `inspector_pane_tests.rs` | 274 | 10,796 | `363fdce5e4629fb65a6fe14d4f994641a30bad835185fdc2a214e30436b558fb` |

All five files were read in full. Production ownership was followed through pane payload builders,
console snapshot capacity, console paint metadata/visible rows, template surface/hit construction,
hierarchy visible-row painting, plugin inspector snapshots and final pane rebuilding. These related
files are not counted in the 5/5 owner total.

## Existing foundations to retain

Console text, severity and jump arrays cross the pane payload boundary as shared `Arc` owners and are
bounded to 256 logical lines. Console paint and pointer hit paths calculate exact visible line ranges.
Hierarchy painting similarly iterates a clipped visible row range even at 10,000 items. Plugin
inspector payloads already carry frozen editor-kind and asset-reference metadata. These are strong
foundations; final conversion must consume them directly instead of recreating parallel rows.

## Structural findings

### P0: inspector selection executes two presentation systems and a third field expansion

The template path projects the pane document, builds a shared runtime surface, computes full layout
and converts a host model. It then reads payload attributes, copies every plugin component/property
into intermediate `InspectorPluginComponentViewData`, and calls `inspector_field_nodes` to generate a
second set of editable visual nodes. The fallback path copies all basic strings and deep-clones all
plugin components into `InspectorVisualFields` before generating the same nodes.

Each selection generation therefore carries snapshot, payload, intermediate view data, runtime host
model and final wide nodes. This is not a retained property editor. The inspector needs one stable
typed property tree with category/component/property identities and small geometry/state patches.

### P0: plugin property editor metadata is copied upstream, then discarded and rediscovered

The payload preserves `field_editor_kind` and asset-reference markers from the frozen extension
field editor. Final inspector projection drops those fields, lowercases the raw `value_kind` for every
property, matches a hard-coded numeric token set, parses the value again and formats control/action
IDs from raw field IDs. This duplicates plugin schema work and can select a different editor from the
extension-owned classification.

Publish the frozen field-editor descriptor with the stable property item. Host rows must dispatch by
that descriptor and patch only value/validation state. Raw type-string classification in the final
presenter must be deleted after parity tests.

### P0: console visible-row culling begins after all line nodes are materialized

The source correctly caps output at 256 logical lines. Final conversion nevertheless splits all
text, clones a wide prototype once or twice per line, formats IDs/actions, inserts up to 512 nodes and
then stores `ConsoleOutputPaintMetadata`. Paint and pointer paths use the metadata to visit only
visible lines, but pane hit-artifact rebuilding still sees the complete model and can build a generic
`UiSurfaceFrame` for every dispatchable jump row. A separate profiling route consumes that surface,
so deleting it locally would be a behavior regression.

Make console line items shared data and generate a visible node window before paint/hit/profiling.
One visible-row index must serve all three consumers. Appending one line must add one item and evict at
most one capacity row, not rebuild every retained line node.

### P0: hierarchy paint is virtualized but every accepted generation remaps the full tree

Hierarchy drawing visits only the clipped visible range, but both legacy and template conversion map
every scene item to a new host `SceneNodeData` model. Search text is patched into the template while
the full hierarchy model remains separate. Selection-only changes replace all IDs/names/depth rows
instead of one selected-state row.

The scene snapshot must publish a persistent hierarchy item generation keyed by entity ID. Filtered
and expanded visible order should be an index over those items; selection/rename patches replace
only addressed rows and share the same visible range with paint, pointer and accessibility.

### P1: inspector node generation repeats scans and temporary formatting

Plugin components/properties are traversed once to calculate panel height and again to build nodes.
Each property lowercases `value_kind`, repeatedly encodes field IDs, and uses a temporary formatted
string for every non-alphanumeric character. Component diagnostics and final field nodes clone the
same messages/values into multiple wide fields. M1 can remove classification/encoding temporaries;
one-pass layout requires the retained property-tree design.

### P1: template control state performs repeated linear searches

Hierarchy searches the template three times. Console source filter, level filter and counts perform
14 separate `find` scans. Current builtin templates are small, so this is not the P0 scale issue, but
one borrowed pass is simpler, deterministic and safe M1 work.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/SDetailsViewBase.h`
- `dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/SOutlinerTreeView.h`
- `dev/UnrealEngine/Engine/Source/Developer/OutputLog/Private/SOutputLog.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`

Unreal DetailsView owns stable `FDetailTreeNode` references in an `STreeView`, generates rows through
`OnGenerateRowForDetailTree`, releases rows when they scroll out and preserves expansion state
(`SDetailsViewBase.h:47`, `375-383`, `533-537`). Scene Outliner directly derives its view from
`STreeView<FSceneOutlinerTreeItemPtr>` and supports invalidation
(`SOutlinerTreeView.h:10-24`). `SListView` starts at the scroll offset, generates until the view is
filled and reuses an already generated row (`SListView.h:1520-1628`, `1664-1688`).

Unreal OutputLog retains message and pending-message arrays in `FOutputLogTextLayoutMarshaller`,
tracks a next-pending index, caches message counts and appends pending messages to its text layout
instead of reconstructing independent wide UI nodes for the complete log
(`SOutputLog.h:568-625`). The transferable invariant is stable typed items plus incremental/visible
presentation, not repeated DTO and widget expansion.

## Target architecture

1. Publish stable `InspectorPropertyItem`, `HierarchyItem` and `ConsoleMessageItem` generations with
   immutable IDs and exact source receipts. Cross the host boundary with shared row owners.
2. Make inspector components/categories a retained tree. Preserve frozen editor descriptors,
   expansion/filter state and row widgets; value changes patch exact property IDs.
3. Make hierarchy filtered/expanded order an index over shared entity items. Selection and rename
   are row patches; scroll changes only the visible window.
4. Replace console text splitting/node cloning with an append/evict message generation and one
   visible-row index shared by paint, hit, profiling and accessibility.
5. Split semantic items from geometry/style/state rows. Width/theme/scroll changes must not reparse
   field types, IDs or console text.
6. Delete intermediate inspector view DTOs, full hierarchy remaps, all-line console nodes and raw
   presenter type classification after every consumer uses the retained item protocol.

Complexity targets:

- stable selection/catalog/log generation: O(1), zero row/node reconstruction;
- inspector value edit: O(1) item lookup plus one row patch;
- hierarchy selection/rename: O(1) lookup plus changed rows, not O(N) remap;
- console append at capacity: O(1) item append/evict plus visible-window change;
- scroll-only paint/hit/accessibility: O(V), where V is visible rows;
- final duplicate inspector/hierarchy/console row models: zero.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| inspector payload/view/host rows and bytes | one shared item generation |
| field kind classifications and ID encodes | once per schema generation; stable = 0 |
| hierarchy rows copied/selected rows patched | copied stable = 0; selected = changed rows |
| console lines split/nodes built/surface rows | append/visible changes only |
| paint/hit/profiling/accessibility row visits | O(V), same visible index |
| main-thread CPU and input-to-paint latency | report median/p95/max |

Matrix: inspector properties 0/1/100/1,000/10,000 with value/selection/schema/filter/expand changes;
hierarchy items 0/1/1,000/10,000/100,000 with selection/rename/filter/scroll; console lines
0/1/256 with append/filter/source/scroll and 0/100% jump rows; visible rows 1/8/32; stable operations
1/1,000. Capture rows/bytes, classifications, ID encodes, splits, surface builds, allocations,
main-thread CPU, latency, RSS and package energy on one source/executable fingerprint.

Use managed Windows validation and WPR/ETW with artifacts only on D/E/F. RenderDoc is reserved for
current-source pixel/draw parity; these CPU/item-ownership bottlenecks require counters and ETW.

## M1 result

Inspector numeric field classification now uses allocation-free ASCII-insensitive comparisons rather
than allocating a lowercase `String` for every property. Component-key escaping writes hexadecimal
code points directly into the destination key instead of allocating one temporary formatted string
per non-alphanumeric character. For P projected properties and E escaped characters this removes P
classification strings and E formatting strings while preserving all existing token and collision
semantics.

Hierarchy template state now visits T template rows once instead of running three separate searches,
reducing row visits from at most 3T to T. Console count, level filter and source filter state now
share one node pass instead of 14 separate searches, reducing template-row visits from at most 14T
to T while matching each visited control once. For normal aligned console payloads the line count reuses
`levels.len()` in O(1); the empty/no-severity compatibility path retains the original byte scan.

The M1 changes preserve pane DTOs, stable IDs, filter/count state, 256-line capacity, severity rows,
jump actions and empty-history behavior. They do not solve duplicated inspector/hierarchy item
models or all-line console nodes; those remain M2-M4.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add item-generation, row copy/patch, classification/encode, console split/node/surface and visible-visit counters; capture baseline. | scale-bound evidence |
| M1 | Collapse template state scans and remove field classification/encoding temporary allocation. | focused RED-to-GREEN static/behavior contracts |
| M2 | Publish retained typed inspector/hierarchy/console item generations and receipts. | stable rows/bytes = 0 |
| M3 | Share visible-row indices across paint, hit, profiling and accessibility. | every consumer O(V) |
| M4 | Remove intermediate DTO maps, raw type rediscovery and full console-node expansion. | one item authority |
| M5 | Run managed scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full owner source review: passed, 5/5 Rust files.
- Payload builders, console capacity/paint/hit/surface, hierarchy visible paint and Unreal references:
  read.
- M1 source implementation: complete. Its projection contract moved RED 4/4 to GREEN 4/4.
- Changed Rust `rustfmt` and scoped diff check: passed.
- Managed Rust behavior tests and M0/M2-M5 remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`.
- WPR and RenderDoc remain pending a current-source launchable editor.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.
