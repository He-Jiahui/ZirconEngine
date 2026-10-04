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
    let document_id = crate::ui::template_runtime::builtin::WORKBENCH_WINDOW_DOCUMENT_ID;
    runtime
        .load_builtin_host_templates_for_document_ids(&[document_id])
        .expect("builtin workbench documents should register through the V2 file cache");
    let projection = runtime
        .project_document(document_id)
        .expect("builtin workbench should project");

    assert_eq!(
        projection.root.source_path.as_deref(),
        Some("zircon_editor/assets/ui/editor/windows/workbench_window.zui")
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
                    "zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_popup_menu.zui",
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
        Some("zircon_editor/assets/ui/editor/windows/workbench_window.zui")
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
        .join("assets/ui/host/workbench_shell.zui");
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
