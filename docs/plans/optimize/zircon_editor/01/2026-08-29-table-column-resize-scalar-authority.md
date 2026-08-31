# Table column resize scalar authority

Date: 2026-08-29

Status: design ready; shared Runtime binding and surface owners remain read-only until their
current-source ownership is reconciled.

## Finding

`UiSurface::apply_default_table_column_width` currently handles every pointer move by reconstructing
both the `column_widths` map and the `columns` array from template metadata. Each aggregate is then
submitted through a full `UiPropertyMutationRequest`, so a single scalar width change pays repeated
TOML-to-`UiValue` conversion, map/array allocation, field matching and binding-report work. The
component reducer already updates both aggregate values in place, which proves that the semantic
operation is one field delta; the surface path is losing that locality before it reaches the reducer.

This is a separate hotspot from hit-grid or layout reconstruction. The resize route correctly captures
the pointer and emits live drag events; removing those events would hide latency rather than improve
the interaction.

## Reference boundary

Unreal `SHeaderRow` stores one `FColumn` per header slot. During splitter resize, the slot's
`FColumn::SetWidth` callback receives the new scalar width directly. The header does not rebuild an
array of serialized column descriptors for every mouse sample. The view can later request a list refresh
or persist the column model at an explicit boundary.

Zircon should preserve one Runtime semantic authority while exposing a field-level mutation boundary:
the live geometry and component state consume `(owner_id, column identity, width)`, and aggregate
`column_widths`/`columns` projections are compatibility products, not the event hot-path authority.

## Target algorithm

1. At resize press, resolve the owner, field and stable column identity once. Store a typed drag token
   containing the owner, field, start/min width and a metadata/schema generation. Do not store a copied
   map or array.
2. At each move, derive the clamped scalar width from the token and pointer delta. If unchanged,
   return without mutation. Otherwise apply one field-level `column_width` transaction that updates
   the component reducer and marks only the owner plus affected visible geometry dirty.
3. The transaction emits a binding update whose source/target carries a canonical nested field path
   (for example `column_widths.<field>` and the corresponding column width slot). Existing aggregate
   consumers receive a typed delta or borrowed view; no generic serializer is called on the move path.
4. At release, publish the final scalar receipt. Only consumers explicitly requiring legacy aggregate
   snapshots receive one bounded compatibility projection. That projection is at most once per drag,
   never once per pointer move, and its cost is visible in a counter.
5. If the schema/metadata generation or field identity no longer matches the token, reject the delta,
   emit a typed diagnostic and request a fresh layout. Never scan for a similarly named column and risk
   updating the wrong slot.
6. Geometry invalidation stays local. A width change may invalidate the table's track metrics and
   visible cells, but it must not mark unrelated panes or rebuild the whole UI surface.

## Complexity model

`tools/ui_table_column_resize_scalar_pressure.py` models 256 columns, 2,000 pointer moves, eight
metadata entries per column and one compatibility flush. The current aggregate route is estimated at
15,620,000 entry/transaction work units. The scalar route is 12,864 units: one schema build, three
scalar operations and one property transaction per move, plus one bounded compatibility projection.
The operation-count reduction is greater than 1,200x in the default model. This is not CPU timing,
allocator, layout, render or input-to-present evidence.

## Ownership and implementation order

| Step | Work | Gate |
| --- | --- | --- |
| 1 | Add a Runtime interface-level field-path/column-width delta contract and lower reducer regression. | aggregate values remain semantically identical after a scalar delta |
| 2 | Add a surface-owned typed resize token with schema/identity validation. | pointer moves perform no aggregate decode/encode or field scan |
| 3 | Route live geometry and component state through the scalar transaction. | only table owner/visible cells become dirty; drag feedback remains live |
| 4 | Add one release-boundary compatibility projection for legacy aggregate consumers. | flush count is bounded and observable |
| 5 | Remove old per-move aggregate mutation helpers. | no fallback serializer or duplicate scalar authority remains |

## Acceptance

- Lower Rust tests prove scalar width updates preserve both aggregate semantics, min-width clamping,
  unchanged-width no-op and typed identity/schema invalidation.
- Source contracts prove no `UiValue::from_toml`/aggregate rebuild occurs in the pointer move branch,
  and that release-only compatibility projection is bounded.
- Existing table resize, sort, scroll, capture and binding-route tests remain behaviorally equivalent.
- A 10,000-cell table records visible-cell/track invalidation counts rather than full-tree rebuilds.
- Managed current-source validation records CPU, allocation bytes/count, RSS and input-to-present
  p50/p95/p99 for continuous resize before promotion beyond `design_ready`.

No production code is changed by this design record. The current Runtime table files have shared dirty
ownership and are intentionally excluded until the field-path contract owner provides a copy-stable
boundary.
