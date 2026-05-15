use std::collections::BTreeMap;

use super::support::*;
use crate::ui::template_runtime::{RetainedUiHostModel, RetainedUiHostNodeProjection};

#[test]
fn editor_ui_compatibility_harness_captures_projection_shape_for_parity_checks() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();

    let snapshot = EditorUiCompatibilityHarness::capture_projection_snapshot(&projection);

    assert_eq!(
        snapshot.components,
        vec![
            "UiHostWindow",
            "VerticalGroup",
            "UiHostToolbar",
            "IconButton",
            "IconButton",
            "IconButton",
            "Container",
            "Container",
            "HorizontalGroup",
            "ActivityRail",
            "IconButton",
            "IconButton",
            "IconButton",
            "DocumentHost",
            "DocumentTabs",
            "PaneSurface",
            "StatusBar",
            "Label",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
        ]
    );
    assert!(snapshot
        .control_ids
        .contains(&"WorkbenchMenuBarRoot".to_string()));
    assert!(snapshot
        .binding_ids
        .contains(&"WorkbenchMenuBar/ResetLayout".to_string()));
}

#[test]
fn editor_ui_host_runtime_builds_host_node_model_with_routes_and_attributes() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();

    let host_model = runtime.build_host_model(&projection).unwrap();

    assert_eq!(host_model.document_id, UI_HOST_WINDOW_DOCUMENT_ID);
    assert_eq!(
        host_model
            .nodes
            .iter()
            .map(|node| node.component.as_str())
            .collect::<Vec<_>>(),
        vec![
            "UiHostWindow",
            "VerticalGroup",
            "UiHostToolbar",
            "IconButton",
            "IconButton",
            "IconButton",
            "Container",
            "Container",
            "HorizontalGroup",
            "ActivityRail",
            "IconButton",
            "IconButton",
            "IconButton",
            "DocumentHost",
            "DocumentTabs",
            "PaneSurface",
            "StatusBar",
            "Label",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
        ]
    );

    let open_project = host_model
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("OpenProject"))
        .unwrap();
    assert_eq!(open_project.node_id, "root.0.0.0");
    assert_eq!(open_project.parent_id.as_deref(), Some("root.0.0"));
    assert_eq!(
        open_project.attributes.get("icon"),
        Some(&Value::String("folder-open-outline".to_string()))
    );
    assert_eq!(
        open_project.attributes.get("label"),
        Some(&Value::String("Open".to_string()))
    );
    let open_project_binding = open_project
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "WorkbenchMenuBar/OpenProject")
        .unwrap();
    assert_eq!(open_project_binding.event_kind, UiEventKind::Click);
    let route_id = open_project_binding.route_id.expect("registered route");
    assert_eq!(
        service.route_binding(route_id).unwrap(),
        projection
            .bindings
            .iter()
            .find(|binding| binding.binding_id == "WorkbenchMenuBar/OpenProject")
            .unwrap()
            .binding
            .as_ui_binding()
    );

    let status_text = host_model
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("StatusText"))
        .unwrap();
    assert_eq!(status_text.node_id, "root.0.4.0");
    assert_eq!(
        status_text.attributes.get("text"),
        Some(&Value::String("Ready".to_string()))
    );
}

#[test]
fn editor_ui_compatibility_harness_captures_host_model_routes_and_attributes() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let host_model = runtime.build_host_model(&projection).unwrap();

    let snapshot = EditorUiCompatibilityHarness::capture_host_model_snapshot(&host_model);

    assert!(snapshot
        .host_nodes
        .contains(&"root.0.0.0|IconButton|OpenProject".to_string()));
    assert!(snapshot
        .host_nodes
        .contains(&"root.0.4.0|Label|StatusText".to_string()));
    assert!(snapshot
        .route_bindings
        .iter()
        .any(|entry: &String| entry.starts_with("WorkbenchMenuBar/OpenProject@")));
    assert!(snapshot
        .attribute_entries
        .contains(&"root.0.0.0.icon=folder-open-outline".to_string()));
    assert!(snapshot
        .attribute_entries
        .contains(&"root.0.4.0.text=Ready".to_string()));
}

#[test]
fn retained_ui_host_adapter_builds_generic_projection_from_host_model() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let host_model = runtime.build_host_model(&projection).unwrap();

    let retained_projection = RetainedUiHostAdapter::build_projection(&host_model);

    assert_eq!(retained_projection.document_id, UI_HOST_WINDOW_DOCUMENT_ID);
    assert_eq!(
        retained_projection
            .nodes
            .iter()
            .map(|node| node.kind)
            .collect::<Vec<_>>(),
        vec![
            RetainedUiHostComponentKind::Root,
            RetainedUiHostComponentKind::VerticalBox,
            RetainedUiHostComponentKind::Toolbar,
            RetainedUiHostComponentKind::IconButton,
            RetainedUiHostComponentKind::IconButton,
            RetainedUiHostComponentKind::IconButton,
            RetainedUiHostComponentKind::Unknown,
            RetainedUiHostComponentKind::Unknown,
            RetainedUiHostComponentKind::HorizontalBox,
            RetainedUiHostComponentKind::ActivityRail,
            RetainedUiHostComponentKind::IconButton,
            RetainedUiHostComponentKind::IconButton,
            RetainedUiHostComponentKind::IconButton,
            RetainedUiHostComponentKind::DocumentHost,
            RetainedUiHostComponentKind::TabStrip,
            RetainedUiHostComponentKind::PaneSurface,
            RetainedUiHostComponentKind::StatusBar,
            RetainedUiHostComponentKind::Label,
            RetainedUiHostComponentKind::Unknown,
            RetainedUiHostComponentKind::Unknown,
            RetainedUiHostComponentKind::Unknown,
            RetainedUiHostComponentKind::Unknown,
            RetainedUiHostComponentKind::Unknown,
            RetainedUiHostComponentKind::Unknown,
            RetainedUiHostComponentKind::Unknown,
            RetainedUiHostComponentKind::Unknown,
            RetainedUiHostComponentKind::Unknown,
        ]
    );

    let open_project = retained_projection
        .node_by_control_id("OpenProject")
        .unwrap();
    assert_eq!(open_project.node_id, "root.0.0.0");
    assert_eq!(open_project.parent_id.as_deref(), Some("root.0.0"));
    assert_eq!(open_project.kind, RetainedUiHostComponentKind::IconButton);
    assert_eq!(open_project.text.as_deref(), Some("Open"));
    assert_eq!(open_project.icon.as_deref(), Some("folder-open-outline"));
    assert_eq!(
        open_project.properties.get("label").cloned().unwrap(),
        RetainedUiHostValue::String("Open".to_string())
    );
    let click_route = open_project
        .routes
        .iter()
        .find(|route| route.binding_id == "WorkbenchMenuBar/OpenProject")
        .unwrap()
        .route_id
        .expect("click route");
    assert_eq!(
        service.route_binding(click_route).unwrap(),
        projection
            .bindings
            .iter()
            .find(|binding| binding.binding_id == "WorkbenchMenuBar/OpenProject")
            .unwrap()
            .binding
            .as_ui_binding()
    );

    let status_text = retained_projection
        .node_by_control_id("StatusText")
        .unwrap();
    assert_eq!(status_text.kind, RetainedUiHostComponentKind::Label);
    assert_eq!(status_text.text.as_deref(), Some("Ready"));
    assert_eq!(status_text.icon, None);
}

#[test]
fn retained_ui_host_adapter_keeps_field_labels_out_of_visible_text() {
    let host_model = RetainedUiHostModel {
        document_id: "field.label.contract".to_string(),
        nodes: vec![
            retained_host_node(
                "InputField",
                "NameField",
                BTreeMap::from([
                    ("label".to_string(), Value::String("Name".to_string())),
                    ("placeholder".to_string(), Value::String("Name".to_string())),
                    ("value_text".to_string(), Value::String(String::new())),
                ]),
            ),
            retained_host_node(
                "IconButton",
                "OpenProject",
                BTreeMap::from([
                    ("label".to_string(), Value::String("Open".to_string())),
                    (
                        "icon".to_string(),
                        Value::String("folder-open-outline".to_string()),
                    ),
                ]),
            ),
        ],
    };

    let retained_projection = RetainedUiHostAdapter::build_projection(&host_model);

    let name_field = retained_projection.node_by_control_id("NameField").unwrap();
    assert_eq!(name_field.text.as_deref(), None);
    assert_eq!(
        name_field.properties.get("placeholder"),
        Some(&RetainedUiHostValue::String("Name".to_string()))
    );

    let open_project = retained_projection
        .node_by_control_id("OpenProject")
        .unwrap();
    assert_eq!(open_project.text.as_deref(), Some("Open"));
}

fn retained_host_node(
    component: &str,
    control_id: &str,
    attributes: BTreeMap<String, Value>,
) -> RetainedUiHostNodeProjection {
    RetainedUiHostNodeProjection {
        node_id: control_id.to_string(),
        parent_id: None,
        component: component.to_string(),
        control_id: Some(control_id.to_string()),
        frame: UiFrame::default(),
        clip_frame: None,
        z_index: 0,
        attributes,
        style_tokens: BTreeMap::new(),
        bindings: Vec::new(),
    }
}

#[test]
fn editor_ui_host_runtime_builds_retained_host_projection_and_snapshot() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();

    let retained_projection = runtime.build_retained_host_projection(&projection).unwrap();
    let snapshot = EditorUiCompatibilityHarness::capture_retained_host_projection_snapshot(
        &retained_projection,
    );

    assert_eq!(retained_projection.nodes.len(), 27);
    assert!(snapshot
        .retained_nodes
        .contains(&"root.0.0.0|IconButton|OpenProject".to_string()));
    assert!(snapshot
        .retained_nodes
        .contains(&"root.0.4.0|Label|StatusText".to_string()));
    assert!(snapshot
        .text_entries
        .contains(&"root.0.0.0=Open".to_string()));
    assert!(snapshot
        .icon_entries
        .contains(&"root.0.0.0=folder-open-outline".to_string()));
    assert!(snapshot
        .route_bindings
        .iter()
        .any(|entry: &String| entry.starts_with("WorkbenchMenuBar/OpenProject@")));
}
