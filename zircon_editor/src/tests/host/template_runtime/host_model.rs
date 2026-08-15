use std::collections::BTreeMap;

use super::support::*;
use crate::ui::template_runtime::{RetainedUiHostModel, RetainedUiHostNodeProjection};
use zircon_runtime::ui::dispatch::UiNavigationDispatcher;
use zircon_runtime_interface::ui::surface::UiNavigationEventKind;

const COMPONENT_SHOWCASE_DOCUMENT_ID: &str = "res://ui/editor/component_showcase.zui";
const WORKBENCH_WINDOW_DOCUMENT_ID: &str = "res://ui/editor/windows/workbench_window.zui";
const OPEN_PROJECT_ICON: &str = "editor_pages/workbench/menu/open-project.svg";

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
        Some(&Value::String(OPEN_PROJECT_ICON.to_string()))
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
        .contains(&format!("root.0.0.0.icon={OPEN_PROJECT_ICON}")));
    assert!(snapshot
        .attribute_entries
        .contains(&"root.0.4.0.text=Ready".to_string()));
}

#[test]
fn surface_backed_host_model_preserves_component_state_attributes_for_semantics() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(COMPONENT_SHOWCASE_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let mut surface = runtime
        .build_shared_surface(COMPONENT_SHOWCASE_DOCUMENT_ID)
        .unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();

    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();

    let enabled_button = host_model.node_by_control_id("ButtonDemo").unwrap();
    assert_eq!(
        enabled_button
            .attributes
            .get("disabled")
            .and_then(Value::as_bool),
        Some(false),
        "surface-backed host model should keep explicit false state for component semantics"
    );

    let disabled_button = host_model.node_by_control_id("ButtonDisabledDemo").unwrap();
    assert_eq!(
        disabled_button
            .attributes
            .get("disabled")
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn surface_backed_retained_projection_preserves_focus_visible_reason() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let projection = runtime
        .project_document(COMPONENT_SHOWCASE_DOCUMENT_ID)
        .unwrap();
    let mut surface = runtime
        .build_shared_surface(COMPONENT_SHOWCASE_DOCUMENT_ID)
        .unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();

    let button_id = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("ButtonDemo")
        })
        .map(|node| node.node_id)
        .expect("component showcase button must remain focusable");
    surface.focus_node(button_id).unwrap();

    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();
    let host_button = host_model.node_by_control_id("ButtonDemo").unwrap();
    assert_eq!(
        host_button
            .attributes
            .get("focused")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        host_button
            .attributes
            .get("focus_visible")
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        host_button
            .attributes
            .get("focus_visible_known")
            .and_then(Value::as_bool),
        Some(true)
    );

    let retained_projection = RetainedUiHostAdapter::build_projection(&host_model);
    let retained_button = retained_projection
        .node_by_control_id("ButtonDemo")
        .unwrap();
    assert!(retained_button.focused);
    assert!(!retained_button.focus_visible);
    assert!(retained_button.focus_visible_known);

    let navigation_target = surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap()
        .focus_changed_to
        .expect("component showcase must provide a subsequent navigation target");
    let navigation_target_path = surface
        .tree
        .node(navigation_target)
        .expect("navigation target must remain in the surface tree")
        .node_path
        .0
        .clone();
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();
    let prior_button = host_model.node_by_control_id("ButtonDemo").unwrap();
    assert_eq!(
        prior_button
            .attributes
            .get("focused")
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        prior_button
            .attributes
            .get("focus_visible")
            .and_then(Value::as_bool),
        Some(false)
    );
    let navigation_node = host_model
        .nodes
        .iter()
        .find(|node| node.node_id == navigation_target_path)
        .expect("surface navigation target must be projected into the host model");
    assert_eq!(
        navigation_node
            .attributes
            .get("focused")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        navigation_node
            .attributes
            .get("focus_visible")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        navigation_node
            .attributes
            .get("focus_visible_known")
            .and_then(Value::as_bool),
        Some(true)
    );

    let retained_projection = RetainedUiHostAdapter::build_projection(&host_model);
    let retained_navigation_node = retained_projection
        .nodes
        .iter()
        .find(|node| node.node_id == navigation_target_path)
        .expect("navigation target must be retained after adapter projection");
    assert!(retained_navigation_node.focused);
    assert!(retained_navigation_node.focus_visible);
    assert!(retained_navigation_node.focus_visible_known);
}

#[test]
fn authored_focus_preview_remains_unknown_to_runtime_focus_projection() {
    let document_id = "res://ui/editor/material_demo_window.zui";
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let projection = runtime.project_document(document_id).unwrap();

    let authored_projection = runtime.build_retained_host_projection(&projection).unwrap();
    let authored_button = authored_projection
        .node_by_control_id("PrimaryButton")
        .unwrap();
    assert!(authored_button.focused);
    assert!(!authored_button.focus_visible);
    assert!(!authored_button.focus_visible_known);

    let mut surface = runtime.build_shared_surface(document_id).unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();
    let surface_projection = runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();
    let surface_button = surface_projection
        .node_by_control_id("PrimaryButton")
        .unwrap();
    assert!(surface_button.focused);
    assert!(!surface_button.focus_visible);
    assert!(!surface_button.focus_visible_known);
}

#[test]
fn surface_backed_retained_projection_exposes_style_overrides_as_effective_properties() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(WORKBENCH_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let mut surface = runtime
        .build_shared_surface(WORKBENCH_WINDOW_DOCUMENT_ID)
        .unwrap();
    surface.compute_layout(UiSize::new(1672.0, 941.0)).unwrap();

    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();
    let retained_projection = RetainedUiHostAdapter::build_projection(&host_model);

    let host_gizmo = host_model
        .node_by_control_id("WorkbenchViewportGizmoX")
        .unwrap();
    assert_eq!(
        host_gizmo
            .style_overrides
            .get("foreground_color")
            .and_then(Value::as_str),
        Some("#ef493f")
    );
    assert_eq!(
        host_gizmo
            .attributes
            .get("foreground_color")
            .and_then(Value::as_str),
        Some("#d8e3e7")
    );

    let retained_gizmo = retained_projection
        .node_by_control_id("WorkbenchViewportGizmoX")
        .unwrap();
    assert_eq!(
        retained_gizmo.properties.get("foreground_color"),
        Some(&RetainedUiHostValue::String("#ef493f".to_string()))
    );
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
    assert_eq!(open_project.icon.as_deref(), Some(OPEN_PROJECT_ICON));
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
        style_overrides: BTreeMap::new(),
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
        .contains(&format!("root.0.0.0={OPEN_PROJECT_ICON}")));
    assert!(snapshot
        .route_bindings
        .iter()
        .any(|entry: &String| entry.starts_with("WorkbenchMenuBar/OpenProject@")));
}
