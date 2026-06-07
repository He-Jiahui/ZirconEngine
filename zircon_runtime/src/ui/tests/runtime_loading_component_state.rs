use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::{UiNodeId, UiTreeId},
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind},
    v2::{
        UiV2AssetDocument, UiV2AssetHeader, UiV2AssetKind, UiV2NodeDefinition, UiV2Root,
        UiV2StyleDeclarationBlock, UiV2StyleRule, UiV2StyleSheet, UI_V2_ASSET_SCHEMA_VERSION,
    },
};

use crate::ui::{
    surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface},
    v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder},
};

#[test]
fn loading_property_mutation_projects_retained_state_to_v2_style_and_painter() {
    let mut surface = loading_button_surface();
    let node_id = node_id_by_control_id(&surface, "RuntimeLoadingButton");

    assert_eq!(
        runtime_attr(&surface, node_id, "background_color"),
        Some("#101010")
    );
    assert!(runtime_bool_attr(&surface, node_id, "loading").is_none());
    assert_eq!(
        button_surface(&surface, node_id).style.painter_state,
        UiPainterResolvedState::Normal
    );

    let loading = surface
        .mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "loading",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(loading.status, UiPropertyMutationStatus::Accepted);
    assert!(loading.invalidation.dirty.render);
    assert!(surface.component_state(node_id).unwrap().flags.loading);
    assert_eq!(runtime_bool_attr(&surface, node_id, "loading"), Some(true));
    assert_eq!(
        runtime_attr(&surface, node_id, "background_color"),
        Some("#505050")
    );
    let dirty = surface.tree.nodes.get(&node_id).unwrap().dirty;
    assert!(dirty.render);
    assert!(!dirty.style);
    assert!(!dirty.text);

    surface.rebuild();
    assert_eq!(
        button_surface(&surface, node_id).style.painter_state,
        UiPainterResolvedState::Loading
    );
    assert_eq!(
        button_surface(&surface, node_id)
            .style
            .background_color
            .as_deref(),
        Some("#505050")
    );

    surface.clear_dirty_flags();
    let cleared = surface
        .mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "loading",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(cleared.status, UiPropertyMutationStatus::Accepted);
    assert!(!surface.component_state(node_id).unwrap().flags.loading);
    assert!(runtime_bool_attr(&surface, node_id, "loading").is_none());
    assert_eq!(
        runtime_attr(&surface, node_id, "background_color"),
        Some("#101010")
    );

    surface.rebuild();
    assert_eq!(
        button_surface(&surface, node_id).style.painter_state,
        UiPainterResolvedState::Normal
    );
}

fn loading_button_surface() -> UiSurface {
    let mut document = v2_document("asset://ui/tests/runtime_loading.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("RuntimeLoadingButton".to_string()),
            classes: vec!["material".to_string()],
            props: BTreeMap::from([
                ("text".to_string(), Value::String("Save".to_string())),
                (
                    "button_color".to_string(),
                    Value::String("secondary".to_string()),
                ),
            ]),
            layout: Some(fixed_size_layout(120.0, 32.0)),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_loading_material".to_string(),
        rules: vec![
            style_rule("Button.material", [("background_color", "#101010")]),
            style_rule("Button.material:loading", [("background_color", "#505050")]),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_loading"),
        &document,
        &compiled,
    )
    .unwrap();
    surface.rebuild();
    surface.clear_dirty_flags();
    surface
}

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

fn node_id_by_control_id(surface: &UiSurface, control_id: &str) -> UiNodeId {
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

fn runtime_attr<'a>(surface: &'a UiSurface, node_id: UiNodeId, key: &str) -> Option<&'a str> {
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

fn runtime_bool_attr(surface: &UiSurface, node_id: UiNodeId, key: &str) -> Option<bool> {
    surface
        .tree
        .nodes
        .get(&node_id)?
        .template_metadata
        .as_ref()?
        .attributes
        .get(key)
        .and_then(Value::as_bool)
}

fn button_surface(surface: &UiSurface, node_id: UiNodeId) -> &UiRenderCommand {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == node_id
                && command.kind == UiRenderCommandKind::Quad
                && command.style.painter_family == UiPainterFamily::Button
        })
        .unwrap()
}
