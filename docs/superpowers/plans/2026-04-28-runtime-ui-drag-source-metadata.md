# Runtime UI Drag Source Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Carry real asset-browser source metadata through Runtime UI drag/drop payloads into `AssetField` reference wells.

**Architecture:** Extend the shared `zircon_runtime::ui::component` drag payload with optional source metadata, retain accepted source metadata in `UiComponentState`, and let the editor Slint host build real asset payloads from the existing asset content pointer bridge. Slint remains a generic host: it receives a projected `drop_source_summary` string rather than asset-browser-specific logic.

**Tech Stack:** Rust, Slint, TOML-backed Runtime UI templates, `cargo test`, `cargo check`.

---

## Execution Notes

- Work in the existing `main` checkout. Do not create worktrees or feature branches.
- Do not create git commits unless the user explicitly grants commit permission during execution. Each task ends with a checkpoint instead of a commit.
- The worktree is already heavily dirty. Do not revert unrelated files.
- `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs` is already above the large-file threshold. This plan only adds `drop_source_summary` pass-through there. The next projection expansion should extract Runtime UI component projection into a focused child module before adding more behavior.

## File Structure

- Modify `zircon_runtime/src/ui/component/drag.rs`: define `UiDragSourceMetadata` and attach it to `UiDragPayload`.
- Modify `zircon_runtime/src/ui/component/mod.rs`: re-export `UiDragSourceMetadata`.
- Modify `zircon_runtime/src/ui/component/state.rs`: retain property-keyed source metadata for accepted drops.
- Modify `zircon_runtime/src/ui/tests/component_catalog.rs`: cover metadata roundtrip, accepted drop retention, and clear behavior.
- Modify `zircon_editor/src/ui/template_runtime/showcase_demo_state.rs`: pass full `UiDragPayload` through showcase drop events and project `drop_source_summary`.
- Modify `zircon_editor/src/tests/host/template_runtime/component_showcase_state.rs`: cover metadata-backed showcase drops.
- Create `zircon_editor/src/ui/slint_host/app/asset_drag_payload.rs`: convert visible asset snapshot rows into runtime drag payloads.
- Modify `zircon_editor/src/ui/slint_host/app.rs`: register the helper module and store the active asset drag payload.
- Modify `zircon_editor/src/ui/slint_host/app/asset_content_pointer.rs`: arm an active asset drag payload from content-row pointer events.
- Modify `zircon_editor/src/ui/slint_host/app/callback_wiring.rs`: wire the new asset content pointer event callback.
- Modify `zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs`: consume the active asset payload for `AssetFieldDropped` and fall back to demo data when absent.
- Modify `zircon_editor/src/ui/slint_host/app/tests.rs`: cover active payload creation and consumption.
- Modify `zircon_editor/ui/workbench/assets.slint`: expose asset-list pointer down/up events.
- Modify `zircon_editor/ui/workbench/pane_content.slint`: forward asset content pointer events for activity and browser surfaces.
- Modify `zircon_editor/ui/workbench/pane_surface_host_context.slint`: add the host callback for asset content pointer events.
- Modify `zircon_editor/ui/workbench/template_node_data.slint`: add `drop_source_summary`.
- Modify `zircon_editor/ui/workbench/template_pane.slint`: render `drop_source_summary` for reference wells.
- Modify `zircon_editor/src/ui/template_runtime/slint_adapter.rs`: expose `drop_source_summary` in host projection DTOs.
- Modify `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs`: pass `drop_source_summary` into `TemplatePaneNodeData`.
- Modify `zircon_editor/src/ui/slint_host/ui/reference_component_tests.rs`: assert reference wells surface source summaries.
- Modify `docs/ui-and-layout/runtime-ui-component-showcase.md`: document the new metadata path and validation.

### Task 1: Runtime Drag Payload Metadata

**Files:**
- Modify: `zircon_runtime/src/ui/component/drag.rs`
- Modify: `zircon_runtime/src/ui/component/mod.rs`
- Test: `zircon_runtime/src/ui/tests/component_catalog.rs`

- [ ] **Step 1: Add the failing runtime payload roundtrip test**

Add `UiDragSourceMetadata` to the import list in `zircon_runtime/src/ui/tests/component_catalog.rs`:

```rust
use crate::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentDescriptorRegistry, UiComponentEvent,
    UiComponentEventKind, UiComponentState, UiDragPayload, UiDragPayloadKind,
    UiDragSourceMetadata, UiValidationLevel, UiValue, UiValueKind,
};
```

Add this test near the existing reference-drop tests:

```rust
#[test]
fn drag_payload_source_metadata_roundtrips_and_summarizes() {
    let source = UiDragSourceMetadata::asset(
        "browser",
        "AssetBrowserContentPanel",
        "asset-uuid-1",
        "res://textures/grid.albedo.png",
        "Grid Albedo",
        "Texture",
        "png",
    );
    let payload = UiDragPayload::new(
        UiDragPayloadKind::Asset,
        "res://textures/grid.albedo.png",
    )
    .with_source(source.clone());

    assert_eq!(payload.source.as_ref(), Some(&source));
    assert_eq!(payload.source_summary().as_deref(), Some("Texture: Grid Albedo"));

    let encoded = serde_json::to_string(&payload).unwrap();
    let decoded: UiDragPayload = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, payload);

    let legacy = UiDragPayload::new(UiDragPayloadKind::Asset, "res://legacy.mat");
    assert!(legacy.source.is_none());
    assert!(legacy.source_summary().is_none());
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run: `cargo test -p zircon_runtime --lib drag_payload_source_metadata_roundtrips_and_summarizes --locked --jobs 1`

Expected: FAIL with unresolved import or missing type `UiDragSourceMetadata`.

- [ ] **Step 3: Implement metadata in `drag.rs`**

Replace `UiDragPayload` in `zircon_runtime/src/ui/component/drag.rs` with this expanded contract while keeping `UiDragPayloadKind` and `UiDropPolicy` intact:

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDragSourceMetadata {
    pub source_surface: String,
    pub source_control_id: String,
    pub asset_uuid: Option<String>,
    pub locator: Option<String>,
    pub display_name: Option<String>,
    pub asset_kind: Option<String>,
    pub extension: Option<String>,
}

impl UiDragSourceMetadata {
    pub fn asset(
        source_surface: impl Into<String>,
        source_control_id: impl Into<String>,
        asset_uuid: impl Into<String>,
        locator: impl Into<String>,
        display_name: impl Into<String>,
        asset_kind: impl Into<String>,
        extension: impl Into<String>,
    ) -> Self {
        Self {
            source_surface: source_surface.into(),
            source_control_id: source_control_id.into(),
            asset_uuid: Some(asset_uuid.into()),
            locator: Some(locator.into()),
            display_name: Some(display_name.into()),
            asset_kind: Some(asset_kind.into()),
            extension: Some(extension.into()),
        }
    }

    pub fn summary(&self) -> Option<String> {
        match (&self.asset_kind, &self.display_name) {
            (Some(kind), Some(name)) if !kind.is_empty() && !name.is_empty() => {
                Some(format!("{kind}: {name}"))
            }
            (_, Some(name)) if !name.is_empty() => Some(name.clone()),
            (_, _) => self.locator.clone().filter(|locator| !locator.is_empty()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDragPayload {
    pub kind: UiDragPayloadKind,
    pub reference: String,
    pub source: Option<UiDragSourceMetadata>,
}

impl UiDragPayload {
    pub fn new(kind: UiDragPayloadKind, reference: impl Into<String>) -> Self {
        Self {
            kind,
            reference: reference.into(),
            source: None,
        }
    }

    pub fn with_source(mut self, source: UiDragSourceMetadata) -> Self {
        self.source = Some(source);
        self
    }

    pub fn source_summary(&self) -> Option<String> {
        self.source.as_ref().and_then(UiDragSourceMetadata::summary)
    }
}
```

- [ ] **Step 4: Re-export metadata**

Change the drag re-export in `zircon_runtime/src/ui/component/mod.rs` to:

```rust
pub use drag::{UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata, UiDropPolicy};
```

- [ ] **Step 5: Run the payload test and verify it passes**

Run: `cargo test -p zircon_runtime --lib drag_payload_source_metadata_roundtrips_and_summarizes --locked --jobs 1`

Expected: PASS.

- [ ] **Step 6: Checkpoint**

Record that Task 1 changed the shared Runtime UI drag payload contract. Do not commit unless explicit commit permission has been granted.

### Task 2: Runtime State Retains Accepted Drop Source

**Files:**
- Modify: `zircon_runtime/src/ui/component/state.rs`
- Test: `zircon_runtime/src/ui/tests/component_catalog.rs`

- [ ] **Step 1: Add the failing state-retention test**

Add this test after `component_state_handles_reference_actions_and_drop_rejection_feedback`:

```rust
#[test]
fn component_state_retains_reference_drop_source_metadata() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let asset = registry.descriptor("AssetField").unwrap();
    let source = UiDragSourceMetadata::asset(
        "browser",
        "AssetBrowserContentPanel",
        "asset-uuid-1",
        "res://textures/grid.albedo.png",
        "Grid Albedo",
        "Texture",
        "png",
    );
    let mut state = UiComponentState::new();

    state
        .apply_event(
            asset,
            UiComponentEvent::DropReference {
                property: "value".to_string(),
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://textures/grid.albedo.png",
                )
                .with_source(source.clone()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("value"),
        Some(&UiValue::AssetRef("res://textures/grid.albedo.png".to_string()))
    );
    assert_eq!(state.reference_source("value"), Some(&source));

    state
        .apply_event(
            asset,
            UiComponentEvent::ClearReference {
                property: "value".to_string(),
            },
        )
        .unwrap();

    assert_eq!(state.value("value"), Some(&UiValue::Null));
    assert_eq!(state.reference_source("value"), None);
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run: `cargo test -p zircon_runtime --lib component_state_retains_reference_drop_source_metadata --locked --jobs 1`

Expected: FAIL with missing method `reference_source`.

- [ ] **Step 3: Add source storage to `UiComponentState`**

Update the import in `state.rs`:

```rust
use super::{
    UiComponentDescriptor, UiComponentEvent, UiComponentEventError, UiComponentEventKind,
    UiDragPayloadKind, UiDragSourceMetadata, UiValidationState, UiValue, UiValueKind,
};
```

Update the struct and constructor:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentState {
    values: BTreeMap<String, UiValue>,
    reference_sources: BTreeMap<String, UiDragSourceMetadata>,
    validation: UiValidationState,
    flags: UiComponentFlags,
}

pub fn new() -> Self {
    Self {
        values: BTreeMap::new(),
        reference_sources: BTreeMap::new(),
        validation: UiValidationState::normal(),
        flags: UiComponentFlags::default(),
    }
}
```

Add the accessor near `values()`:

```rust
pub fn reference_source(&self, property: &str) -> Option<&UiDragSourceMetadata> {
    self.reference_sources.get(property)
}
```

- [ ] **Step 4: Retain and clear source metadata in event application**

In the `DropReference` arm, clone the optional source before moving the reference and update the property-keyed map:

```rust
UiComponentEvent::DropReference { property, payload } => {
    if !descriptor.accepts_drag_payload(payload.kind) {
        self.validation = UiValidationState::error(format!(
            "rejected drop payload `{}` for {}",
            payload.kind.as_str(),
            descriptor.id
        ));
        return Err(UiComponentEventError::RejectedDrop {
            component_id: descriptor.id.clone(),
            payload_kind: payload.kind.as_str().to_string(),
        });
    }
    let source = payload.source.clone();
    let value = match payload.kind {
        UiDragPayloadKind::Asset => UiValue::AssetRef(payload.reference),
        UiDragPayloadKind::SceneInstance | UiDragPayloadKind::Object => {
            UiValue::InstanceRef(payload.reference)
        }
    };
    if let Some(source) = source {
        self.reference_sources.insert(property.clone(), source);
    } else {
        self.reference_sources.remove(&property);
    }
    self.values.insert(property, value);
    Ok(())
}
```

In the `ClearReference` arm, remove retained metadata:

```rust
UiComponentEvent::ClearReference { property } => {
    self.reference_sources.remove(&property);
    self.values.insert(property, UiValue::Null);
    Ok(())
}
```

- [ ] **Step 5: Run runtime component tests**

Run: `cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1`

Expected: PASS.

- [ ] **Step 6: Checkpoint**

Record that Task 2 changed retained component state serialization shape. Do not commit unless explicit commit permission has been granted.

### Task 3: Showcase Reducer Carries Full Payloads

**Files:**
- Modify: `zircon_editor/src/ui/template_runtime/showcase_demo_state.rs`
- Modify: `zircon_editor/src/tests/host/template_runtime/component_showcase_state.rs`

- [ ] **Step 1: Update showcase tests to pass payload metadata**

Update the import list in `component_showcase_state.rs` to include `UiDragPayload` and `UiDragSourceMetadata` where `UiDragPayloadKind` is imported.

Replace the existing `AssetFieldDropped` input block with:

```rust
let source = UiDragSourceMetadata::asset(
    "browser",
    "AssetBrowserContentPanel",
    "asset-uuid-1",
    "res://materials/demo.mat",
    "Demo Material",
    "Material",
    "mat",
);
apply_showcase_binding(
    &mut runtime,
    "UiComponentShowcase/AssetFieldDropped",
    UiComponentShowcaseDemoEventInput::DropReference {
        payload: UiDragPayload::new(UiDragPayloadKind::Asset, "res://materials/demo.mat")
            .with_source(source),
    },
);
```

Add this projection assertion near the existing `AssetFieldDemo` `value_text` assertion:

```rust
assert_eq!(
    host_projection
        .node_by_control_id("AssetFieldDemo")
        .and_then(|node| node.attributes.get("drop_source_summary"))
        .and_then(toml::Value::as_str),
    Some("Material: Demo Material")
);
```

- [ ] **Step 2: Run the showcase state test and verify it fails**

Run: `cargo test -p zircon_editor --lib showcase_demo_state_applies_projected_bindings_to_retained_values_and_log --locked --jobs 1`

Expected: FAIL because `DropReference` still expects `kind` and `reference` fields.

- [ ] **Step 3: Change showcase input to carry full payloads**

Change the enum variant in `showcase_demo_state.rs`:

```rust
DropReference {
    payload: UiDragPayload,
},
```

Change the drop mapping in `showcase_event`:

```rust
"DropReference" => match input {
    UiComponentShowcaseDemoEventInput::DropReference { payload } => Ok((
        UiComponentEvent::DropReference {
            property: "value".to_string(),
            payload,
        },
        Some("value".to_string()),
    )),
    _ => Err(mismatch()),
},
```

In `apply_to_host_model`, after primary value projection, add source summary projection:

```rust
if let Some(source_summary) = state
    .reference_source("value")
    .and_then(UiDragSourceMetadata::summary)
{
    node.attributes.insert(
        "drop_source_summary".to_string(),
        TomlValue::String(source_summary),
    );
} else {
    node.attributes.remove("drop_source_summary");
}
```

Make sure `UiDragSourceMetadata` is imported in `showcase_demo_state.rs`.

- [ ] **Step 4: Update all remaining `DropReference` call sites**

Change every `UiComponentShowcaseDemoEventInput::DropReference { kind, reference }` construction to:

```rust
UiComponentShowcaseDemoEventInput::DropReference {
    payload: UiDragPayload::new(kind, reference),
}
```

For literal asset, instance, and object cases, use the explicit kind and reference:

```rust
UiComponentShowcaseDemoEventInput::DropReference {
    payload: UiDragPayload::new(
        UiDragPayloadKind::SceneInstance,
        "scene://Root/RuntimeDemoLight",
    ),
}
```

- [ ] **Step 5: Run showcase tests**

Run: `cargo test -p zircon_editor --lib showcase_demo_state_ --locked --jobs 1`

Expected: PASS.

- [ ] **Step 6: Checkpoint**

Record that Task 3 keeps fixed showcase fallback behavior while allowing real payload metadata. Do not commit unless explicit commit permission has been granted.

### Task 4: Build Asset Drag Payloads From Visible Asset Rows

**Files:**
- Create: `zircon_editor/src/ui/slint_host/app/asset_drag_payload.rs`
- Modify: `zircon_editor/src/ui/slint_host/app.rs`
- Test: `zircon_editor/src/ui/slint_host/app/tests.rs`

- [ ] **Step 1: Add a failing unit test for snapshot-to-payload conversion**

Add these imports to `app/tests.rs` if they are not already present:

```rust
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetWorkspaceSnapshot};
use zircon_runtime::core::resource::ResourceKind;
```

Add this test near the asset pointer tests:

```rust
#[test]
fn asset_drag_payload_resolves_visible_asset_metadata() {
    let mut snapshot = AssetWorkspaceSnapshot::default();
    snapshot.visible_assets.push(AssetItemSnapshot {
        uuid: "asset-uuid-1".to_string(),
        locator: "res://textures/grid.albedo.png".to_string(),
        display_name: "Grid Albedo".to_string(),
        file_name: "grid.albedo.png".to_string(),
        extension: "png".to_string(),
        kind: ResourceKind::Texture,
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected: false,
        resource_state: None,
        resource_revision: None,
    });

    let payload = super::asset_drag_payload::asset_drag_payload_from_snapshot(
        "browser",
        "asset-uuid-1",
        &snapshot,
    )
    .expect("visible asset should create a drag payload");

    assert_eq!(payload.reference, "res://textures/grid.albedo.png");
    assert_eq!(payload.kind, UiDragPayloadKind::Asset);
    assert_eq!(payload.source_summary().as_deref(), Some("Texture: Grid Albedo"));
    let source = payload.source.as_ref().expect("source metadata");
    assert_eq!(source.source_surface, "browser");
    assert_eq!(source.source_control_id, "AssetBrowserContentPanel");
    assert_eq!(source.asset_uuid.as_deref(), Some("asset-uuid-1"));
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run: `cargo test -p zircon_editor --lib asset_drag_payload_resolves_visible_asset_metadata --locked --jobs 1`

Expected: FAIL because `asset_drag_payload` does not exist.

- [ ] **Step 3: Create the helper module**

Create `zircon_editor/src/ui/slint_host/app/asset_drag_payload.rs`:

```rust
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetWorkspaceSnapshot};
use zircon_runtime::ui::component::{
    UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata,
};

pub(super) const ASSET_CONTENT_CONTROL_ID: &str = "AssetBrowserContentPanel";

pub(super) fn asset_drag_payload_from_snapshot(
    surface_mode: &str,
    asset_uuid: &str,
    snapshot: &AssetWorkspaceSnapshot,
) -> Option<UiDragPayload> {
    snapshot
        .visible_assets
        .iter()
        .find(|asset| asset.uuid == asset_uuid)
        .map(|asset| asset_drag_payload_from_item(surface_mode, asset))
}

fn asset_drag_payload_from_item(surface_mode: &str, asset: &AssetItemSnapshot) -> UiDragPayload {
    UiDragPayload::new(UiDragPayloadKind::Asset, asset.locator.clone()).with_source(
        UiDragSourceMetadata::asset(
            surface_mode,
            ASSET_CONTENT_CONTROL_ID,
            asset.uuid.clone(),
            asset.locator.clone(),
            asset.display_name.clone(),
            format!("{:?}", asset.kind),
            asset.extension.clone(),
        ),
    )
}
```

- [ ] **Step 4: Register the module**

Add this module declaration to `zircon_editor/src/ui/slint_host/app.rs` beside the other `mod asset_*` entries:

```rust
mod asset_drag_payload;
```

- [ ] **Step 5: Run the helper test**

Run: `cargo test -p zircon_editor --lib asset_drag_payload_resolves_visible_asset_metadata --locked --jobs 1`

Expected: PASS.

- [ ] **Step 6: Checkpoint**

Record that Task 4 keeps asset metadata lookup editor-local and pure. Do not commit unless explicit commit permission has been granted.

### Task 5: Arm Active Asset Drag Payloads From Asset Content Pointer Events

**Files:**
- Modify: `zircon_editor/ui/workbench/assets.slint`
- Modify: `zircon_editor/ui/workbench/pane_content.slint`
- Modify: `zircon_editor/ui/workbench/pane_surface_host_context.slint`
- Modify: `zircon_editor/src/ui/slint_host/app/callback_wiring.rs`
- Modify: `zircon_editor/src/ui/slint_host/asset_pointer/content/bridge.rs`
- Modify: `zircon_editor/src/ui/slint_host/app.rs`
- Modify: `zircon_editor/src/ui/slint_host/app/asset_content_pointer.rs`
- Test: `zircon_editor/src/ui/slint_host/app/tests.rs`

- [ ] **Step 1: Add a failing host test for active payload arming**

Add this test to `app/tests.rs` near the existing root browser asset pointer tests:

```rust
#[test]
fn asset_content_pointer_down_arms_active_asset_drag_payload() {
    let harness = ChildWindowHostHarness::new("zircon_slint_asset_drag_source_payload");
    let _asset_browser = harness.open_view("editor.asset_browser");

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        0,
        1,
        96.0,
        96.0,
        0.0,
        0.0,
    );

    let host = harness.host.borrow();
    let payload = host
        .active_asset_drag_payload
        .as_ref()
        .expect("asset row pointer down should arm an active payload");
    assert_eq!(payload.kind, UiDragPayloadKind::Asset);
    assert!(payload.reference.starts_with("res://"));
    assert!(payload.source_summary().is_some());
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run: `cargo test -p zircon_editor --lib asset_content_pointer_down_arms_active_asset_drag_payload --locked --jobs 1`

Expected: FAIL because `asset_content_pointer_event` and `active_asset_drag_payload` do not exist.

- [ ] **Step 3: Add Slint asset list pointer-event callbacks**

In `zircon_editor/ui/workbench/assets.slint`, add this callback to `AssetListView`:

```slint
callback pointer_event(kind: int, button: int, x: float, y: float, width: float, height: float);
```

Inside the `TouchArea` at the end of `AssetListView`, add:

```slint
pointer-event(event) => {
    if (event.button == PointerEventButton.left && event.kind == PointerEventKind.down) {
        root.pointer_event(0, 1, self.mouse-x / 1px, self.mouse-y / 1px + root.header_height / 1px + 1.0, root.width / 1px, root.height / 1px);
    } else if (event.button == PointerEventButton.left && event.kind == PointerEventKind.up) {
        root.pointer_event(2, 1, self.mouse-x / 1px, self.mouse-y / 1px + root.header_height / 1px + 1.0, root.width / 1px, root.height / 1px);
    }
}
```

- [ ] **Step 4: Forward pointer events through pane content**

In both `AssetsActivityPaneView` and `AssetBrowserPaneView` in `pane_content.slint`, add:

```slint
callback content_pointer_event(kind: int, button: int, x: float, y: float, width: float, height: float);
```

In each `AssetListView` binding, add:

```slint
pointer_event(kind, button, x, y, width, height) => { root.content_pointer_event(kind, button, x, y, width, height); }
```

At the `PaneContent` call sites, add:

```slint
content_pointer_event(kind, button, x, y, width, height) => { PaneSurfaceHostContext.asset_content_pointer_event("activity", kind, button, x, y, width, height); }
```

and:

```slint
content_pointer_event(kind, button, x, y, width, height) => { PaneSurfaceHostContext.asset_content_pointer_event("browser", kind, button, x, y, width, height); }
```

- [ ] **Step 5: Add the host context callback**

Add this callback to `pane_surface_host_context.slint` beside the existing asset content callbacks:

```slint
callback asset_content_pointer_event(surface_mode: string, kind: int, button: int, x: float, y: float, width: float, height: float);
```

Wire it in `callback_wiring.rs`:

```rust
let weak = Rc::downgrade(host);
let source_ui = ui.clone_strong();
pane_surface_host.on_asset_content_pointer_event(
    move |surface_mode: SharedString, kind, button, x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.asset_content_pointer_event(
                surface_mode.as_str(),
                kind,
                button,
                x,
                y,
                width,
                height,
            );
        });
    },
);
```

- [ ] **Step 6: Add a content bridge press helper**

Add this method to `AssetContentListPointerBridge` in `asset_pointer/content/bridge.rs`:

```rust
pub(crate) fn handle_press(
    &mut self,
    point: UiPoint,
) -> Result<AssetContentListPointerDispatch, String> {
    let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
    self.state.hovered_row_index = hovered_row_from_target(route.as_ref());
    Ok(AssetContentListPointerDispatch {
        route: route.map(to_public_route),
        state: self.state.clone(),
    })
}
```

- [ ] **Step 7: Store active payload in `SlintEditorHost`**

Import `UiDragPayload` in `app.rs`:

```rust
use zircon_runtime::ui::component::UiDragPayload;
```

Add this field to `SlintEditorHost`:

```rust
active_asset_drag_payload: Option<UiDragPayload>,
```

Initialize it to `None` in every `SlintEditorHost` constructor struct literal.

- [ ] **Step 8: Arm active payload from asset content pointer down**

Add this method to `app/asset_content_pointer.rs`:

```rust
pub(super) fn asset_content_pointer_event(
    &mut self,
    surface_mode: &str,
    kind: i32,
    button: i32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    if kind != 0 || button != 1 {
        return;
    }
    self.recompute_if_dirty();
    self.focus_callback_source_window();
    let Some(snapshot) = self.asset_workspace_snapshot_for_pointer(surface_mode) else {
        self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
        return;
    };
    let Some(content_size) = self
        .asset_surface_pointer_state(surface_mode)
        .and_then(|surface| {
            self.resolve_callback_surface_size_for_asset_surface(
                surface_mode,
                width,
                height,
                surface.content_size,
            )
        })
    else {
        self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
        return;
    };

    let dispatch = {
        let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return;
        };
        surface.content_size = content_size;
        surface.content_bridge.sync(
            AssetContentListPointerLayout::from_snapshot(&snapshot, surface.content_size),
            surface.content_state.clone(),
        );
        surface.content_bridge.handle_press(UiPoint::new(x, y))
    };

    match dispatch {
        Ok(dispatch) => {
            if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
                surface.content_state = dispatch.state;
            }
            self.apply_asset_pointer_state_to_ui(surface_mode);
            if let Some(AssetPointerContentRoute::Item { asset_uuid, .. }) = dispatch.route {
                self.active_asset_drag_payload =
                    asset_drag_payload::asset_drag_payload_from_snapshot(
                        surface_mode,
                        asset_uuid.as_str(),
                        &snapshot,
                    );
                if let Some(summary) = self
                    .active_asset_drag_payload
                    .as_ref()
                    .and_then(UiDragPayload::source_summary)
                {
                    self.set_status_line(format!("Asset drag source: {summary}"));
                }
            } else {
                self.active_asset_drag_payload = None;
            }
        }
        Err(error) => self.set_status_line(error),
    }
}
```

Make sure `AssetPointerContentRoute` is available through the existing `super::*` imports. If it is not, import it from `crate::ui::slint_host::asset_pointer`.

- [ ] **Step 9: Run the active payload test**

Run: `cargo test -p zircon_editor --lib asset_content_pointer_down_arms_active_asset_drag_payload --locked --jobs 1`

Expected: PASS.

- [ ] **Step 10: Checkpoint**

Record that Task 5 introduces only internal Slint host drag arming. Do not claim OS-native drag/drop support.

### Task 6: Consume Active Payload And Project Source Summary

**Files:**
- Modify: `zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs`
- Modify: `zircon_editor/ui/workbench/template_node_data.slint`
- Modify: `zircon_editor/ui/workbench/template_pane.slint`
- Modify: `zircon_editor/src/ui/template_runtime/slint_adapter.rs`
- Modify: `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs`
- Modify: `zircon_editor/src/ui/slint_host/ui/reference_component_tests.rs`
- Test: `zircon_editor/src/ui/slint_host/app/tests.rs`

- [ ] **Step 1: Add a failing test for consuming active payload**

Add this test to `app/tests.rs`:

```rust
#[test]
fn asset_field_drop_consumes_active_asset_drag_payload() {
    let harness = ChildWindowHostHarness::new("zircon_slint_asset_field_real_payload_drop");
    {
        let mut host = harness.host.borrow_mut();
        host.active_asset_drag_payload = Some(
            UiDragPayload::new(UiDragPayloadKind::Asset, "res://textures/grid.albedo.png")
                .with_source(UiDragSourceMetadata::asset(
                    "browser",
                    "AssetBrowserContentPanel",
                    "asset-uuid-1",
                    "res://textures/grid.albedo.png",
                    "Grid Albedo",
                    "Texture",
                    "png",
                )),
        );
        host.dispatch_component_showcase_control_activated(
            "AssetFieldDemo",
            "UiComponentShowcase/AssetFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_asset_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("AssetFieldDemo", "value")
            .as_deref(),
        Some("res://textures/grid.albedo.png")
    );
}
```

Add `UiDragPayload`, `UiDragPayloadKind`, and `UiDragSourceMetadata` imports if needed.

- [ ] **Step 2: Run the test and verify it fails**

Run: `cargo test -p zircon_editor --lib asset_field_drop_consumes_active_asset_drag_payload --locked --jobs 1`

Expected: FAIL because the active payload is not consumed.

- [ ] **Step 3: Consume active payload in pane surface actions**

In `pane_surface_actions.rs`, change `dispatch_component_showcase_control_activated` to call a host method rather than the free helper:

```rust
let input = self.demo_input_for_showcase_action(control_id, action_id);
```

Add this method inside `impl SlintEditorHost`:

```rust
fn demo_input_for_showcase_action(
    &mut self,
    control_id: &str,
    action_id: &str,
) -> UiComponentShowcaseDemoEventInput {
    if action_id.contains("AssetFieldDropped") {
        if let Some(payload) = self.active_asset_drag_payload.take() {
            return UiComponentShowcaseDemoEventInput::DropReference { payload };
        }
    }
    demo_input_for_showcase_action(control_id, action_id)
}
```

Update the free helper’s drop arms to construct full payload variants:

```rust
action if action.contains("AssetFieldDropped") => {
    UiComponentShowcaseDemoEventInput::DropReference {
        payload: UiDragPayload::new(
            UiDragPayloadKind::Asset,
            "res://materials/runtime_demo.mat",
        ),
    }
}
```

Apply the same `payload: UiDragPayload::new(...)` pattern for `InstanceFieldDropped` and `ObjectFieldDropped`.

- [ ] **Step 4: Add `drop_source_summary` to Slint DTOs**

In `template_node_data.slint`, add this field after `accepted_drag_payloads`:

```slint
drop_source_summary: string,
```

In `template_pane.slint`, change the reference well detail text to prefer the source summary:

```slint
text: root.node.validation_message != "" ? root.node.validation_message :
    root.node.drop_source_summary != "" ? root.node.drop_source_summary :
    root.node.accepted_drag_payloads;
```

- [ ] **Step 5: Project `drop_source_summary` through host models**

In `slint_adapter.rs`, add the field to `SlintUiHostNodeModel`:

```rust
pub drop_source_summary: Option<String>,
```

Set it during projection:

```rust
drop_source_summary: string_attribute(&node.attributes, "drop_source_summary"),
```

In `pane_data_conversion/mod.rs`, add `drop_source_summary: "".into(),` to `to_slint_template_node`, and add this to `host_template_node` initializers after `accepted_drag_payloads`:

```rust
drop_source_summary: node
    .attributes
    .get("drop_source_summary")
    .and_then(value_as_string)
    .unwrap_or_default()
    .into(),
```

This is the only planned edit to `pane_data_conversion/mod.rs` in this slice. Do not add new helper families to that file.

- [ ] **Step 6: Assert template support**

Add this assertion to `reference_component_tests.rs`:

```rust
assert!(
    template.contains("root.node.drop_source_summary"),
    "Reference wells should surface real drag source summaries when metadata exists"
);
```

- [ ] **Step 7: Run focused editor tests**

Run: `cargo test -p zircon_editor --lib asset_field_drop_consumes_active_asset_drag_payload --locked --jobs 1`

Expected: PASS.

Run: `cargo test -p zircon_editor --lib component_showcase_template_materializes_reference_drop_wells --locked --jobs 1`

Expected: PASS.

Run: `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`

Expected: PASS.

- [ ] **Step 8: Checkpoint**

Record that Task 6 connects active real asset payloads to showcase drop actions and projection. Do not commit unless explicit commit permission has been granted.

### Task 7: Documentation And Validation

**Files:**
- Modify: `docs/ui-and-layout/runtime-ui-component-showcase.md`
- Modify: `.codex/sessions/20260428-0044-runtime-ui-drag-source-metadata.md`

- [ ] **Step 1: Update module documentation**

In `docs/ui-and-layout/runtime-ui-component-showcase.md`, update the Slint Host Boundary and Validation Coverage sections to state:

```markdown
Asset Browser content-row pointer events now arm a real `UiDragPayload` for asset items. The payload keeps `kind = Asset` and writes the asset locator as the reference value, while optional `UiDragSourceMetadata` carries source surface, source control id, asset UUID, locator, display name, asset kind, and extension. Accepted `AssetField` drops retain that metadata in `UiComponentState` and project a generic `drop_source_summary` string so the Slint host can show the source without hardcoding asset-browser behavior.
```

Add the exact tests run in this slice to the document’s `tests:` header list if the header is still maintained there.

- [ ] **Step 2: Run focused validation**

Run: `cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1`

Expected: PASS.

Run: `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`

Expected: PASS.

Run: `cargo test -p zircon_editor --lib slint_host --locked --jobs 1`

Expected: PASS.

Run: `cargo check -p zircon_editor --lib --locked --jobs 1`

Expected: PASS.

- [ ] **Step 3: Expand validation if shared API churn breaks wider consumers**

If the focused commands pass and no public API consumer outside `zircon_runtime::ui::component` and `zircon_editor` changed, stop at focused validation and report that workspace-wide validation was not run due the dirty concurrent workspace.

If a shared API compile failure points outside the focused crates, run: `cargo test --workspace --locked --jobs 1`

Expected: PASS, or report the exact unrelated failure and recent coordination context.

- [ ] **Step 4: Update coordination note**

Update `.codex/sessions/20260428-0044-runtime-ui-drag-source-metadata.md` with touched modules, validation commands, and any blockers. If the implementation completes cleanly and no handoff is needed, remove the active note instead of leaving it in `.codex/sessions/`.

- [ ] **Step 5: Final checkpoint**

Summarize changed files, validation commands, remaining risks, and the large-file follow-up for `pane_data_conversion/mod.rs`. Do not commit unless explicit commit permission has been granted.

## Self-Review

Spec coverage:

- Shared payload metadata contract is covered by Tasks 1 and 2.
- Real asset-browser row metadata source is covered by Tasks 4 and 5.
- `AssetField` active payload consumption is covered by Task 6.
- Generic projection and Slint display are covered by Task 6.
- Documentation and validation are covered by Task 7.

Placeholder scan:

- No `TBD`, `TODO`, or undefined future helper remains in the executable steps.

Type consistency:

- The plan consistently uses `UiDragSourceMetadata`, `UiDragPayload::with_source`, `UiComponentState::reference_source`, and `drop_source_summary` across runtime, editor, and Slint DTOs.
