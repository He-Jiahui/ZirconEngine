use std::fs;
use std::path::Path;

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::binding::UiEventKind;

use crate::tests::support::load_test_ui_asset;
use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::host::module::{self, module_descriptor, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorManager;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, SlintUiHostNodeModel, SlintUiHostValue,
};
use crate::ui::workbench::view::ViewDescriptorId;

fn editor_runtime() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime.store_config_value(
        crate::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
        serde_json::json!([
            crate::ui::host::EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
            crate::ui::host::EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING,
        ]),
    );
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    runtime
}

fn pane_body_path(file_name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("editor")
        .join("host")
        .join(file_name)
}

fn editor_component_showcase_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("editor")
        .join("component_showcase.ui.toml")
}

fn collect_showcase_prop_schema_mismatches(
    node: &toml::Value,
    registry: &UiComponentDescriptorRegistry,
    mismatches: &mut Vec<String>,
) {
    if let Some(component_type) = node.get("type").and_then(toml::Value::as_str) {
        if let Some(descriptor) = registry.descriptor(component_type) {
            if let Some(props) = node.get("props").and_then(toml::Value::as_table) {
                for prop in props.keys() {
                    if descriptor.prop(prop).is_none() {
                        let control_id = node
                            .get("control_id")
                            .and_then(toml::Value::as_str)
                            .unwrap_or("<missing-control-id>");
                        mismatches.push(format!("{control_id} `{component_type}.{prop}`"));
                    }
                }
            }
        }
    }

    if let Some(children) = node.get("children").and_then(toml::Value::as_array) {
        for child in children {
            if let Some(child_node) = child.get("node") {
                collect_showcase_prop_schema_mismatches(child_node, registry, mismatches);
            }
        }
    }
}

fn host_numeric_property(node: &SlintUiHostNodeModel, property: &str) -> Option<f64> {
    match node.properties.get(property) {
        Some(SlintUiHostValue::Float(value)) => Some(*value),
        Some(SlintUiHostValue::Integer(value)) => Some(*value as f64),
        _ => None,
    }
}

fn assert_host_numeric_property(node: &SlintUiHostNodeModel, property: &str, expected: f64) {
    let actual = host_numeric_property(node, property)
        .unwrap_or_else(|| panic!("missing numeric property `{property}` on `{}`", node.node_id));
    assert_eq!(actual, expected);
}

fn assert_material_button_layout_properties(node: &SlintUiHostNodeModel) {
    assert_host_numeric_property(node, "layout_padding_left", 24.0);
    assert_host_numeric_property(node, "layout_padding_right", 24.0);
    assert_host_numeric_property(node, "layout_padding_top", 10.0);
    assert_host_numeric_property(node, "layout_padding_bottom", 10.0);
    assert_host_numeric_property(node, "layout_spacing", 8.0);
    assert_host_numeric_property(node, "layout_min_width", 40.0);
    assert_host_numeric_property(node, "layout_min_height", 40.0);
    assert_host_numeric_property(node, "layout_icon_size", 18.0);
}

fn assert_material_field_layout_properties(node: &SlintUiHostNodeModel) {
    assert_host_numeric_property(node, "layout_padding_left", 16.0);
    assert_host_numeric_property(node, "layout_padding_right", 16.0);
    assert_host_numeric_property(node, "layout_padding_top", 4.0);
    assert_host_numeric_property(node, "layout_padding_bottom", 4.0);
    assert_host_numeric_property(node, "layout_min_height", 56.0);
}

fn assert_material_list_layout_properties(node: &SlintUiHostNodeModel) {
    assert_host_numeric_property(node, "layout_padding_left", 16.0);
    assert_host_numeric_property(node, "layout_padding_right", 16.0);
    assert_host_numeric_property(node, "layout_spacing", 8.0);
    assert_host_numeric_property(node, "layout_min_height", 40.0);
}

#[test]
fn builtin_activity_window_documents_are_registered_in_host_runtime() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut ui_runtime = EditorUiHostRuntime::default();
    ui_runtime.load_builtin_host_templates().unwrap();

    for document_id in [
        "editor.host.editor_main_frame",
        "editor.host.activity_drawer_window",
        "editor.window.workbench",
        "editor.window.asset",
        "editor.window.ui_layout_editor",
        "editor.window.ui_component_showcase",
    ] {
        let projection = ui_runtime
            .project_document(document_id)
            .unwrap_or_else(|error| panic!("failed to project `{document_id}`: {error}"));
        assert_eq!(projection.document_id, document_id);
        assert_eq!(projection.root.component, "VerticalBox");
    }
}

#[test]
fn component_showcase_authored_props_are_declared_by_runtime_catalog() {
    let source = fs::read_to_string(editor_component_showcase_path()).unwrap();
    let document: toml::Value = toml::from_str(&source).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let mut mismatches = Vec::new();

    collect_showcase_prop_schema_mismatches(
        document
            .get("root")
            .expect("component showcase asset should declare a root node"),
        &registry,
        &mut mismatches,
    );

    assert!(
        mismatches.is_empty(),
        "component_showcase.ui.toml has props missing from the runtime component catalog:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn component_showcase_projection_carries_runtime_component_semantics() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut ui_runtime = EditorUiHostRuntime::default();
    ui_runtime.load_builtin_host_templates().unwrap();

    let document_id = "editor.window.ui_component_showcase";
    let projection = ui_runtime.project_document(document_id).unwrap();
    let surface = ui_runtime.build_shared_surface(document_id).unwrap();
    let host_projection = ui_runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();

    let material_button = host_projection
        .node_by_control_id("ButtonDemo")
        .expect("component showcase should project ButtonDemo through the Material meta component");
    assert_eq!(material_button.component, "Button");
    assert_material_button_layout_properties(material_button);
    assert!(
        material_button.frame.width >= 84.0,
        "ButtonDemo arranged width should include text intrinsic width plus Material horizontal padding, got {}",
        material_button.frame.width
    );
    assert!(
        material_button.frame.height >= 40.0,
        "ButtonDemo arranged height should include Material control height, got {}",
        material_button.frame.height
    );

    let material_input = host_projection.node_by_control_id("InputFieldDemo").expect(
        "component showcase should project InputFieldDemo through the Material meta component",
    );
    assert_eq!(material_input.component, "InputField");
    assert_material_field_layout_properties(material_input);

    let number = host_projection
        .node_by_control_id("NumberFieldDemo")
        .expect("component showcase should project NumberFieldDemo through MaterialSpinBox");
    assert_eq!(number.component, "NumberField");
    assert_eq!(number.component_role.as_deref(), Some("number-field"));
    assert_eq!(number.value_text.as_deref(), Some("42"));
    assert_eq!(number.validation_level.as_deref(), Some("normal"));
    assert!(number.routes.iter().any(|route| {
        route.binding_id == "UiComponentShowcase/NumberFieldDragUpdate"
            && route.event_kind == UiEventKind::DragUpdate
    }));
    assert!(number.routes.iter().any(|route| {
        route.binding_id == "UiComponentShowcase/NumberFieldCommitted"
            && route.event_kind == UiEventKind::Submit
    }));

    let combo_box = host_projection
        .node_by_control_id("ComboBoxDemo")
        .expect("component showcase should project ComboBoxDemo through MaterialComboBox");
    assert_eq!(combo_box.component, "ComboBox");
    assert_material_field_layout_properties(combo_box);

    let material_list = host_projection
        .node_by_control_id("ListRowDemo")
        .expect("component showcase should project ListRowDemo through MaterialListItem");
    assert_eq!(material_list.component, "ListRow");
    assert_material_list_layout_properties(material_list);

    let dropdown = host_projection
        .node_by_control_id("DropdownDemo")
        .expect("component showcase should project DropdownDemo");
    assert_eq!(dropdown.component_role.as_deref(), Some("dropdown"));
    assert!(dropdown.popup_open);
    assert_eq!(dropdown.selection_state.as_deref(), Some("multi"));
    assert_eq!(
        dropdown.options_text.as_deref(),
        Some("runtime, editor, debug")
    );
    assert_eq!(
        dropdown.options,
        vec![
            "runtime".to_string(),
            "editor".to_string(),
            "debug".to_string()
        ]
    );
    assert!(dropdown.routes.iter().any(|route| {
        route.binding_id == "UiComponentShowcase/DropdownChanged"
            && route.event_kind == UiEventKind::Change
    }));

    let asset = host_projection
        .node_by_control_id("AssetFieldDemo")
        .expect("component showcase should project AssetFieldDemo");
    assert_eq!(asset.component_role.as_deref(), Some("asset-field"));
    assert!(asset
        .accepted_drag_payloads
        .iter()
        .any(|kind| kind == "asset"));
    assert!(asset.routes.iter().any(|route| {
        route.binding_id == "UiComponentShowcase/AssetFieldDropped"
            && route.event_kind == UiEventKind::Drop
    }));

    let drop_binding = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "UiComponentShowcase/AssetFieldDropped")
        .expect("showcase asset field drop binding should be projected");
    match drop_binding.binding.payload() {
        EditorUiBindingPayload::Custom(call) => {
            assert_eq!(call.symbol, "UiComponentShowcase");
            assert_eq!(
                call.argument(0).and_then(|value| value.as_str()),
                Some("DropReference.AssetField")
            );
        }
        other => panic!("unexpected showcase binding payload: {other:?}"),
    }
}

#[test]
fn host_projection_carries_runtime_component_properties_and_routes() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut ui_runtime = EditorUiHostRuntime::default();
    ui_runtime.load_builtin_host_templates().unwrap();

    let document_id = "inspector.surface_controls";
    let projection = ui_runtime.project_document(document_id).unwrap();
    let surface = ui_runtime.build_shared_surface(document_id).unwrap();
    let host_projection = ui_runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();

    let name_field = host_projection
        .node_by_control_id("NameField")
        .expect("inspector surface should project NameField");
    assert_eq!(name_field.component, "IconButton");
    assert_eq!(name_field.text.as_deref(), Some("NameField"));
    assert_eq!(
        name_field.properties.get("label"),
        Some(&SlintUiHostValue::String("NameField".to_string()))
    );
    assert!(name_field.routes.iter().any(|route| {
        route.binding_id == "InspectorView/NameField" && route.event_kind == UiEventKind::Change
    }));
}

#[test]
fn builtin_pane_body_documents_match_descriptor_ids_and_runtime_registration() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let runtime = editor_runtime();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let descriptors = manager.descriptors();

    let mut ui_runtime = EditorUiHostRuntime::default();
    ui_runtime.load_builtin_host_templates().unwrap();

    let cases = [
        (
            "editor.console",
            "pane.console.body",
            "ConsolePaneBody",
            "ConsolePaneBody/FocusConsole",
        ),
        (
            "editor.inspector",
            "pane.inspector.body",
            "InspectorPaneBody",
            "InspectorPaneBody/ApplyDraft",
        ),
        (
            "editor.hierarchy",
            "pane.hierarchy.body",
            "HierarchyPaneBody",
            "HierarchyPaneBody/SelectRoot",
        ),
        (
            "editor.animation_sequence",
            "pane.animation.sequence.body",
            "AnimationSequencePaneBody",
            "AnimationSequencePaneBody/ScrubTimeline",
        ),
        (
            "editor.animation_graph",
            "pane.animation.graph.body",
            "AnimationGraphPaneBody",
            "AnimationGraphPaneBody/AddNode",
        ),
        (
            "editor.runtime_diagnostics",
            "pane.runtime.diagnostics.body",
            "RuntimeDiagnosticsPaneBody",
            "RuntimeDiagnosticsPaneBody/FocusDiagnostics",
        ),
        (
            "editor.module_plugins",
            "pane.module_plugins.body",
            "ModulePluginsPaneBody",
            "ModulePluginsPaneBody/FocusModulePlugins",
        ),
        (
            "editor.build_export_desktop",
            "pane.build_export_desktop.body",
            "BuildExportPaneBody",
            "BuildExportPaneBody/FocusBuildExport",
        ),
    ];

    for (descriptor_id, document_id, component_id, binding_id) in cases {
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.descriptor_id == ViewDescriptorId::new(descriptor_id))
            .unwrap_or_else(|| panic!("missing builtin descriptor `{descriptor_id}`"));
        let pane_template = descriptor
            .pane_template
            .as_ref()
            .unwrap_or_else(|| panic!("descriptor `{descriptor_id}` is missing pane_template"));

        assert_eq!(
            pane_template.body.document_id, document_id,
            "descriptor `{descriptor_id}` must use the stable pane body document id"
        );

        let component = ui_runtime
            .component_descriptor(component_id)
            .unwrap_or_else(|| panic!("missing builtin component descriptor `{component_id}`"));
        assert_eq!(component.document_id, document_id);
        assert_eq!(component.binding_namespace, component_id);

        let projection = ui_runtime
            .project_document(document_id)
            .unwrap_or_else(|error| {
                panic!("failed to project builtin pane body document `{document_id}`: {error}")
            });
        assert_eq!(projection.document_id, document_id);
        assert_eq!(projection.root.component, "VerticalBox");
        assert!(
            projection
                .bindings
                .iter()
                .any(|binding| binding.binding_id == binding_id),
            "document `{document_id}` must expose binding `{binding_id}` through runtime projection"
        );
    }
}

#[test]
fn builtin_hybrid_pane_body_documents_declare_stable_native_slot_names() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());

    let cases = [
        (
            "hierarchy_body.ui.toml",
            "HierarchyPaneBody",
            "hierarchy_tree_slot",
        ),
        (
            "animation_sequence_body.ui.toml",
            "AnimationSequencePaneBody",
            "animation_timeline_slot",
        ),
        (
            "animation_graph_body.ui.toml",
            "AnimationGraphPaneBody",
            "animation_graph_canvas_slot",
        ),
        (
            "module_plugins_body.ui.toml",
            "ModulePluginsPaneBody",
            "module_plugin_list_slot",
        ),
        (
            "build_export_desktop_body.ui.toml",
            "BuildExportPaneBody",
            "build_export_targets_slot",
        ),
    ];

    for (file_name, component_id, slot_name) in cases {
        let source = fs::read_to_string(pane_body_path(file_name))
            .unwrap_or_else(|error| panic!("failed to read `{file_name}`: {error}"));
        let document = load_test_ui_asset(&source)
            .unwrap_or_else(|error| panic!("failed to parse `{file_name}`: {error}"));
        let component = document
            .components
            .get(component_id)
            .unwrap_or_else(|| panic!("missing component `{component_id}` in `{file_name}`"));

        assert!(
            component.slots.contains_key(slot_name),
            "component `{component_id}` in `{file_name}` must declare slot `{slot_name}`"
        );
        assert!(
            component
                .root
                .children
                .iter()
                .any(|child| child.node.slot_name.as_deref() == Some(slot_name)),
            "component `{component_id}` in `{file_name}` must expose slot placeholder `{slot_name}` in its root children"
        );
    }
}

#[test]
fn builtin_pane_body_bindings_stay_in_expected_command_namespaces() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    let cases = [
        (
            "pane.console.body",
            "ConsolePaneBody/FocusConsole",
            "DockCommand",
        ),
        (
            "pane.inspector.body",
            "InspectorPaneBody/ApplyDraft",
            "DraftCommand",
        ),
        (
            "pane.hierarchy.body",
            "HierarchyPaneBody/SelectRoot",
            "SelectionCommand",
        ),
        (
            "pane.animation.sequence.body",
            "AnimationSequencePaneBody/ScrubTimeline",
            "AnimationCommand",
        ),
        (
            "pane.animation.graph.body",
            "AnimationGraphPaneBody/AddNode",
            "AnimationCommand",
        ),
        (
            "pane.runtime.diagnostics.body",
            "RuntimeDiagnosticsPaneBody/FocusDiagnostics",
            "DockCommand",
        ),
        (
            "pane.module_plugins.body",
            "ModulePluginsPaneBody/FocusModulePlugins",
            "DockCommand",
        ),
        (
            "pane.build_export_desktop.body",
            "BuildExportPaneBody/FocusBuildExport",
            "DockCommand",
        ),
    ];

    for (document_id, binding_id, expected_namespace) in cases {
        let projection = runtime.project_document(document_id).unwrap();
        let binding = projection
            .bindings
            .iter()
            .find(|binding| binding.binding_id == binding_id)
            .unwrap_or_else(|| panic!("missing binding `{binding_id}` in `{document_id}`"));

        let actual_namespace = match &binding.binding.payload {
            EditorUiBindingPayload::DockCommand(_) => "DockCommand",
            EditorUiBindingPayload::DraftCommand(_) => "DraftCommand",
            EditorUiBindingPayload::SelectionCommand(_) => "SelectionCommand",
            EditorUiBindingPayload::AnimationCommand(_) => "AnimationCommand",
            other => panic!(
                "binding `{binding_id}` in `{document_id}` used unexpected payload namespace: {other:?}"
            ),
        };
        assert_eq!(actual_namespace, expected_namespace);
    }
}
