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
use zircon_runtime_interface::ui::tree::UiInputPolicy;
use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetError, UiV2AssetHeader, UiV2AssetKind, UiV2ChildMount,
    UiV2NodeDefinition, UiV2Root, UiV2StyleDeclarationBlock, UiV2StyleRule, UiV2StyleSheet,
    UI_V2_ASSET_SCHEMA_VERSION,
};

use crate::ui::layout::compute_virtual_list_window;
use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};
use crate::ui::v2::{
    UiV2AssetLoader, UiV2DocumentCompiler, UiV2PrototypeStore, UiV2PrototypeStoreFileCache,
    UiV2StyleResolver, UiV2SurfaceBuilder, UiZuiAssetLoader,
};

#[test]
fn ui_v2_parses_flat_view_asset() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/project_overview.v2.ui"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalGroup"
classes = ["editor-pane"]

[[nodes.root.children]]
node = "title"

[nodes.title]
component = "Text"
control_id = "ProjectTitle"

[nodes.title.props]
text = "Project"
"#,
    )
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();

    assert_eq!(compiled.arena.node_count(), 2);
    assert_eq!(compiled.component_graph.node_count(), 2);
    let root_handle = compiled.arena.root.unwrap();
    let root_graph = &compiled.component_graph.nodes[root_handle.index()];
    assert_eq!(root_graph.source_id, "root");
    assert_eq!(root_graph.children.len(), 1);
    assert_eq!(
        compiled.arena.node(root_handle).unwrap().component,
        "VerticalGroup"
    );
}

#[test]
fn ui_zui_loader_accepts_single_component_asset() {
    let document = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/primary_toolbar.zui"
version = 2

[components.PrimaryToolbar]
root = "root"

[nodes.root]
component = "HorizontalGroup"
classes = ["toolbar"]

[[nodes.root.children]]
node = "run_button"

[nodes.run_button]
component = "Button"
control_id = "RunButton"

[nodes.run_button.props]
text = "Run"
"#,
    )
    .unwrap();

    assert_eq!(document.asset.kind, UiV2AssetKind::Component);
    assert!(document.root.is_none());
    assert_eq!(document.components.len(), 1);
    assert!(document.components.contains_key("PrimaryToolbar"));
    assert!(document.nodes.contains_key("root"));
}

#[test]
fn ui_zui_loader_rejects_view_assets() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/editor/workbench.zui"
version = 2

[root]
node = "root"

[nodes.root]
component = "Container"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("asset.kind")
    ));
}

#[test]
fn ui_zui_loader_rejects_view_root_on_component_assets() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/invalid_root.zui"
version = 2

[root]
node = "root"

[components.InvalidRoot]
root = "root"

[nodes.root]
component = "Container"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("[root]")
    ));
}

#[test]
fn ui_zui_loader_rejects_multiple_components() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/invalid_multiple.zui"
version = 2

[components.LeftPanel]
root = "left_root"

[components.RightPanel]
root = "right_root"

[nodes.left_root]
component = "Container"

[nodes.right_root]
component = "Container"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("exactly one component")
    ));
}

#[test]
fn ui_zui_loader_rejects_missing_component() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/invalid_missing_component.zui"
version = 2

[nodes.root]
component = "Container"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("exactly one component")
    ));
}

#[test]
fn ui_zui_loader_rejects_empty_component_root() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/invalid_empty_root.zui"
version = 2

[components.EmptyRoot]
root = ""

[nodes.root]
component = "Container"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("non-empty root node")
    ));
}

#[test]
fn ui_zui_loader_rejects_missing_component_root_node() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/invalid_missing_root_node.zui"
version = 2

[components.MissingRoot]
root = "missing"

[nodes.root]
component = "Container"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::MissingNode { node_id, .. } if node_id == "missing"
    ));
}

#[test]
fn ui_zui_loader_rejects_style_assets() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "style"
id = "asset://ui/editor/theme.zui"
version = 2

[[stylesheets]]
id = "editor_theme"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("asset.kind")
    ));
}

#[test]
fn ui_v2_rejects_cycles_before_surface_build() {
    let mut document = v2_document("asset://ui/tests/cycle.v2.ui", "a");
    document.nodes.insert(
        "a".to_string(),
        UiV2NodeDefinition {
            component: "Container".to_string(),
            children: vec![UiV2ChildMount {
                node: "b".to_string(),
                slot: BTreeMap::new(),
            }],
            ..Default::default()
        },
    );
    document.nodes.insert(
        "b".to_string(),
        UiV2NodeDefinition {
            component: "Container".to_string(),
            children: vec![UiV2ChildMount {
                node: "a".to_string(),
                slot: BTreeMap::new(),
            }],
            ..Default::default()
        },
    );

    let error = UiV2DocumentCompiler::compile(&document).unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("cycle")
    ));
}

#[test]
fn ui_v2_style_specificity_and_pseudo_state_are_resolved() {
    let mut document = v2_document("asset://ui/tests/style.v2.ui", "root");
    document.tokens.insert(
        "material.primary".to_string(),
        Value::String("#6750a4".to_string()),
    );
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("RunButton".to_string()),
            classes: vec!["primary".to_string()],
            state: BTreeMap::from([("hovered".to_string(), Value::Boolean(true))]),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "editor_material".to_string(),
        rules: vec![
            style_rule("Button", [("fg", "#111111")]),
            style_rule(".primary", [("fg", "#222222")]),
            style_rule("Button.primary:hover", [("fg", "$material.primary")]),
            style_rule("#RunButton", [("radius", "6")]),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let resolved = UiV2StyleResolver::resolve(&document, &compiled.arena).unwrap();
    let root = resolved.nodes.get("root").unwrap();

    assert_eq!(root.self_values["fg"].as_str(), Some("#6750a4"));
    assert_eq!(root.self_values["radius"].as_str(), Some("6"));
}

#[test]
fn ui_v2_surface_runtime_pseudo_state_restyles_from_retained_component_state() {
    let mut document = v2_document("asset://ui/tests/runtime_style.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("RuntimeButton".to_string()),
            classes: vec!["material".to_string()],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_material".to_string(),
        rules: vec![
            style_rule("Button.material", [("background", "#101010")]),
            style_rule("Button.material:hover", [("background", "#202020")]),
            style_rule("Button.material:active", [("background", "#303030")]),
            style_rule("#RuntimeButton:focus", [("outline", "#404040")]),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_style"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = surface.tree.roots[0];

    assert_eq!(
        surface
            .tree
            .nodes
            .get(&node_id)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("background")
            .and_then(Value::as_str),
        Some("#101010")
    );
    assert!(surface
        .tree
        .nodes
        .get(&node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get("outline")
        .is_none());

    assert!(surface.component_states.set_hovered(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#202020")
    );
    let dirty = surface.tree.nodes.get(&node_id).unwrap().dirty;
    assert!(dirty.render);
    assert!(!dirty.style);
    assert!(!dirty.text);

    surface.clear_dirty_flags();
    assert!(surface.component_states.set_pressed(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#303030")
    );

    surface.clear_dirty_flags();
    assert!(surface.component_states.set_focused(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(runtime_attr(&surface, node_id, "outline"), Some("#404040"));
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#303030")
    );

    surface.clear_dirty_flags();
    assert!(surface.component_states.set_pressed(node_id, false));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#202020")
    );

    surface.clear_dirty_flags();
    assert!(surface.component_states.set_hovered(node_id, false));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#101010")
    );
    assert_eq!(runtime_attr(&surface, node_id, "outline"), Some("#404040"));
}

#[test]
fn ui_v2_surface_authored_pseudo_state_is_seeded_but_not_baked_into_baseline() {
    let mut document = v2_document("asset://ui/tests/runtime_style_seeded.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("SeededButton".to_string()),
            classes: vec!["material".to_string()],
            state: BTreeMap::from([("hovered".to_string(), Value::Boolean(true))]),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_material".to_string(),
        rules: vec![
            style_rule("Button.material", [("background", "#101010")]),
            style_rule("Button.material:hover", [("background", "#202020")]),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_style_seeded"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = surface.tree.roots[0];

    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#202020")
    );

    assert!(surface.component_states.set_hovered(node_id, false));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#101010")
    );
}

#[test]
fn ui_v2_surface_runtime_pseudo_state_restyles_deep_descendant_from_parent_state() {
    const NODE_COUNT: usize = 512;

    let mut document = v2_document("asset://ui/tests/runtime_style_deep.v2.ui", "n0");
    for index in 0..NODE_COUNT {
        let is_leaf = index + 1 == NODE_COUNT;
        let child = (!is_leaf).then(|| UiV2ChildMount {
            node: format!("n{}", index + 1),
            slot: BTreeMap::new(),
        });
        document.nodes.insert(
            format!("n{index}"),
            UiV2NodeDefinition {
                component: if is_leaf {
                    "Text".to_string()
                } else {
                    "Container".to_string()
                },
                control_id: is_leaf.then(|| "DeepRuntimeLabel".to_string()),
                classes: if index == 0 {
                    vec!["runtime-host".to_string()]
                } else if is_leaf {
                    vec!["deep-label".to_string()]
                } else {
                    Vec::new()
                },
                children: child.into_iter().collect(),
                ..Default::default()
            },
        );
    }
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_deep_material".to_string(),
        rules: vec![
            style_rule("Text.deep-label", [("foreground", "#111111")]),
            style_rule(
                ".runtime-host:hover Text.deep-label",
                [("foreground", "#abcdef")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_style_deep"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_id = surface.tree.roots[0];
    let leaf_id = node_id_by_control_id(&surface, "DeepRuntimeLabel");

    assert_eq!(
        runtime_color_attr(&surface, leaf_id, "foreground"),
        Some("#111111")
    );

    assert!(surface.component_states.set_hovered(root_id, true));
    surface.mark_component_state_render_dirty(root_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, leaf_id, "foreground"),
        Some("#abcdef")
    );
    let leaf_dirty = surface.tree.nodes.get(&leaf_id).unwrap().dirty;
    assert!(leaf_dirty.render);
    assert!(!leaf_dirty.layout);
    assert!(!leaf_dirty.style);

    surface.clear_dirty_flags();
    assert!(surface.component_states.set_hovered(root_id, false));
    surface.mark_component_state_render_dirty(root_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, leaf_id, "foreground"),
        Some("#111111")
    );
}

#[test]
fn ui_v2_surface_property_mutation_restyles_checked_and_disabled_pseudo_state() {
    let mut document = v2_document("asset://ui/tests/runtime_style_property.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Toggle".to_string(),
            control_id: Some("RuntimeToggle".to_string()),
            classes: vec!["material-toggle".to_string()],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_property_material".to_string(),
        rules: vec![
            style_rule("Toggle.material-toggle", [("background", "#101010")]),
            style_rule(
                "Toggle.material-toggle:checked",
                [("background", "#225533")],
            ),
            style_rule(
                "Toggle.material-toggle:disabled",
                [("foreground", "#778899")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_style_property"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "RuntimeToggle");

    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );

    let checked = surface
        .mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "checked",
            UiValue::Bool(true),
        ))
        .unwrap();
    assert_eq!(checked.status, UiPropertyMutationStatus::Accepted);
    assert!(checked.invalidation.dirty.render);
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#225533")
    );

    surface.clear_dirty_flags();
    let disabled = surface
        .mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "enabled",
            UiValue::Bool(false),
        ))
        .unwrap();
    assert_eq!(disabled.status, UiPropertyMutationStatus::Accepted);
    assert!(disabled.invalidation.dirty.input);
    assert!(disabled.invalidation.dirty.render);
    assert_eq!(
        runtime_color_attr(&surface, node_id, "foreground"),
        Some("#778899")
    );
    let dirty = surface.tree.nodes.get(&node_id).unwrap().dirty;
    assert!(dirty.input);
    assert!(dirty.render);
    assert!(!dirty.layout);
}

#[test]
fn ui_v2_inline_style_overrides_cascade_values_in_style_overrides() {
    let mut document = v2_document("asset://ui/tests/style_override_priority.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("OverrideButton".to_string()),
            classes: vec!["material-button".to_string()],
            style: UiV2StyleDeclarationBlock {
                self_values: BTreeMap::from([(
                    "button_variant".to_string(),
                    Value::String("outlined".to_string()),
                )]),
                slot: BTreeMap::new(),
            },
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "style_override_priority".to_string(),
        rules: vec![style_rule(
            "Button.material-button",
            [("button_variant", "contained")],
        )],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.style_override_priority"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "OverrideButton");
    let metadata = surface
        .tree
        .nodes
        .get(&node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();

    assert_eq!(
        metadata.attributes["button_variant"].as_str(),
        Some("contained")
    );
    assert_eq!(
        metadata.style_overrides["button_variant"].as_str(),
        Some("outlined")
    );
}

#[test]
fn ui_v2_surface_default_toggle_click_mutates_checked_and_restyles_runtime_pseudo_state() {
    let mut document = v2_document("asset://ui/tests/runtime_toggle_click.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Toggle".to_string(),
            control_id: Some("RuntimeToggle".to_string()),
            classes: vec!["material-toggle".to_string()],
            layout: Some(fixed_size_layout(120.0, 32.0)),
            events: vec![UiBindingRef {
                id: "RuntimeToggle/Changed".to_string(),
                event: UiEventKind::Change,
                route: Some("RuntimeToggle.Change".to_string()),
                action: None,
                targets: Vec::new(),
            }],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_toggle_material".to_string(),
        rules: vec![
            style_rule("Toggle.material-toggle", [("background", "#101010")]),
            style_rule(
                "Toggle.material-toggle:checked",
                [("background", "#225533")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_toggle_click"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(200.0, 100.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeToggle");
    assert!(
        !surface
            .tree
            .nodes
            .get(&node_id)
            .unwrap()
            .state_flags
            .checked
    );
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );

    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let point = UiPoint::new(12.0, 12.0);
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.rebuild_dirty(root_size).unwrap();

    let up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(
        surface
            .tree
            .nodes
            .get(&node_id)
            .unwrap()
            .state_flags
            .checked
    );
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#225533")
    );
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
    assert!(up.component_events.iter().any(|event| {
        event.node_id == node_id
            && event.event_kind == UiEventKind::Change
            && event.reason == UiPointerComponentEventReason::DefaultClick
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "checked" && value == &UiValue::Bool(true)
            )
    }));

    let rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
    assert!(!rebuild.arranged_rebuilt);
    assert!(!rebuild.hit_grid_rebuilt);
    assert_eq!(
        render_command_background(&surface, node_id).as_deref(),
        Some("#225533")
    );
}

#[test]
fn ui_v2_surface_authored_checked_state_can_toggle_off_without_stale_prop_style() {
    let mut document = v2_document("asset://ui/tests/runtime_toggle_checked_seed.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Toggle".to_string(),
            control_id: Some("RuntimeToggle".to_string()),
            classes: vec!["material-toggle".to_string()],
            props: BTreeMap::from([("checked".to_string(), Value::Boolean(true))]),
            layout: Some(fixed_size_layout(120.0, 32.0)),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_toggle_checked_seed_material".to_string(),
        rules: vec![
            style_rule("Toggle.material-toggle", [("background", "#101010")]),
            style_rule(
                "Toggle.material-toggle:checked",
                [("background", "#225533")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_toggle_checked_seed"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(200.0, 100.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeToggle");
    assert!(
        surface
            .tree
            .nodes
            .get(&node_id)
            .unwrap()
            .state_flags
            .checked
    );
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#225533")
    );

    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let point = UiPoint::new(12.0, 12.0);
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.rebuild_dirty(root_size).unwrap();
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(
        !surface
            .tree
            .nodes
            .get(&node_id)
            .unwrap()
            .state_flags
            .checked
    );
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );
    let rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
}

#[test]
fn ui_v2_surface_default_foldout_click_toggles_expanded_and_restyles_runtime_pseudo_state() {
    let mut document = v2_document("asset://ui/tests/runtime_foldout_toggle.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Foldout".to_string(),
            control_id: Some("RuntimeFoldout".to_string()),
            classes: vec!["material-foldout".to_string()],
            props: BTreeMap::from([("expanded".to_string(), Value::Boolean(true))]),
            layout: Some(fixed_size_layout(160.0, 32.0)),
            events: vec![UiBindingRef {
                id: "RuntimeFoldout/Toggled".to_string(),
                event: UiEventKind::Toggle,
                route: Some("RuntimeFoldout.Toggle".to_string()),
                action: None,
                targets: Vec::new(),
            }],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_foldout_material".to_string(),
        rules: vec![
            style_rule("Foldout.material-foldout", [("background", "#101010")]),
            style_rule(
                "Foldout.material-foldout:expanded",
                [("background", "#225533")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_foldout_toggle"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(220.0, 100.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeFoldout");
    assert!(surface
        .component_state(node_id)
        .is_some_and(|state| state.flags.expanded));
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#225533")
    );

    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let point = UiPoint::new(12.0, 12.0);
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.rebuild_dirty(root_size).unwrap();

    let up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(!surface
        .component_state(node_id)
        .is_some_and(|state| state.flags.expanded));
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
    assert!(up.component_events.iter().any(|event| {
        event.node_id == node_id
            && event.event_kind == UiEventKind::Toggle
            && event.reason == UiPointerComponentEventReason::DefaultClick
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ToggleExpanded { expanded } if !expanded
            )
    }));

    let rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
    assert!(!rebuild.arranged_rebuilt);
    assert!(!rebuild.hit_grid_rebuilt);
}

#[test]
fn ui_v2_surface_default_combobox_click_toggles_popup_open_and_routes_typed_events() {
    let mut document = v2_document("asset://ui/tests/runtime_combobox_popup.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "ComboBox".to_string(),
            control_id: Some("RuntimeComboBox".to_string()),
            classes: vec!["material-combo".to_string()],
            props: BTreeMap::from([
                ("value".to_string(), Value::String("scene".to_string())),
                ("popup_open".to_string(), Value::Boolean(false)),
            ]),
            layout: Some(fixed_size_layout(180.0, 32.0)),
            events: vec![
                UiBindingRef {
                    id: "RuntimeComboBox/OpenPopup".to_string(),
                    event: UiEventKind::Click,
                    route: Some("RuntimeComboBox.OpenPopup".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
                UiBindingRef {
                    id: "RuntimeComboBox/ClosePopup".to_string(),
                    event: UiEventKind::Click,
                    route: Some("RuntimeComboBox.ClosePopup".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
            ],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_combobox_popup_material".to_string(),
        rules: vec![
            style_rule("ComboBox.material-combo", [("background", "#101010")]),
            style_rule("ComboBox.material-combo:open", [("background", "#225533")]),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_combobox_popup"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(240.0, 100.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeComboBox");
    assert!(!surface
        .component_state(node_id)
        .is_some_and(|state| state.flags.popup_open));
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );

    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let point = UiPoint::new(12.0, 12.0);
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.rebuild_dirty(root_size).unwrap();

    let open = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(surface
        .component_state(node_id)
        .is_some_and(|state| state.flags.popup_open));
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#225533")
    );
    assert!(open.component_events.iter().any(|event| {
        event.node_id == node_id
            && event.binding_id == "RuntimeComboBox/OpenPopup"
            && event.event_kind == UiEventKind::Click
            && event.reason == UiPointerComponentEventReason::DefaultClick
            && matches!(&event.envelope.event, UiComponentEvent::OpenPopup)
    }));
    assert!(!open
        .component_events
        .iter()
        .any(|event| event.binding_id == "RuntimeComboBox/ClosePopup"));

    let rebuild_open = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild_open.render_rebuilt);
    assert!(!rebuild_open.layout_recomputed);
    assert!(!rebuild_open.arranged_rebuilt);
    assert!(!rebuild_open.hit_grid_rebuilt);
    assert_eq!(
        render_command_background(&surface, node_id).as_deref(),
        Some("#225533")
    );

    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.rebuild_dirty(root_size).unwrap();

    let close = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(!surface
        .component_state(node_id)
        .is_some_and(|state| state.flags.popup_open));
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );
    assert!(close.component_events.iter().any(|event| {
        event.node_id == node_id
            && event.binding_id == "RuntimeComboBox/ClosePopup"
            && event.event_kind == UiEventKind::Click
            && event.reason == UiPointerComponentEventReason::DefaultClick
            && matches!(&event.envelope.event, UiComponentEvent::ClosePopup)
    }));
    assert!(!close
        .component_events
        .iter()
        .any(|event| event.binding_id == "RuntimeComboBox/OpenPopup"));
}

#[test]
fn ui_v2_surface_default_rangefield_click_sets_value_and_rebuilds_render_only() {
    let mut document = v2_document("asset://ui/tests/runtime_rangefield_click.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "RangeField".to_string(),
            control_id: Some("RuntimeRange".to_string()),
            classes: vec!["material-range".to_string()],
            props: BTreeMap::from([
                ("value".to_string(), Value::Float(0.0)),
                ("min".to_string(), Value::Float(0.0)),
                ("max".to_string(), Value::Float(100.0)),
                ("step".to_string(), Value::Float(5.0)),
            ]),
            layout: Some(fixed_size_layout(100.0, 24.0)),
            events: vec![UiBindingRef {
                id: "RuntimeRange/ValueChanged".to_string(),
                event: UiEventKind::Change,
                route: Some("RuntimeRange.ValueChanged".to_string()),
                action: None,
                targets: Vec::new(),
            }],
            ..Default::default()
        },
    );

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_rangefield_click"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(160.0, 80.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeRange");
    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let frame = surface.arranged_tree.get(node_id).unwrap().frame;
    let point = UiPoint::new(frame.x + frame.width * 0.73, frame.y + frame.height * 0.5);
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.rebuild_dirty(root_size).unwrap();

    let up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    let value = surface
        .tree
        .nodes
        .get(&node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get("value")
        .and_then(Value::as_float)
        .unwrap();
    assert!((value - 75.0).abs() < f64::EPSILON);
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
    assert!(up.component_events.iter().any(|event| {
        event.node_id == node_id
            && event.binding_id == "RuntimeRange/ValueChanged"
            && event.event_kind == UiEventKind::Change
            && event.reason == UiPointerComponentEventReason::DefaultClick
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "value" && value == &UiValue::Float(75.0)
            )
    }));

    let rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
    assert!(!rebuild.arranged_rebuilt);
    assert!(!rebuild.hit_grid_rebuilt);
}

#[test]
fn ui_v2_surface_rangefield_drag_captures_pointer_and_updates_value_outside_hit() {
    let mut document = v2_document("asset://ui/tests/runtime_rangefield_drag.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "RangeField".to_string(),
            control_id: Some("RuntimeRange".to_string()),
            classes: vec!["material-range".to_string()],
            props: BTreeMap::from([
                ("value".to_string(), Value::Float(50.0)),
                ("min".to_string(), Value::Float(0.0)),
                ("max".to_string(), Value::Float(100.0)),
                ("step".to_string(), Value::Float(5.0)),
            ]),
            layout: Some(fixed_size_layout(100.0, 24.0)),
            events: vec![
                UiBindingRef {
                    id: "RuntimeRange/ValueChanged".to_string(),
                    event: UiEventKind::Change,
                    route: Some("RuntimeRange.ValueChanged".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
                UiBindingRef {
                    id: "RuntimeRange/DragBegin".to_string(),
                    event: UiEventKind::DragBegin,
                    route: Some("RuntimeRange.BeginDrag".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
                UiBindingRef {
                    id: "RuntimeRange/DragUpdate".to_string(),
                    event: UiEventKind::DragUpdate,
                    route: Some("RuntimeRange.DragUpdate".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
                UiBindingRef {
                    id: "RuntimeRange/DragEnd".to_string(),
                    event: UiEventKind::DragEnd,
                    route: Some("RuntimeRange.EndDrag".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
            ],
            ..Default::default()
        },
    );

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_rangefield_drag"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(160.0, 80.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeRange");
    let frame = surface.arranged_tree.get(node_id).unwrap().frame;
    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let down_point = UiPoint::new(frame.x + frame.width * 0.2, frame.y + frame.height * 0.5);
    let down = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, down_point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(down.captured_by, Some(node_id));
    assert_eq!(surface.focus.captured, Some(node_id));
    assert!(down.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRange/DragBegin"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::BeginDrag { property } if property == "value"
            )
    }));
    surface.rebuild_dirty(root_size).unwrap();

    let outside_right = UiPoint::new(frame.x + frame.width * 1.25, frame.y + frame.height * 0.5);
    let drag = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, outside_right),
        )
        .unwrap();

    assert_eq!(drag.handled_by, Some(node_id));
    assert_eq!(surface.focus.captured, Some(node_id));
    assert_range_value(&surface, node_id, 100.0);
    assert!(drag.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRange/ValueChanged"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "value" && value == &UiValue::Float(100.0)
            )
    }));
    assert!(drag.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRange/DragUpdate"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::DragDelta { property, delta }
                    if property == "value" && (*delta - 50.0).abs() < f64::EPSILON
            )
    }));
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
    let drag_rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(drag_rebuild.render_rebuilt);
    assert!(!drag_rebuild.layout_recomputed);
    assert!(!drag_rebuild.hit_grid_rebuilt);

    let outside_left = UiPoint::new(frame.x - frame.width * 0.25, frame.y + frame.height * 0.5);
    let up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, outside_left)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(up.released_capture, Some(node_id));
    assert_eq!(surface.focus.captured, None);
    assert_range_value(&surface, node_id, 0.0);
    assert!(up.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRange/DragEnd"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::EndDrag { property } if property == "value"
            )
    }));
}

#[test]
fn ui_v2_surface_rangefield_keyboard_navigation_steps_value_render_only() {
    let mut document = v2_document("asset://ui/tests/runtime_rangefield_keyboard.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "RangeField".to_string(),
            control_id: Some("RuntimeRange".to_string()),
            props: BTreeMap::from([
                ("value".to_string(), Value::Float(50.0)),
                ("min".to_string(), Value::Float(0.0)),
                ("max".to_string(), Value::Float(100.0)),
                ("step".to_string(), Value::Float(5.0)),
            ]),
            layout: Some(fixed_size_layout(100.0, 24.0)),
            ..Default::default()
        },
    );

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_rangefield_keyboard"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(160.0, 80.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeRange");
    surface.focus_node(node_id).unwrap();
    surface.clear_dirty_flags();

    let right = surface
        .dispatch_navigation_event(
            &crate::ui::dispatch::UiNavigationDispatcher::default(),
            UiNavigationEventKind::Right,
        )
        .unwrap();
    assert_eq!(right.handled_by, Some(node_id));
    assert_eq!(right.focus_changed_to, None);
    assert_range_value(&surface, node_id, 55.0);
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
    surface.rebuild_dirty(root_size).unwrap();

    let left = surface
        .dispatch_navigation_event(
            &crate::ui::dispatch::UiNavigationDispatcher::default(),
            UiNavigationEventKind::Left,
        )
        .unwrap();
    assert_eq!(left.handled_by, Some(node_id));
    assert_range_value(&surface, node_id, 50.0);
    let rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
    assert!(!rebuild.hit_grid_rebuilt);
}

#[test]
fn material_demo_window_compiles_and_resolves_material_dark_states() {
    let document = UiV2AssetLoader::load_toml_str(include_str!(
        "../../../../zircon_editor/assets/ui/editor/material_demo_window.v2.ui.toml"
    ))
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let resolved = UiV2StyleResolver::resolve(&document, &compiled.arena).unwrap();

    let components = compiled
        .arena
        .nodes
        .iter()
        .map(|node| node.component.as_str())
        .collect::<BTreeSet<_>>();
    for component in [
        "Button",
        "IconButton",
        "TextField",
        "Checkbox",
        "Switch",
        "Dropdown",
        "Slider",
        "Tabs",
        "Menu",
        "Tooltip",
        "Scrollbar",
        "Splitter",
        "Panel",
        "Modal",
    ] {
        assert!(components.contains(component), "missing {component}");
    }

    assert_eq!(
        resolved.nodes["window"].self_values["background"].as_str(),
        Some("#121212")
    );
    assert_eq!(
        resolved.nodes["primary_button"].self_values["background"].as_str(),
        Some("rgba(255,255,255,0.08)")
    );
    assert_eq!(
        resolved.nodes["icon_button"].self_values["background"].as_str(),
        Some("rgba(255,255,255,0.12)")
    );
    assert_eq!(
        resolved.nodes["primary_button"].self_values["outline"].as_str(),
        Some("#90caf9")
    );
    assert_eq!(
        resolved.nodes["text_field"].self_values["fg"].as_str(),
        Some("#ffa726")
    );
    assert_eq!(
        resolved.nodes["modal"].self_values["fg"].as_str(),
        Some("#f44336")
    );
    assert_eq!(
        resolved.nodes["switch"].self_values["fg"].as_str(),
        Some("rgba(255,255,255,0.30)")
    );
}

#[test]
fn editor_material_theme_runtime_pseudo_states_drive_imported_v2_surface() {
    let mut surface = welcome_material_surface("runtime.ui.v2.editor_material_state");
    let button_id = node_id_by_control_id(&surface, "WelcomeStartupDemoButton");

    assert_eq!(
        runtime_color_attr(&surface, button_id, "background"),
        Some("#202830")
    );

    assert!(surface.component_states.set_hovered(button_id, true));
    surface
        .mark_component_state_render_dirty(button_id)
        .unwrap();
    assert_eq!(
        runtime_color_attr(&surface, button_id, "background"),
        Some("#2f4650")
    );
    assert_eq!(
        runtime_color_attr(&surface, button_id, "foreground"),
        Some("#e6f1f4")
    );

    assert!(surface.component_states.set_pressed(button_id, true));
    surface
        .mark_component_state_render_dirty(button_id)
        .unwrap();
    assert_eq!(
        runtime_color_attr(&surface, button_id, "background"),
        Some("#103c4a")
    );

    assert!(surface.component_states.set_focused(button_id, true));
    surface
        .mark_component_state_render_dirty(button_id)
        .unwrap();
    assert_eq!(
        runtime_color_attr(&surface, button_id, "border"),
        Some("#80eaff")
    );
}

#[test]
fn editor_material_runtime_pseudo_states_rebuild_render_extract_variants() {
    let mut surface = welcome_material_surface("runtime.ui.v2.editor_material_render_state");
    let root_size = UiSize::new(960.0, 640.0);
    surface.compute_layout(root_size).unwrap();
    let button_id = node_id_by_control_id(&surface, "WelcomeStartupDemoButton");
    let button_frame = surface.arranged_tree.get(button_id).unwrap().frame;
    let pointer = UiPoint::new(button_frame.x + 2.0, button_frame.y + 2.0);

    assert_eq!(
        render_command_background(&surface, button_id),
        Some("#202830")
    );

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, pointer),
        )
        .unwrap();
    let hover_report = surface.rebuild_dirty(root_size).unwrap();
    assert!(hover_report.render_rebuilt);
    assert!(!hover_report.layout_recomputed);
    assert!(!hover_report.arranged_rebuilt);
    assert_eq!(
        render_command_background(&surface, button_id),
        Some("#2f4650")
    );

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, pointer)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let press_report = surface.rebuild_dirty(root_size).unwrap();
    assert!(press_report.render_rebuilt);
    assert!(!press_report.layout_recomputed);
    assert_eq!(
        render_command_background(&surface, button_id),
        Some("#103c4a")
    );
    assert_eq!(render_command_border(&surface, button_id), Some("#80eaff"));
}

#[test]
fn layout_demo_window_compiles_with_window_drawer_and_data_view_components() {
    let document = UiV2AssetLoader::load_toml_str(include_str!(
        "../../../../zircon_editor/assets/ui/editor/layout_demo_window.v2.ui.toml"
    ))
    .unwrap();

    let source_components = document
        .nodes
        .values()
        .map(|node| node.component.as_str())
        .collect::<BTreeSet<_>>();
    assert!(source_components.contains("Slot"), "missing source Slot");

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("layout demo root"))
        .unwrap();
    assert_eq!(root.component, "DockHost");

    let components = compiled
        .arena
        .nodes
        .iter()
        .map(|node| node.component.as_str())
        .collect::<BTreeSet<_>>();
    for component in [
        "Window",
        "WorkbenchShell",
        "Drawer",
        "View",
        "WindowFrame",
        "DocumentNode",
        "TabStack",
        "FloatingWindow",
        "FlexGroup",
        "HorizontalGroup",
        "GridGroup",
        "Overlay",
        "ListView",
        "VirtualList",
        "TreeView",
        "PropertyGrid",
        "InspectorSection",
        "Composite",
    ] {
        assert!(components.contains(component), "missing {component}");
    }
    assert!(compiled.arena.node_count() >= 25);
}

#[test]
fn fyrox_panel_demo_window_compiles_with_all_panel_role_components() {
    let document = UiV2AssetLoader::load_toml_str(include_str!(
        "../../../../zircon_editor/assets/ui/editor/fyrox_panel_demo_window.v2.ui.toml"
    ))
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("fyrox panel demo root"))
        .unwrap();
    assert_eq!(root.component, "WorkbenchShell");

    let components = compiled
        .arena
        .nodes
        .iter()
        .map(|node| node.component.as_str())
        .collect::<BTreeSet<_>>();
    for component in [
        "AssetGrid",
        "AssetList",
        "CategorizedList",
        "ContextMenu",
        "FieldEditor",
        "FilterBar",
        "FolderTree",
        "GizmoControls",
        "GraphCanvas",
        "InspectorSection",
        "MetadataPane",
        "PaneToolbar",
        "PreviewPane",
        "PropertyGrid",
        "SearchField",
        "SeverityChips",
        "SourceEditor",
        "StatusActionControls",
        "Timeline",
        "TreeView",
        "VirtualList",
        "ViewportHost",
        "VisualDesigner",
    ] {
        assert!(components.contains(component), "missing {component}");
    }
    assert_eq!(compiled.arena.node_count(), document.nodes.len());
}

#[test]
fn ui_v2_builds_deep_surface_without_recursive_template_tree() {
    const NODE_COUNT: usize = 10_000;
    let mut document = v2_document("asset://ui/tests/deep.v2.ui", "n0");
    for index in 0..NODE_COUNT {
        let child = (index + 1 < NODE_COUNT).then(|| UiV2ChildMount {
            node: format!("n{}", index + 1),
            slot: BTreeMap::new(),
        });
        document.nodes.insert(
            format!("n{index}"),
            UiV2NodeDefinition {
                component: "Container".to_string(),
                control_id: (index + 1 == NODE_COUNT).then(|| "DeepLeaf".to_string()),
                children: child.into_iter().collect(),
                ..Default::default()
            },
        );
    }

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.deep"),
        &document,
        &compiled,
    )
    .unwrap();

    assert_eq!(surface.tree.nodes.len(), NODE_COUNT);
    assert_eq!(surface.tree.roots.len(), 1);
}

#[test]
fn ui_v2_surface_builder_preserves_direct_runtime_tree_contracts() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/surface_tree_contract.v2.ui"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalBox"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 4.0 } }
children = [{ node = "action", slot = { layout = { padding = { left = 3.0, top = 5.0, right = 7.0, bottom = 11.0 }, linear_size = { rule = "Auto" } } } }]

[nodes.action]
component = "Button"
control_id = "ActionButton"
props = { text = "Run", input_clickable = false, input_focusable = true }
layout = { width = { min = 80.0, preferred = 120.0, max = 160.0, stretch = "Fixed" }, height = { min = 24.0, preferred = 32.0, max = 48.0, stretch = "Fixed" } }
events = [{ id = "Run", event = "Click", route = "Runtime.Run" }]
"#,
    )
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.surface_tree_contract"),
        &document,
        &compiled,
    )
    .unwrap();
    let root = surface.tree.nodes.get(&surface.tree.roots[0]).unwrap();
    let action = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("ActionButton")
        })
        .expect("action node should be projected");

    assert_eq!(root.children, vec![action.node_id]);
    assert!(root.layout_stretch_width);
    assert!(root.layout_stretch_height);
    assert!(!action.state_flags.clickable);
    assert!(action.state_flags.hoverable);
    assert!(action.state_flags.focusable);
    let metadata = action.template_metadata.as_ref().unwrap();
    assert_eq!(metadata.component, "Button");
    assert_eq!(
        metadata.attributes.get("text").and_then(Value::as_str),
        Some("Run")
    );
    assert_eq!(metadata.bindings.len(), 1);
    let slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == action.node_id)
        .expect("action slot should be projected");
    assert_eq!(slot.padding.left, 3.0);
    assert_eq!(
        slot.linear_sizing.as_ref().map(|sizing| sizing.rule),
        Some(zircon_runtime_interface::ui::layout::UiLinearSlotSizeRule::Auto)
    );
}

#[test]
fn ui_v2_surface_builder_infers_interaction_from_component_catalog() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/catalog_interaction.v2.ui"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalGroup"
children = [
    { node = "toggle" },
    { node = "range" },
    { node = "combo" },
    { node = "tree" },
    { node = "table" },
    { node = "message" },
    { node = "progress" },
]

[nodes.toggle]
component = "Toggle"
control_id = "DefaultToggle"
props = { checked = true }

[nodes.range]
component = "RangeField"
control_id = "DefaultRange"
props = { value = 42.0, min = 0.0, max = 100.0 }

[nodes.combo]
component = "ComboBox"
control_id = "DefaultCombo"
props = { value = "scene", options = ["scene", "asset"] }

[nodes.tree]
component = "TreeView"
control_id = "DefaultTree"
props = { selected_index = 0, expanded = true, items = ["Root"] }

[nodes.table]
component = "EditableTable"
control_id = "DefaultTable"
props = { selected_row = 0, selected_column = 0, rows = [], columns = [] }

[nodes.message]
component = "MessageBox"
control_id = "DefaultMessage"
props = { severity = "info", text = "Ready", rich_text = "<b>Ready</b>", open = true, actions = ["Dismiss"] }

[nodes.progress]
component = "ProgressBar"
control_id = "DefaultProgress"
props = { value = 0.5 }
"#,
    )
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.catalog_interaction"),
        &document,
        &compiled,
    )
    .unwrap();

    let node_by_control_id = |control_id: &str| {
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
    };

    for control_id in [
        "DefaultToggle",
        "DefaultRange",
        "DefaultCombo",
        "DefaultTree",
        "DefaultTable",
        "DefaultMessage",
    ] {
        let node = node_by_control_id(control_id);
        assert_eq!(node.input_policy, UiInputPolicy::Receive, "{control_id}");
        assert!(node.state_flags.clickable, "{control_id} clickable");
        assert!(node.state_flags.hoverable, "{control_id} hoverable");
        assert!(node.state_flags.focusable, "{control_id} focusable");
    }

    let progress = node_by_control_id("DefaultProgress");
    assert_eq!(progress.input_policy, UiInputPolicy::Inherit);
    assert!(!progress.state_flags.clickable);
    assert!(!progress.state_flags.hoverable);
    assert!(!progress.state_flags.focusable);
}

#[test]
fn ui_v2_virtual_list_window_uses_visible_range_and_overscan() {
    let window = compute_virtual_list_window(48.0, 96.0, 24.0, 100, 2);

    assert_eq!(window.first_visible, 0);
    assert_eq!(window.last_visible_exclusive, 8);
}

#[test]
fn ui_v2_composite_component_patches_root_props_and_fills_slots() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/composite.v2.ui"
version = 2

[root]
node = "root"

[components.Card]
root = "card_root"
default_classes = ["material-card"]
slots = { content = {} }

[nodes.root]
component = "Card"
control_id = "AssetCard"
classes = ["dense"]
props = { surface_variant = "outlined" }

[[nodes.root.children]]
node = "card_body"
slot = { name = "content" }

[nodes.card_root]
component = "VerticalGroup"
control_id = "CardPrototypeRoot"
props = { surface_variant = "filled" }

[[nodes.card_root.children]]
node = "card_title"

[[nodes.card_root.children]]
node = "card_content_slot"

[nodes.card_title]
component = "Text"
control_id = "CardTitle"
props = { text = "Prototype" }

[nodes.card_content_slot]
component = "Slot"
props = { name = "content" }

[nodes.card_body]
component = "Text"
control_id = "CardBody"
props = { text = "Instanced body" }
"#,
    )
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("expanded root"))
        .unwrap();

    assert_eq!(compiled.arena.node_count(), 3);
    assert_eq!(root.component, "VerticalGroup");
    assert_eq!(root.control_id.as_deref(), Some("AssetCard"));
    assert!(root.classes.iter().any(|class| class == "material-card"));
    assert!(root.classes.iter().any(|class| class == "dense"));
    assert_eq!(
        root.props.get("surface_variant").and_then(Value::as_str),
        Some("outlined")
    );
    assert!(compiled
        .arena
        .nodes
        .iter()
        .any(|node| node.control_id.as_deref() == Some("CardBody")));
}

#[test]
fn ui_v2_composite_component_validates_declared_slots() {
    let mut document = v2_document("asset://ui/tests/slot_validation.v2.ui", "root");
    document.components.insert(
        "Card".to_string(),
        zircon_runtime_interface::ui::v2::UiV2ComponentDefinition {
            root: "card_root".to_string(),
            slots: BTreeMap::from([(
                "content".to_string(),
                UiNamedSlotSchema {
                    required: true,
                    multiple: false,
                },
            )]),
            ..Default::default()
        },
    );
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Card".to_string(),
            children: vec![
                UiV2ChildMount {
                    node: "body_a".to_string(),
                    slot: BTreeMap::from([(
                        "name".to_string(),
                        Value::String("content".to_string()),
                    )]),
                },
                UiV2ChildMount {
                    node: "body_b".to_string(),
                    slot: BTreeMap::from([(
                        "name".to_string(),
                        Value::String("content".to_string()),
                    )]),
                },
            ],
            ..Default::default()
        },
    );
    document.nodes.insert(
        "card_root".to_string(),
        UiV2NodeDefinition {
            component: "Slot".to_string(),
            props: BTreeMap::from([("name".to_string(), Value::String("content".to_string()))]),
            ..Default::default()
        },
    );
    document.nodes.insert(
        "body_a".to_string(),
        UiV2NodeDefinition {
            component: "Text".to_string(),
            ..Default::default()
        },
    );
    document.nodes.insert(
        "body_b".to_string(),
        UiV2NodeDefinition {
            component: "Text".to_string(),
            ..Default::default()
        },
    );

    let error = UiV2DocumentCompiler::compile(&document).unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::SlotDoesNotAcceptMultiple { slot_name, .. } if slot_name == "content"
    ));
}

#[test]
fn ui_v2_composite_component_can_be_loaded_from_prototype_store() {
    let component = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/components/material_button.v2.ui"
version = 2

[components.MaterialButton]
root = "button_root"

[nodes.button_root]
component = "Button"
control_id = "PrototypeButton"
props = { text = "Prototype" }
"#,
    )
    .unwrap();
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/imported_component.v2.ui"
version = 2

[imports]
widgets = ["asset://ui/components/material_button.v2.ui#MaterialButton"]

[root]
node = "root"

[nodes.root]
component = "MaterialButton"
control_id = "ApplyDraft"
props = { text = "Apply Draft" }
"#,
    )
    .unwrap();
    let mut store = UiV2PrototypeStore::new();
    store.insert(component);

    let compiled = UiV2DocumentCompiler::compile_with_prototype_store(&document, &store).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("expanded root"))
        .unwrap();

    assert_eq!(compiled.arena.node_count(), 1);
    assert_eq!(root.component, "Button");
    assert_eq!(root.control_id.as_deref(), Some("ApplyDraft"));
    assert_eq!(
        root.props.get("text").and_then(Value::as_str),
        Some("Apply Draft")
    );
}

#[test]
fn ui_v2_file_cache_reuses_compiled_store_and_resolves_transitive_styles() {
    let temp_dir = v2_cache_temp_dir("res_alias_imports");
    let assets_root = temp_dir.join("assets");
    let layout_path = assets_root.join("ui/editor/layout.v2.ui.toml");
    let base_style_path = assets_root.join("ui/theme/base.v2.ui.toml");
    let material_style_path = assets_root.join("ui/theme/material.v2.ui.toml");
    std::fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(base_style_path.parent().unwrap()).unwrap();
    std::fs::write(
        &layout_path,
        r##"
[asset]
kind = "view"
id = "ui.editor.layout"
version = 2

[imports]
styles = ["res://ui/theme/base.v2.ui.toml"]

[root]
node = "root"

[nodes.root]
component = "Label"
control_id = "CacheRoot"
classes = ["cache-root"]
props = { text = "Cache" }
"##,
    )
    .unwrap();
    std::fs::write(
        &base_style_path,
        r##"
[asset]
kind = "style"
id = "ui.theme.base"
version = 2

[imports]
styles = ["res://ui/theme/material.v2.ui.toml"]

[tokens]
base_color = "$material_color"

[[stylesheets]]
id = "base"

[[stylesheets.rules]]
selector = ".cache-root"
set = { self = { foreground_color = "$base_color" } }
"##,
    )
    .unwrap();
    std::fs::write(
        &material_style_path,
        r##"
[asset]
kind = "style"
id = "ui.theme.material"
version = 2

[tokens]
material_color = "#abcdef"
"##,
    )
    .unwrap();
    let mut cache = UiV2PrototypeStoreFileCache::new();

    let first = cache.load_store(vec![layout_path.clone()]).unwrap();
    let second = cache.load_store(vec![layout_path]).unwrap();

    assert!(!first.cache_hit);
    assert!(second.cache_hit);
    assert_eq!(second.root_asset_id, "ui.editor.layout");
    assert!(second.store.get("res://ui/theme/base.v2.ui.toml").is_some());
    assert!(second
        .store
        .get("res://ui/theme/material.v2.ui.toml")
        .is_some());
    assert!(Arc::ptr_eq(&first.compiled, &second.compiled));
    assert_eq!(cache.len(), 1);

    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.file_cache"),
        second.root_document.as_ref(),
        second.compiled.as_ref(),
    )
    .unwrap();
    let root = surface.tree.nodes.values().next().unwrap();
    let metadata = root.template_metadata.as_ref().unwrap();
    assert_eq!(
        metadata
            .attributes
            .get("foreground_color")
            .and_then(Value::as_str),
        Some("#abcdef")
    );

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn ui_v2_file_cache_resolves_builtin_asset_id_widget_imports() {
    let temp_dir = v2_cache_temp_dir("asset_id_widget_imports");
    let assets_root = temp_dir.join("assets");
    let window_path = assets_root.join("ui/editor/windows/workbench_window.v2.ui.toml");
    let component_path = assets_root.join("ui/editor/host/activity_drawer_window.zui");
    std::fs::create_dir_all(window_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(component_path.parent().unwrap()).unwrap();
    std::fs::write(
        &window_path,
        r##"
[asset]
kind = "view"
id = "editor.window.workbench"
version = 2

[imports]
widgets = ["editor.host.activity_drawer_window#ActivityDrawerWindow"]

[root]
node = "root"

[nodes.root]
component = "ActivityDrawerWindow"
control_id = "WorkbenchWindow"
"##,
    )
    .unwrap();
    std::fs::write(
        &component_path,
        r##"
[asset]
kind = "component"
id = "editor.host.activity_drawer_window"
version = 2

[components.ActivityDrawerWindow]
root = "root"

[nodes.root]
component = "VerticalGroup"
control_id = "ActivityDrawerWindowRoot"
"##,
    )
    .unwrap();
    let mut cache = UiV2PrototypeStoreFileCache::new();

    let outcome = cache.load_store(vec![window_path]).unwrap();

    assert!(outcome
        .store
        .get("editor.host.activity_drawer_window")
        .is_some());
    assert!(outcome
        .store
        .get("res://ui/editor/host/activity_drawer_window.zui")
        .is_some());
    let root = outcome
        .compiled
        .arena
        .root
        .and_then(|handle| outcome.compiled.arena.node(handle))
        .expect("expanded root");
    assert_eq!(root.component, "VerticalGroup");
    assert_eq!(root.control_id.as_deref(), Some("WorkbenchWindow"));

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn ui_v2_file_cache_applies_zui_profile_for_uppercase_extension() {
    let temp_dir = v2_cache_temp_dir("uppercase_zui_profile");
    let assets_root = temp_dir.join("assets");
    let component_path = assets_root.join("ui/editor/Invalid.ZUI");
    std::fs::create_dir_all(component_path.parent().unwrap()).unwrap();
    std::fs::write(
        &component_path,
        r##"
[asset]
kind = "view"
id = "editor.invalid.uppercase_zui"
version = 2

[root]
node = "root"

[nodes.root]
component = "Container"
"##,
    )
    .unwrap();
    let mut cache = UiV2PrototypeStoreFileCache::new();

    let error = cache.load_store(vec![component_path]).unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("asset.kind")
    ));

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn ui_v2_file_cache_prefers_zui_asset_id_over_legacy_v2_document() {
    let temp_dir = v2_cache_temp_dir("asset_id_prefers_zui");
    let assets_root = temp_dir.join("assets");
    let window_path = assets_root.join("ui/editor/window.v2.ui.toml");
    let legacy_component_path = assets_root.join("ui/legacy/shared_component.v2.ui.toml");
    let zui_component_path = assets_root.join("ui/components/shared_component.zui");
    std::fs::create_dir_all(window_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(legacy_component_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(zui_component_path.parent().unwrap()).unwrap();
    std::fs::write(
        &window_path,
        r##"
[asset]
kind = "view"
id = "editor.window.prefers_zui"
version = 2

[imports]
widgets = ["editor.shared.component#SharedComponent"]

[root]
node = "root"

[nodes.root]
component = "SharedComponent"
control_id = "WindowRoot"
"##,
    )
    .unwrap();
    std::fs::write(
        &legacy_component_path,
        r##"
[asset]
kind = "component"
id = "editor.shared.component"
version = 2

[components.SharedComponent]
root = "legacy_root"

[nodes.legacy_root]
component = "Label"
control_id = "LegacyComponentRoot"
props = { text = "Legacy" }
"##,
    )
    .unwrap();
    std::fs::write(
        &zui_component_path,
        r##"
[asset]
kind = "component"
id = "editor.shared.component"
version = 2

[components.SharedComponent]
root = "zui_root"

[nodes.zui_root]
component = "Button"
control_id = "ZuiComponentRoot"
props = { text = "Zui" }
"##,
    )
    .unwrap();
    let mut cache = UiV2PrototypeStoreFileCache::new();

    let outcome = cache.load_store(vec![window_path]).unwrap();

    assert!(outcome
        .store
        .get("res://ui/components/shared_component.zui")
        .is_some());
    assert!(outcome
        .store
        .get("res://ui/legacy/shared_component.v2.ui.toml")
        .is_none());
    let root = outcome
        .compiled
        .arena
        .root
        .and_then(|handle| outcome.compiled.arena.node(handle))
        .expect("expanded root");
    assert_eq!(root.component, "Button");
    assert_eq!(root.control_id.as_deref(), Some("WindowRoot"));
    assert_eq!(root.props.get("text").and_then(Value::as_str), Some("Zui"));

    let _ = std::fs::remove_dir_all(temp_dir);
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

fn v2_cache_temp_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "zircon_ui_v2_store_{test_name}_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
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

fn assert_range_value(surface: &crate::ui::surface::UiSurface, node_id: UiNodeId, expected: f64) {
    let value = surface
        .tree
        .nodes
        .get(&node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get("value")
        .and_then(Value::as_float)
        .unwrap();
    assert!(
        (value - expected).abs() < f64::EPSILON,
        "expected range value {expected}, got {value}"
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
    let welcome_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../zircon_editor/assets/ui/editor/welcome.v2.ui.toml");
    let mut cache = UiV2PrototypeStoreFileCache::new();
    let outcome = cache.load_store(vec![welcome_path]).unwrap();
    UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new(tree_id),
        outcome.root_document.as_ref(),
        outcome.compiled.as_ref(),
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
