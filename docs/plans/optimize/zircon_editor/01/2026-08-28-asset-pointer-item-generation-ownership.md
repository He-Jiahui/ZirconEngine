# Asset pointer item-generation ownership

Date: 2026-08-28

Status: static candidate; managed Rust and product profiling pending.

## Finding

`AssetWorkspaceSnapshot.visible_assets` is already an immutable `AssetWorkspaceItemGeneration`. It stores 64-item `Arc` chunks, exposes `len/get(index)`, owns the UUID index, and clones its products by `Arc`. Cloning the workspace snapshot therefore does not deep-copy every visible asset.

`AssetContentListPointerLayout::from_snapshot` discards that ownership advantage. For Activity and Browser independently it walks every visible item, clones `item.uuid`, and creates a new `Vec<String>`. `AssetContentListPointerBridge::sync` receives the completed vector before it can discover that the layout is unchanged; derived `PartialEq` then compares the vector again. The bridge only needs item count for arithmetic hit testing and the UUID at the one hit index.

This is a duplicate identity projection, not evidence that the retained `UiSurface` hit grid is rebuilding. Pane-size-only updates already use `sync_pane_size` and avoid this path. The remaining cost occurs whenever an otherwise stable asset snapshot is republished through `sync_asset_pointer_layouts`.

## Reference boundary

Unreal `SAssetView` retains one `FilteredAssetItems` collection of `TSharedPtr<FAssetViewItem>` and passes its address to list, tile, and column views through `ListItemsSource(&FilteredAssetItems)`. `RefreshList` requests view refresh on those consumers; it does not first create a second array of copied asset identifiers for pointer routing.

The corresponding Zircon boundary is to share the already-published item generation across paint, selection, drag, and pointer hit consumers. It must not add another UUID map or pointer-only string cache.

## Target algorithm

1. Replace `AssetContentListPointerLayout.item_ids: Vec<String>` with an `AssetWorkspaceItemGeneration` handle (or a narrower borrowed identity-generation handle owned by that same generation).
2. At sync, shallow-clone the generation's `Arc` products. Compare a stable identity/order product, not every UUID string. Item-payload-only replacement may update the retained handle without rebuilding pointer geometry.
3. Compute item count with `generation.len()` and resolve a hit with `generation.get(item_index)`. Clone only `item.uuid` when constructing the owned public route for that actual hit.
4. Keep Activity `visible_folders` as a separate projection in this slice. It is a distinct collection and is not justification for duplicating the much larger asset generation.
5. Keep `sync_pane_size` geometry-only. Hover, scroll, and pane-size changes must not clone or compare N UUID payloads.

One detail needs an explicit API rather than inference: `shares_items_with` currently compares the outer chunk table, so a one-item payload update changes identity even when UUID order is unchanged. The generation owner should publish a stable item-identity/order token, or expose pointer equality for its existing UUID-index/order product. The pointer bridge must consume that token; it must not reconstruct one.

## Complexity and deterministic pressure model

`tools/editor_asset_pointer_generation_pressure.py` models 100,000 visible assets, 256 Activity folders, 1,000 stable two-surface layout publications, and 10,000 real item hits.

Current item identity work:

- 200,000,000 UUID payload clones;
- 200,000,000 UUID identity comparisons after those vectors have already been built;
- 10,000 necessary hit-route UUID clones.

Target item identity work:

- zero stable-sync UUID payload clones and zero per-item equality comparisons;
- 10,000 `Arc` product handle clones plus 2,000 generation identity comparisons under the current five-Arc generation shape;
- the same 10,000 necessary hit-route UUID clones.

The deterministic item-identity operation ratio is 33,333.33x (400,000,000 versus 12,000). This is not a CPU timing claim and deliberately does not estimate variable UUID byte length, allocation cost, allocator contention, cache misses, or `Arc` atomic cost. Activity folder cloning remains visible and out of scope instead of being hidden in the item result.

Artifact: `E:\zircon-profiles\editor-asset-pointer-generation-pressure-20260828.json`; SHA-256 `CE3E16991314520D4333D0D78BE3700FCD23A7E07AEF26BCA77D3A475852114D`.

## Implementation result

`AssetWorkspaceItemGeneration` now exposes `shares_item_identity_with`, an O(1) pointer comparison
over its existing UUID-to-index product. `replace_existing_items`, `project_items` and incremental
projection already preserve that product when membership and order remain unchanged; a generation
built from a new item collection owns a new product. No pointer-only UUID map or hash is added.

`AssetContentListPointerLayout` now owns the published `AssetWorkspaceItemGeneration` handle instead
of a `Vec<String>`. Its equality compares pane/profile/view geometry, the intentionally separate
Activity folder projection, and the generation's order identity. Stable sync moves the latest shallow
generation handle into the bridge before returning without surface work. This avoids retaining stale
payload chunks while keeping the pointer authority generation unchanged for payload-only updates.

List and thumbnail arithmetic use `items.len()`. A real route resolves `items.get(item_index)` and
clones only that item's UUID into the owned public dispatch value. Pane-size, hover and scroll paths do
not project the generation. A lower Rust regression covers a one-item display-name update: item chunks
change, UUID identity remains shared, the bridge adopts the new handle, and surface authority does not
advance.

The context-menu route now consumes the same published UUID index (`selected_index -> get`) after a
row hit. It no longer scans every visible asset to recover the display name, so a right-click event is
expected O(1) in visible-asset count just like drag-start. Missing or stale indices retain the existing
diagnostic and abort behavior.

Static evidence:

- target source/pressure plus adjacent pointer contracts: 12/12 GREEN;
- drag/context UUID-index contract: 3/3 GREEN;
- Asset Browser selected-asset lookup contract and deterministic pressure model: 3/3 GREEN;
- four directly involved Rust files pass scoped `rustfmt --check` and `git diff --check`;
- refreshed artifact: `E:\zircon-profiles\editor-asset-pointer-generation-pressure-20260829.json`,
  SHA-256 `CE3E16991314520D4333D0D78BE3700FCD23A7E07AEF26BCA77D3A475852114D`;
- selection/context lookup artifact: `E:\zircon-profiles\editor-asset-selection-lookup-pressure-20260829.json`,
  SHA-256 `B39FF3490BDABBFD292EBD1F1702603F7BD52492F205E4615815F40401A51445`;
- model result remains 400,000,000 current identity units versus 12,000 target units (33,333.33x),
  while the 10,000 necessary routed-hit UUID clones remain unchanged.
- context-menu lookup contract is GREEN and contains no `visible_assets.iter()`/`.find()` fallback;
  drag and context-menu routes now share the same UUID-index authority.
- selection/context pressure model is 200,000,000 current scan units versus 5,000 target index
  units (40,000x); this is deterministic work, not CPU timing.
- full `test_*performance_contract.py` discovery refresh ran 1,472 tests with 1,459 passing;
  the 8 failures and 5 errors are pre-existing current-source split/path assertion drift outside
  this slice. Log: `E:\zircon-profiles\all-performance-contracts-20260829-asset-selection.log`,
  SHA-256 `C4545E1F51FAB4D354B18319EE190D8931D3D6DD92B49A6298D54DAA211C7BBB`.

## Acceptance

- 100,000-item, 1,000 stable-sync product profile reports zero UUID payload clones and zero N-item pointer identity comparisons.
- Activity and Browser pointer layouts share the published item identity generation; generation/order changes replace the handle exactly once.
- List and thumbnail arithmetic hit indices and resulting UUIDs remain identical to the workspace generation.
- Pane-size, hover, and scroll changes retain generation identity and do not publish item vectors.
- A one-item non-identity payload update does not force an N-item pointer projection; membership/order change does update the identity token.
- Route diagnostics and drag payloads still own the one selected UUID after the borrowed generation access ends.
- Context-menu display-name lookup is expected O(1) and preserves the stale-generation diagnostic.
- Asset Browser summary/detail selection lookup uses the published UUID and selected-index products,
  preserves first-in-display-order behavior, and performs no visible-row scan.
- Managed Rust plus a real 100,000-item Editor profile records CPU, allocator bytes/count, RSS, and input-to-present p50/p95/p99 before this item is promoted beyond `static_candidate`.

No Cargo validation was run in this slice. The generation owner is still an untracked shared module-split
leaf absent from HEAD, and the content bridge already contained external geometry-only patch work.
Managed validation must use a copy-complete support commit; this candidate must not be overlaid or
submitted alone. Product CPU, allocator, RSS and input-to-present latency remain unmeasured.
