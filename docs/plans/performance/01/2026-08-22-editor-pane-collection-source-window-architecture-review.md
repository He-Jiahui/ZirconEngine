---
title: Editor pane collection source-window performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/{collection_fields/**,collection_projection/**,collection_window.rs}
priority: MVP-P0 shared editor lists, tables and collection fields
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate SListView visible-item generation and row reuse
---

# Goal

Apply list/table viewport and page selection at the source item generation before cloning, parsing,
validating or constructing host rows. Stable collections must retain typed item identities and row
widgets; scroll, page, selection and value edits must patch only the visible or addressed rows.

## Reviewed source

- owner files: array/map field projection and validation, collection projection/model,
  virtualization/pagination metadata and visible-window helper
- Rust files: 15/15
- current lines: 737
- current bytes: 26,302
- joined current source-bytes SHA256:
  `b2b5a99d19c9d294abed08fc83e72c385aea56343c4ca5b35782d357ef14818d`
- pre-M1 lines: 732
- pre-M1 bytes: 26,153
- joined pre-M1 source-bytes SHA256:
  `d5af24174c152ecaf4b1e124120411c7936dc081f00f9de34649432e2aa6dcd7`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `collection_fields/mod.rs` | 26 | 678 | `7484f3191e3637a21911648c414fb26001c4200448207168b1288fae2199f398` |
| `collection_fields/array.rs` | 88 | 3,596 | `39591bdd4e45467de01933aa63563c00d97182f04a74de77d6c4aa0810432027` |
| `collection_fields/empty.rs` | 33 | 1,100 | `0396a1499f3f49db2790ce8df4f6c7ae8dd2d34e1e4313f976838304df7353d6` |
| `collection_fields/map.rs` | 73 | 3,095 | `200aa21126393c8f0b71afd678b5a30bb9dc18e898388bb585c504cec3d330ba` |
| `collection_fields/roles.rs` | 44 | 1,503 | `8280e8eae7ac187c9ac46243dee1a0816c0331b9a0bd48a6c04a0c95d72db43c` |
| `collection_fields/type_tokens.rs` | 21 | 740 | `b4ae86bc94ccbdfdf32a38f2079787538cd72b07a130e584901ba1f97d34bfc4` |
| `collection_fields/validation.rs` | 121 | 4,330 | `be0963a9a87842aefbdaf43af0305e62c0022db8353ee3da6bb1e6b540360b13` |
| `collection_fields/tests.rs` | 72 | 2,664 | `7c22fad2521230693a46b07788793bedb004cfee05cadc8381b7a2658e0bb52a` |
| `collection_projection/mod.rs` | 36 | 1,431 | `a6d64b723fb82daba1542a45d494a73f96d60cf614affc98c762a9e955224420` |
| `collection_projection/model.rs` | 17 | 871 | `615c8d387bf2081f379247479da62e935e2863f5f389530aba51cc304d755c49` |
| `collection_projection/items.rs` | 87 | 2,855 | `8ce76e6326b2e728693c720db474fd25fc63e94c8174d61b9eee3e10e3b2b422` |
| `collection_projection/fields.rs` | 14 | 510 | `bcf5cc7f47b410718ed3da596d4554ca7538f74f54a1571f60a0043da27a3180` |
| `collection_projection/virtualization.rs` | 45 | 1,392 | `10560b916914006bfcb9117cfac18a3b994832b086e56ec6cd073801ea25372c` |
| `collection_projection/pagination.rs` | 33 | 900 | `081f60ef1e1a4cc3b3c5ec1c451dab20e2c425a0d7a079cb97d3e9e1863736a9` |
| `collection_window.rs` | 27 | 637 | `5745f349db6781dcc0b69280c3851097b394075a88c47224c0b0f365ec471b00` |

All fifteen files were read in full. Production ownership was followed through raw TOML-to-`UiValue`
conversion, component collection tests, table action identity, hit testing and keyboard window
selection. These related files are not counted in the 15/15 owner total.

## Existing foundations to retain

Virtualization metadata carries item extent, overscan, total count, visible start and visible count.
Visible ranges clamp negative inputs and use saturating arithmetic. Typed table rows preserve the
original source index and scalar identity kind/text; hit testing returns that source index, and table
actions validate the identity against the current source snapshot. These contracts are correct and
must survive earlier source windowing.

## Structural findings

### P0: virtualization is applied after full item and row materialization

`projected_collection_items` first clones every string option into a `Vec<String>` and only then
passes that vector to `visible_collection_items`. Typed table rows similarly scan all source rows,
convert every scalar identity, clone identity/label strings and build every wide row before slicing
the completed vector. For N valid source rows and a visible-plus-overscan window W, allocation and
row construction remain O(N) even though the final host model contains W.

Apply skip/take to the filtered source iterator and materialize only the window. Preserve filtering
before windowing and preserve original source indices. This is behavior-preserving M1 work; the
stable indexed source in M2 must reduce late-window scans from O(start + W) to O(W).

### P0: ArrayField and MapField deep-copy and validate every entry

Both field projectors call `UiValue::from_toml` on the complete collection. That conversion
recursively clones array elements, map keys and nested values. They then build all field rows,
format IDs/payloads, derive display text, validate types and clone action IDs. Explicit
virtualization metadata is not passed to collection-field projection, so large editable arrays/maps
cannot avoid offscreen work.

Publish typed editable collection generations at the component/data owner. The generation must hold
declared type descriptors, stable row IDs, values and validation. The presenter receives a visible
index and constructs only visible field rows; edits patch one stable item.

### P1: declared type classification is repeated per field row

Array rows lowercase and classify the same declared element type separately for role and validation.
Map rows repeat key and value normalization for key role, value role and both validations. Action IDs
are also cloned into every field. Normalize declared key/value descriptors once per schema generation
and share typed action descriptors across rows.

### P1: pagination is metadata without an explicit item-generation receipt

Page index/size/count/total are copied to the host row, but collection items/rows/fields are not
selected by page metadata. This may be correct only when the producer already publishes the current
page. The contract does not carry a page-generation/source receipt to prove that assumption. Define
producer-owned page generations explicitly; do not blindly slice again in the presenter and risk
double pagination.

### P2: window arithmetic is bounded and correct

Negative starts/counts/overscan and integer overflow are handled with clamp/saturating arithmetic.
Keep one shared arithmetic helper, but make it collect from a lazy iterator or return source bounds.
The bottleneck is the completed `Vec` input, not the arithmetic.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Views/STableViewBase.cpp`

Unreal `SListView` calculates a start index from the scroll offset, generates rows only until the view
area is filled, reuses an already generated widget by item identity, and releases widgets no longer
seen (`SListView.h:978-1067`, `1524-1690`). `STableViewBase` coalesces a pending layout refresh rather
than reconstructing immediately (`STableViewBase.cpp:1393-1406`).

The transferable invariant is viewport selection before row generation plus retained row identity.
Slicing a fully materialized vector is not virtualization.

## Target architecture

1. Publish `CollectionSourceGeneration` with stable item IDs, typed values/schema and exact source/
   filter/sort/page receipts.
2. Derive visible/page indices over that generation before display text, validation, action and host
   row materialization.
3. Share the visible index across paint, hit, keyboard, accessibility and profiling. Preserve source
   index and stable identity separately from visible index.
4. Publish editable array/map field rows by stable ID. Normalize type and action descriptors once;
   patch one value/validation row on edit.
5. Make pagination producer-owned and receipt-backed. The presenter consumes the current page
   generation and must not infer whether input is pre-paged.
6. Reuse retained row widgets across scroll/page changes and delete full intermediate vectors.

Complexity targets:

- stable collection refresh: O(1), zero item/row construction;
- scroll/page projection: O(V), where V includes overscan;
- one field edit/validation: O(1) lookup plus one row patch;
- offscreen string/value/row/action allocations: zero;
- paint/hit/keyboard/accessibility visits: O(V), one visible index.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| source items scanned vs rows materialized | materialized O(V); target scanned O(V) |
| TOML/UiValue nodes and bytes cloned | offscreen = 0 |
| type normalizations/validations/action-ID copies | once per schema or changed row |
| visible/source/page indices consumed by each UI path | one shared index |
| stable rows reused/released | stable = all reused |
| main-thread CPU and input-to-paint latency | report median/p95/max |

Matrix: source items 0/1/1,000/10,000/100,000; visible rows 0/1/8/32; overscan 0/2/32;
valid/invalid filtered rows; array/map nested value depth 0/1/8; page sizes 1/32/1,000; stable
refreshes and scroll/page/selection/edit/schema changes. Capture scans, clones/bytes, rows, type work,
allocations, visible visits, CPU, latency, RSS and package energy on one source/executable fingerprint.

Use managed Windows validation and WPR/ETW with artifacts only on D/E/F. RenderDoc is reserved for
current-source pixel/draw parity; it cannot validate source windowing or row ownership.

## M1 result

The shared collection window now collects from any lazy iterator. String items are filtered from the
source TOML array and windowed before owned `String` collection. Typed table rows retain enumeration
over the original source array, filter invalid identities, and window that iterator before row DTO
construction. This preserves filtering-before-window semantics and original `source_index` values.

For N valid items, visible-plus-overscan window W and visible start S, required owned string/row DTO
construction falls from N to W and source visits fall from N to `min(N, S + W)`. Mixed invalid rows
retain the prior valid-item window semantics and stop once the requested valid window is collected.
M1 does not virtualize editable ArrayField/MapField rows or retain an indexed source, so M2-M4 remain
required for stable O(1) refresh and O(W) late-window access.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add scan/materialize/clone/type/action/visible-visit counters; capture baseline. | collection scale evidence |
| M1 | Apply the existing visible window to lazy item and typed-row iterators. | focused RED-to-GREEN contract and behavior parity |
| M2 | Publish stable typed collection/page generations and source indices. | stable = 0; scroll O(V) |
| M3 | Virtualize editable ArrayField/MapField rows and cache schema/action descriptors. | offscreen field work = 0 |
| M4 | Share visible indices across paint, hit, keyboard, accessibility and profiling; remove full vectors. | one window authority |
| M5 | Run managed scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full owner source review: passed, 15/15 Rust files.
- TOML/UiValue conversion, collection behavior tests, table action identity, hit/keyboard consumers
  and Unreal references: read.
- M1 source implementation: complete. Its focused source-window contract moved RED 3/3 to GREEN 3/3.
- Combined owned performance contracts: passed, 31/31. Related fixture/row-patch contracts: passed,
  5/5. Changed Rust `rustfmt` and scoped diff check: passed.
- Managed Rust behavior tests and M0-M5 remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; the focused command
  was rejected before Cargo launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a current-source launchable editor.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.
