# Runtime UI Drag Source Metadata Design

## Context

The Runtime UI component showcase already validates typed drop acceptance for `AssetField`, `InstanceField`, and `ObjectField`, but the dropped payload is still generated from fixed showcase demo data. The next slice connects the asset-browser source side so an `AssetField` drop can carry the actual source asset identity and display metadata.

The current codebase has three useful foundations:

- `zircon_runtime/src/ui/component/drag.rs` defines `UiDragPayloadKind` and `UiDragPayload`.
- `zircon_runtime/src/ui/component/state.rs` applies `UiComponentEvent::DropReference` and enforces the descriptor drop policy.
- `zircon_editor/src/ui/slint_host/asset_pointer/content/bridge.rs` already resolves asset content row pointer hits to an asset UUID, and `AssetWorkspaceSnapshot.visible_assets` already contains the locator, display name, kind, extension, and selection/preview state used by the asset browser.

## Goal

Dragging from the real asset browser into a Runtime UI reference well should produce a typed asset drop payload with source asset metadata. `AssetField` should accept the payload by policy, set its value from the asset locator, and retain enough source metadata for host projection, logs, and later Inspector integration.

## Non-Goals

- Do not connect scene tree, hierarchy, instance list, or object-reference sources in this slice.
- Do not implement OS-native cross-window drag/drop. This slice is the Slint host internal asset-row to Runtime UI reference-well path.
- Do not replace the existing `.ui.toml` showcase structure or make Slint a business source of truth.
- Do not implement full asset thumbnail rendering. The payload may carry metadata that later preview rendering can use.

## Recommended Approach

Extend the shared Runtime UI drag contract rather than keeping source metadata editor-only.

`UiDragPayload` remains the canonical input to `UiComponentEvent::DropReference`, but gains optional source metadata. The payload still has a simple accepted/rejected kind, while editor-specific details live in a generic metadata struct that is serializable and reusable by later hosts.

Expected shape:

```rust
pub struct UiDragPayload {
    pub kind: UiDragPayloadKind,
    pub reference: String,
    pub source: Option<UiDragSourceMetadata>,
}

pub struct UiDragSourceMetadata {
    pub source_surface: String,
    pub source_control_id: String,
    pub asset_uuid: Option<String>,
    pub locator: Option<String>,
    pub display_name: Option<String>,
    pub asset_kind: Option<String>,
    pub extension: Option<String>,
}
```

`UiDragPayload::new(kind, reference)` continues to create a payload without metadata. A builder-style method such as `with_source(...)` attaches metadata where a real source exists.

## Data Flow

1. The asset browser content surface receives pointer drag intent on an asset item row.
2. The existing asset content pointer bridge resolves the row to `asset_uuid`.
3. The Slint editor host looks up that UUID in the current `AssetWorkspaceSnapshot.visible_assets` for the same surface mode (`activity` or `browser`).
4. The host builds `UiDragPayload { kind: Asset, reference: locator, source: Some(...) }`.
5. When the pointer is released over a Runtime UI reference well, the reference-well action dispatches `UiComponentEvent::DropReference` with the real payload.
6. Runtime validation remains descriptor-driven: `AssetField` accepts `UiDragPayloadKind::Asset`; incompatible reference wells reject according to existing drop policy.
7. Accepted drops update the retained reference value from `payload.reference` and store the source metadata under the dropped property for event log/detail projection.

## Runtime Contract

`UiDragSourceMetadata` belongs in `zircon_runtime::ui::component` because the payload crosses the runtime/editor UI component boundary. It must stay editor-neutral:

- Use plain strings and optional fields.
- Do not depend on `AssetWorkspaceSnapshot`, Slint types, editor asset manager types, or filesystem paths.
- Serialize and deserialize with the rest of the component event contract.
- Keep policy checks based on `UiDragPayloadKind`, not metadata contents.

The source metadata is descriptive, not authoritative. The dropped `reference` remains the value written into the control.

`UiComponentState` will retain accepted drop metadata in a property-keyed map:

```rust
reference_sources: BTreeMap<String, UiDragSourceMetadata>
```

`DropReference` stores `payload.source` under the event property when metadata is present. `ClearReference` removes any retained source metadata for that property. A read accessor such as `reference_source(property)` exposes the metadata to editor projection without making callers inspect private state internals.

## Editor Host Integration

The editor host should reuse the existing asset pointer hit-test path:

- Extend asset content pointer dispatch with enough information to represent a drag source candidate, or add a narrow drag-start helper beside the existing click helper.
- Add a narrow drag-source helper beside the existing click helper rather than changing click dispatch semantics.
- Store the active drag payload in `SlintEditorHost` while the pointer is dragging.
- On reference-well drop, pass the active payload into the component showcase/runtime-ui drop action instead of the current fixed demo payload.
- Clear active drag state on drop completion, cancellation, or pointer release outside a drop target.

The source lookup should prefer the current snapshot item whose `uuid` matches the hit row. If a UUID cannot be resolved, no real drag payload is started; the host should not synthesize partial metadata from stale row text.

## Host Projection And Display

Reference wells continue to render generically. Projection adds a `drop_source_summary` string to `TemplatePaneNodeData`; the Slint template can display that string below the current reference value without knowing that it came from the asset browser.

The source summary format for this slice is `"{asset_kind}: {display_name}"` when both fields exist, falling back to `display_name`, then `locator`, then an empty string. If no source metadata exists, existing behavior remains unchanged and only accepted payload kinds or validation text are shown.

## Validation Plan

Runtime tests:

- `UiDragPayload::new` preserves the existing kind/reference contract with no source metadata.
- A payload created with source metadata serializes and round-trips.
- `AssetField` accepts an asset payload carrying metadata and stores the locator as its reference value.
- `InstanceField` or `ObjectField` still rejects an asset payload if its descriptor policy does not accept asset drops.

Editor tests:

- Asset content pointer drag source creation resolves a visible asset UUID to locator, name, kind, extension, and source surface metadata.
- Component showcase or template host drop dispatch uses the active real asset payload rather than the fixed `res://materials/runtime_demo.mat` payload when one exists.
- The retained projection exposes a source summary for a metadata-backed drop.
- Existing fixed showcase drop behavior remains available when there is no active real source payload.

Focused validation commands should include:

- `cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`
- `cargo test -p zircon_editor --lib slint_host --locked --jobs 1`
- `cargo check -p zircon_editor --lib --locked --jobs 1`

## Documentation Updates

Update `docs/ui-and-layout/runtime-ui-component-showcase.md` after implementation to record:

- the new `UiDragSourceMetadata` contract;
- the asset-browser source row to reference-well data flow;
- the tests that prove accepted and rejected metadata-backed drops.

## Open Decisions Resolved

- The first source is real asset-browser asset rows only.
- Source metadata belongs in the shared Runtime UI drag payload contract.
- The implementation should reuse the existing asset pointer bridge instead of adding a separate hit-test system.
- Slint remains a generic host; no showcase-specific business structure is added to `.slint`.
