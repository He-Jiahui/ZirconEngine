use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use toml::Value;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::component::{UiComponentEvent, UiValue};
use zircon_runtime_interface::ui::dispatch::{UiPointerComponentEventReason, UiPointerEvent};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiNavigationEventKind, UiPointerButton, UiPointerEventKind,
};
use zircon_runtime_interface::ui::template::{UiBindingRef, UiNamedSlotSchema};
use zircon_runtime_interface::ui::tree::{UiInputPolicy, UiVisibility};
use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetError, UiV2AssetHeader, UiV2AssetKind, UiV2ChildMount,
    UiV2NodeDefinition, UiV2Root, UiV2StyleDeclarationBlock, UiV2StyleRule, UiV2StyleSheet,
    UI_V2_ASSET_SCHEMA_VERSION, UI_V2_REPEAT_ATTRIBUTE, UI_V2_REPEAT_FIELD_AUTHORED_COUNT,
    UI_V2_REPEAT_FIELD_KIND, UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE,
    UI_V2_REPEAT_KIND_VIRTUAL_ROWS,
};

use crate::ui::layout::compute_virtual_list_window;
use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};
use crate::ui::theme::UiThemeRegistry;
use crate::ui::v2::{
    UiV2AssetLoader, UiV2DocumentCompiler, UiV2PrototypeStore, UiV2PrototypeStoreFileCache,
    UiV2StyleResolver, UiV2SurfaceBuilder, UiZuiAssetLoader,
};

mod asset_loading;
mod composite_components;
mod default_controls;
mod demo_and_builder;
mod file_cache;
mod range_controls;
mod style_runtime;

fn v2_document(asset_id: &str, root: &str) -> UiV2AssetDocument {
    UiV2AssetDocument {
        asset: UiV2AssetHeader {
            kind: UiV2AssetKind::View,
            id: asset_id.to_string(),
            version: UI_V2_ASSET_SCHEMA_VERSION,
            display_name: String::new(),
        },
        root: Some(UiV2Root {
            node: root.to_string(),
        }),
        imports: Default::default(),
        tokens: BTreeMap::new(),
        nodes: BTreeMap::new(),
        components: BTreeMap::new(),
        stylesheets: Vec::new(),
    }
}

fn v2_cache_temp_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "zircon_ui_v2_store_{test_name}_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn persistent_cache_style(color: &str) -> String {
    format!(
        r##"
[asset]
kind = "style"
id = "ui.theme.persistent"
version = 2

[[stylesheets]]
id = "persistent"

[[stylesheets.rules]]
selector = ".persistent-root"
set = {{ self = {{ foreground_color = "{color}" }} }}
"##
    )
}

fn runtime_range_slider_surface(
    asset_id: &str,
    tree_id: &str,
    disable_swap: Option<bool>,
) -> (UiSurface, UiSize) {
    let mut props = BTreeMap::from([
        ("range_min".to_string(), Value::Float(20.0)),
        ("value".to_string(), Value::Float(80.0)),
        ("min".to_string(), Value::Float(0.0)),
        ("max".to_string(), Value::Float(100.0)),
        ("step".to_string(), Value::Float(5.0)),
    ]);
    if let Some(disable_swap) = disable_swap {
        props.insert("disable_swap".to_string(), Value::Boolean(disable_swap));
    }

    let mut document = v2_document(asset_id, "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "RangeSlider".to_string(),
            control_id: Some("RuntimeRangeSlider".to_string()),
            classes: vec!["material-range".to_string()],
            props,
            layout: Some(fixed_size_layout(100.0, 24.0)),
            events: vec![
                UiBindingRef {
                    id: "RuntimeRangeSlider/ValueChanged".to_string(),
                    event: UiEventKind::Change,
                    route: Some("RuntimeRangeSlider.ValueChanged".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
                UiBindingRef {
                    id: "RuntimeRangeSlider/DragBegin".to_string(),
                    event: UiEventKind::DragBegin,
                    route: Some("RuntimeRangeSlider.BeginDrag".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
                UiBindingRef {
                    id: "RuntimeRangeSlider/DragDelta".to_string(),
                    event: UiEventKind::DragUpdate,
                    route: Some("RuntimeRangeSlider.DragDelta".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
                UiBindingRef {
                    id: "RuntimeRangeSlider/DragEnd".to_string(),
                    event: UiEventKind::DragEnd,
                    route: Some("RuntimeRangeSlider.EndDrag".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
            ],
            ..Default::default()
        },
    );

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new(tree_id),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(160.0, 80.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();
    (surface, root_size)
}

fn fixed_size_layout(width: f64, height: f64) -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            "width".to_string(),
            Value::Table(fixed_axis_constraint(width)),
        ),
        (
            "height".to_string(),
            Value::Table(fixed_axis_constraint(height)),
        ),
    ])
}

fn fixed_axis_constraint(value: f64) -> toml::map::Map<String, Value> {
    toml::map::Map::from_iter([
        ("min".to_string(), Value::Float(value)),
        ("preferred".to_string(), Value::Float(value)),
        ("max".to_string(), Value::Float(value)),
        ("stretch".to_string(), Value::String("Fixed".to_string())),
    ])
}

fn runtime_attr<'a>(
    surface: &'a crate::ui::surface::UiSurface,
    node_id: UiNodeId,
    key: &str,
) -> Option<&'a str> {
    surface
        .tree
        .nodes
        .get(&node_id)?
        .template_metadata
        .as_ref()?
        .attributes
        .get(key)
        .and_then(Value::as_str)
}

fn runtime_color_attr<'a>(
    surface: &'a crate::ui::surface::UiSurface,
    node_id: UiNodeId,
    key: &str,
) -> Option<&'a str> {
    let value = surface
        .tree
        .nodes
        .get(&node_id)?
        .template_metadata
        .as_ref()?
        .attributes
        .get(key)?;
    value
        .as_str()
        .or_else(|| value.as_table()?.get("color")?.as_str())
}

fn runtime_style_token<'a>(
    surface: &'a crate::ui::surface::UiSurface,
    node_id: UiNodeId,
    key: &str,
) -> Option<&'a str> {
    surface
        .tree
        .nodes
        .get(&node_id)?
        .template_metadata
        .as_ref()?
        .style_tokens
        .get(key)
        .map(String::as_str)
}

fn assert_range_value(surface: &crate::ui::surface::UiSurface, node_id: UiNodeId, expected: f64) {
    assert_range_property_value(surface, node_id, "value", expected);
}

fn assert_range_property_value(
    surface: &crate::ui::surface::UiSurface,
    node_id: UiNodeId,
    property: &str,
    expected: f64,
) {
    let value = surface
        .tree
        .nodes
        .get(&node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get(property)
        .and_then(Value::as_float)
        .unwrap();
    assert!(
        (value - expected).abs() < f64::EPSILON,
        "expected range property {property} value {expected}, got {value}"
    );
}

fn node_id_by_control_id(surface: &crate::ui::surface::UiSurface, control_id: &str) -> UiNodeId {
    surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(control_id)
        })
        .unwrap_or_else(|| panic!("{control_id} should be projected"))
        .node_id
}

fn welcome_material_surface(tree_id: &str) -> UiSurface {
    editor_v2_theme_surface("welcome.zui", tree_id)
}

fn editor_v2_theme_surface(asset_file_name: &str, tree_id: &str) -> UiSurface {
    let asset_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../zircon_editor/assets/ui/editor")
        .join(asset_file_name);
    let mut cache = UiV2PrototypeStoreFileCache::new();
    let outcome = cache.load_store(vec![asset_path]).unwrap();
    UiV2SurfaceBuilder::build_surface_from_compiled_document_with_theme(
        UiTreeId::new(tree_id),
        outcome.root_document.as_ref(),
        outcome.compiled.as_ref(),
        &UiThemeRegistry::default(),
    )
    .unwrap()
}

fn render_command_background(surface: &UiSurface, node_id: UiNodeId) -> Option<&str> {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == node_id)?
        .style
        .background_color
        .as_deref()
}

fn render_command_border(surface: &UiSurface, node_id: UiNodeId) -> Option<&str> {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == node_id)?
        .style
        .border_color
        .as_deref()
}

fn style_rule<'a, const N: usize>(
    selector: &str,
    values: [(&'a str, &'a str); N],
) -> UiV2StyleRule {
    UiV2StyleRule {
        id: None,
        selector: selector.to_string(),
        set: UiV2StyleDeclarationBlock {
            self_values: values
                .into_iter()
                .map(|(key, value)| (key.to_string(), Value::String(value.to_string())))
                .collect(),
            slot: BTreeMap::new(),
        },
    }
}
