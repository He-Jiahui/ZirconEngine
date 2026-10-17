use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiFrame,
    template::UiActionRef,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

use super::*;
use crate::ui::template_runtime::{RetainedUiHostNodeProjection, RetainedUiNodeProjection};

fn collect_projection_nodes<'a>(
    node: &'a RetainedUiNodeProjection,
    output: &mut Vec<&'a RetainedUiNodeProjection>,
) {
    output.push(node);
    for child in &node.children {
        collect_projection_nodes(child, output);
    }
}

#[test]
fn builtin_workbench_projection_keeps_authored_owner_and_component_callsite() {
    let mut runtime = EditorUiHostRuntime::default();
    let document_id = crate::ui::template_runtime::builtin::UNREAL_EDITOR_WINDOW_DOCUMENT_ID;
    runtime
        .load_builtin_host_templates_for_document_ids(&[document_id])
        .expect("builtin workbench documents should register through the V2 file cache");
    let projection = runtime
        .project_document(document_id)
        .expect("builtin workbench should project");

    assert_eq!(
        projection.root.source_path.as_deref(),
        Some("zircon_editor/assets/ui/editor/unreal/level_editor_window.zui")
    );
    assert_eq!(projection.root.source_node_id.as_deref(), Some("root"));
    assert!(projection
        .root
        .instance_path
        .as_ref()
        .is_some_and(Vec::is_empty));

    let mut nodes = Vec::new();
    collect_projection_nodes(&projection.root, &mut nodes);
    let popup_root = nodes
        .into_iter()
        .find(|node| {
            node.source_path.as_deref()
                == Some(
                    "zircon_editor/assets/ui/editor/workbench/primitives/feedback/popup_menu.zui",
                )
        })
        .expect("an imported popup component node should retain its source owner");
    assert_eq!(
        popup_root.instance_path.as_ref().map(|steps| steps
            .iter()
            .map(|step| step.source_node_id.as_str())
            .collect::<Vec<_>>()),
        Some(vec!["toolbar_main_menu"])
    );
    assert_eq!(
        popup_root
            .instance_path
            .as_ref()
            .map(|steps| { steps[0].source_path.as_str() }),
        Some("zircon_editor/assets/ui/editor/unreal/level_editor_window.zui")
    );
}

#[test]
fn pane_control_state_projects_rows_selection_and_disabled_to_native_and_retained_models() {
    let node_id = UiNodeId::new(1);
    let mut surface = UiSurface::new(UiTreeId::new("editor.template.v2.pane-state"));
    surface.tree.insert_root(
        UiTreeNode::new(node_id, UiNodePath::new("root/RowList")).with_template_metadata(
            UiTemplateNodeMetadata {
                component: "Table".to_string(),
                control_id: Some("RowList".to_string()),
                ..Default::default()
            },
        ),
    );
    let rows = Value::Array(vec![Value::Table(toml::map::Map::from_iter([(
        "surface_entity".to_string(),
        Value::Integer(73),
    )]))]);
    let control_attributes = BTreeMap::from([(
        "RowList".to_string(),
        BTreeMap::from([
            ("rows".to_string(), rows.clone()),
            ("selected_row_identity".to_string(), Value::Integer(73)),
            ("disabled".to_string(), Value::Boolean(true)),
            ("enabled".to_string(), Value::Boolean(true)),
        ]),
    )]);
    let mut host_model = RetainedUiHostModel {
        document_id: "plugin.rows.panel".to_string(),
        nodes: vec![RetainedUiHostNodeProjection {
            node_id: "root/RowList".to_string(),
            surface_node_id: None,
            has_workbench_icon_tooltip: false,
            parent_id: None,
            component: "Table".to_string(),
            control_id: Some("RowList".to_string()),
            source_path: None,
            source_node_id: None,
            instance_path: None,
            parent_source_path: None,
            parent_source_node_id: None,
            parent_instance_path: None,
            frame: UiFrame::default(),
            clip_frame: None,
            z_index: 0,
            attributes: BTreeMap::new(),
            style_overrides: BTreeMap::new(),
            style_tokens: BTreeMap::new(),
            bindings: Vec::new(),
        }],
    };

    dynamic_control_state::apply_template_control_attributes_to_host_model(
        &host_model.document_id.clone(),
        &mut host_model,
        &control_attributes,
    )
    .expect("retained host should receive the current pane control state");
    dynamic_control_state::apply_template_control_attributes_to_surface(
        &host_model.document_id,
        &mut surface,
        &control_attributes,
    )
    .expect("native surface should receive the current pane control state");

    assert_eq!(
        host_model
            .node_by_control_id("RowList")
            .and_then(|node| node.attributes.get("rows")),
        Some(&rows)
    );
    assert_eq!(
        host_model
            .node_by_control_id("RowList")
            .and_then(|node| node.attributes.get("selected_row_identity")),
        Some(&Value::Integer(73))
    );
    assert_eq!(
        surface
            .component_state(node_id)
            .and_then(|state| state.value("selected_row_identity")),
        Some(&UiValue::Int(73))
    );
    assert_eq!(
        surface
            .tree
            .node(node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.attributes.get("rows")),
        Some(&rows)
    );
    assert!(
        !surface
            .tree
            .node(node_id)
            .expect("native control should remain in the surface")
            .state_flags
            .enabled
    );
}

#[test]
fn plugin_document_replacement_evicts_same_id_action_slots_before_pane_rebuild() {
    let runtime = EditorUiHostRuntime::default();
    let source_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/unreal/editor_shell.zui");
    let source = |document_id| {
        super::super::plugin_documents::EditorPluginV2DocumentSource::new(
            document_id,
            "plugins://fixture.plugin/workbench_shell.zui",
            [source_path.clone()],
        )
        .expect("fixture plugin document source should be valid")
    };
    let first =
        super::super::plugin_documents::EditorPluginV2DocumentOwner::new("fixture.plugin", 1)
            .expect("first plugin generation should be valid");
    let second =
        super::super::plugin_documents::EditorPluginV2DocumentOwner::new("fixture.plugin", 2)
            .expect("replacement plugin generation should be valid");
    runtime
        .replace_plugin_v2_documents(first.clone(), [source("fixture.plugin.panel")])
        .expect("first plugin generation should load");
    let token = runtime
        .template_action_registry
        .lock()
        .expect("template action registry mutex should not be poisoned")
        .bind(
            "fixture.plugin.pane",
            "fixture.plugin.panel",
            "Bake/Click",
            Some(first),
            BTreeMap::new(),
            UiActionRef {
                route: Some("fixture.operation".to_string()),
                action: None,
                payload: BTreeMap::new(),
                payload_missing_policy: Default::default(),
            },
            BTreeMap::new(),
        );

    let update = runtime
        .replace_plugin_v2_documents(second, [source("fixture.plugin.panel")])
        .expect("same-id replacement should load the next generation");

    assert!(update.retired_document_ids().is_empty());
    assert!(!runtime
        .template_action_registry
        .lock()
        .expect("template action registry mutex should not be poisoned")
        .contains_token(&token));
}

#[test]
fn module_plugin_detail_patches_bind_current_rows_and_actions_to_both_models() {
    use crate::ui::layouts::windows::workbench_host_window::{
        ModulePluginDetailsPayload, ModulePluginFeatureRowPayload, ModulePluginsPanePayload,
        PaneBodyPresentation, PanePayload,
    };
    use crate::ui::workbench::view::{PaneInteractionMode, PanePayloadKind, PaneRouteNamespace};
    use zircon_runtime_interface::ui::layout::UiSize;

    let document_id = "res://ui/editor/host/module_plugins_body.zui";
    let mut runtime = EditorUiHostRuntime::default();
    runtime
        .load_builtin_host_templates_for_document_ids(&[document_id])
        .unwrap();
    let mut body = PaneBodyPresentation {
        document_id: document_id.into(),
        payload_kind: PanePayloadKind::ModulePluginsV1,
        route_namespace: PaneRouteNamespace::Template,
        interaction_mode: PaneInteractionMode::HybridNativeSlot,
        payload: PanePayload::ModulePluginsV1(ModulePluginsPanePayload {
            diagnostics: String::new(),
            plugins: vec![],
            detail_generation: 1,
            detail_locale: "en".into(),
            selected_plugin_id: "vendor.owner".into(),
            focused_control: "features".into(),
            details: Some(ModulePluginDetailsPayload {
                plugin_id: "vendor.owner".into(),
                title: "Owner".into(),
                total_features: 2,
                rows: ["first.feature", "second.feature"]
                    .into_iter()
                    .map(|id| ModulePluginFeatureRowPayload {
                        feature_id: id.into(),
                        label: format!(
                            "{id}\nDesired: No\nEffective: Unknown\nRequired: No\nBlocked: No"
                        ),
                        effective: "Unknown".into(),
                        ..Default::default()
                    })
                    .collect(),
                selected_feature_id: "second.feature".into(),
                selected_identity: "Owner: vendor.owner\nFeature: second.feature".into(),
                selected_states: "Desired: No\nEffective: Unknown\nRequired: No\nBlocked: No"
                    .into(),
                action_label: "Enable Feature".into(),
                action_id: "qualified-owner-feature-action".into(),
                intent_id: "intent-current".into(),
                ..Default::default()
            }),
        }),
    };
    let build = |body: &PaneBodyPresentation| {
        let projection = runtime.project_pane_body(body).unwrap();
        let mut surface = runtime.build_shared_surface(document_id).unwrap();
        runtime
            .apply_pane_component_patches_to_surface(body, &mut surface)
            .unwrap();
        surface.compute_layout(UiSize::new(320.0, 520.0)).unwrap();
        let mut model = runtime
            .build_host_model_with_surface(&projection, &surface)
            .unwrap();
        runtime
            .bind_template_actions_for_pane("editor.module_plugins#dual", &mut surface, &mut model)
            .unwrap();
        (surface, model)
    };
    let (surface, model) = build(&body);
    let table = model.node_by_control_id("ModulePluginFeatureRows").unwrap();
    let node = surface.tree.node(table.surface_node_id.unwrap()).unwrap();
    for key in [
        "rows",
        "row_identity_field",
        "selected_row_identity",
        "detail_generation",
    ] {
        assert_eq!(
            table.attributes.get(key),
            node.template_metadata.as_ref().unwrap().attributes.get(key),
            "{key}"
        );
    }
    assert_eq!(
        table.attributes.get("selected_row_identity"),
        Some(&Value::String("second.feature".into()))
    );
    let apply = model
        .node_by_control_id("ModulePluginDetailsApply")
        .unwrap();
    assert_eq!(
        apply.attributes.get("disabled"),
        Some(&Value::Boolean(false))
    );
    let token = apply
        .bindings
        .iter()
        .find(|binding| !binding.action_id.is_empty())
        .unwrap()
        .action_id
        .clone();
    let invocation = runtime
        .dispatch_template_action_for_token(&token, Clone::clone)
        .unwrap();
    assert_eq!(invocation.target_id(), "workbench.plugin.details.apply");
    assert_eq!(
        invocation.payload.get("intent"),
        Some(&UiValue::String("intent-current".into()))
    );
    assert!(runtime.select_template_table_row(
        "editor.module_plugins#dual",
        "ModulePluginFeatureRows",
        1,
        "string",
        "second.feature"
    ));
    assert!(!runtime.select_template_table_row(
        "other-pane",
        "ModulePluginFeatureRows",
        1,
        "string",
        "second.feature"
    ));
    assert!(!runtime.select_template_table_row(
        "editor.module_plugins#dual",
        "ModulePluginFeatureRows",
        1,
        "string",
        "first.feature"
    ));

    let PanePayload::ModulePluginsV1(payload) = &mut body.payload else {
        unreachable!()
    };
    payload.detail_generation += 1;
    payload.focused_control = "back".into();
    let detail = payload.details.as_mut().unwrap();
    detail.plugin_id = "another.owner".into();
    detail.selected_feature_id.clear();
    detail.action_id.clear();
    detail.intent_id.clear();
    let (surface, model) = build(&body);
    assert!(runtime
        .dispatch_template_action_for_token(&token, Clone::clone)
        .is_none());
    let apply = model
        .node_by_control_id("ModulePluginDetailsApply")
        .unwrap();
    assert_eq!(
        apply.attributes.get("disabled"),
        Some(&Value::Boolean(true))
    );
    assert!(
        !surface
            .tree
            .node(apply.surface_node_id.unwrap())
            .unwrap()
            .state_flags
            .enabled
    );
    let table = model.node_by_control_id("ModulePluginFeatureRows").unwrap();
    assert_eq!(
        table.attributes.get("selected_row_identity"),
        Some(&Value::String(String::new()))
    );
    assert!(table.attributes.get("selected_index").is_none());
    assert_eq!(
        table
            .attributes
            .get("rows")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(2)
    );
}
