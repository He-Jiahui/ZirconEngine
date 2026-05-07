use std::fs;
use std::path::Path;

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

use crate::tests::support::load_test_ui_asset;
use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::host::module::{self, module_descriptor, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorManager;
use crate::ui::template_runtime::{EditorUiHostRuntime, SlintUiHostNodeModel, SlintUiHostValue};
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

fn runtime_ui_asset_path(file_name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("runtime")
        .join(file_name)
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
    let actual = host_numeric_property(node, property).unwrap_or_else(|| {
        panic!(
            "missing numeric property `{property}` on `{}`",
            node.node_id
        )
    });
    assert_eq!(actual, expected);
}

fn assert_host_bool_property(node: &SlintUiHostNodeModel, property: &str, expected: bool) {
    let actual = match node.properties.get(property) {
        Some(SlintUiHostValue::Bool(value)) => *value,
        _ => panic!("missing bool property `{property}` on `{}`", node.node_id),
    };
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

fn assert_runtime_material_button_metadata(
    source_file: &str,
    document_id: &str,
    control_ids: &[&str],
) {
    let source = fs::read_to_string(runtime_ui_asset_path(source_file)).unwrap();
    let mut ui_runtime = EditorUiHostRuntime::default();
    ui_runtime
        .register_document_source(document_id, &source)
        .unwrap();
    let mut surface = ui_runtime.build_shared_surface(document_id).unwrap();
    surface.compute_layout(UiSize::new(360.0, 240.0)).unwrap();

    for control_id in control_ids {
        let expected_control_id = *control_id;
        let node = surface
            .tree
            .nodes
            .values()
            .find(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    == Some(expected_control_id)
            })
            .unwrap_or_else(|| {
                panic!("runtime asset `{source_file}` should contain `{control_id}`")
            });
        let metadata = node
            .template_metadata
            .as_ref()
            .unwrap_or_else(|| panic!("`{control_id}` should carry template metadata"));
        assert_eq!(metadata.component, "Button");
        assert_eq!(node.layout_cache.desired_size.height, 40.0);
        assert!(
            node.layout_cache.desired_size.width >= 40.0,
            "`{control_id}` should be measured through Material button metrics"
        );
        assert_eq!(
            metadata.attributes.get("layout_padding_left"),
            Some(&toml::Value::Float(24.0))
        );
        assert_eq!(
            metadata.attributes.get("layout_min_height"),
            Some(&toml::Value::Float(40.0))
        );
    }
}

fn assert_frame_covers_text_with_horizontal_padding(
    node: &SlintUiHostNodeModel,
    padding_left: f64,
    padding_right: f64,
) {
    let text = node
        .text
        .as_deref()
        .or(node.value_text.as_deref())
        .unwrap_or_else(|| {
            panic!(
                "node `{}` should project visible text or value text",
                node.node_id
            )
        });
    let font_size = host_numeric_property(node, "font_size").unwrap_or(12.0);
    let expected_width =
        text.chars().count() as f64 * (font_size * 0.5).max(1.0) + padding_left + padding_right;
    assert!(
        f64::from(node.frame.width) >= expected_width,
        "node `{}` frame width {} should cover text `{}` plus Material padding {}",
        node.node_id,
        node.frame.width,
        text,
        expected_width
    );
}

fn projected_visible_text(node: &SlintUiHostNodeModel) -> &str {
    node.text
        .as_deref()
        .or(node.value_text.as_deref())
        .unwrap_or_else(|| {
            panic!(
                "node `{}` should project visible text or value text",
                node.node_id
            )
        })
}

fn text_width_with_padding(
    node: &SlintUiHostNodeModel,
    padding_left: f64,
    padding_right: f64,
) -> f64 {
    let text = projected_visible_text(node);
    let font_size = host_numeric_property(node, "font_size").unwrap_or(12.0);
    text.chars().count() as f64 * (font_size * 0.5).max(1.0) + padding_left + padding_right
}

fn surface_desired_width_for_control(surface: &UiSurface, control_id: &str) -> f32 {
    surface
        .tree
        .nodes
        .values()
        .find_map(|node| {
            let metadata = node.template_metadata.as_ref()?;
            (metadata.control_id.as_deref() == Some(control_id))
                .then_some(node.layout_cache.desired_size.width)
        })
        .unwrap_or_else(|| panic!("shared surface should contain control `{control_id}`"))
}

fn assert_desired_size_covers_projected_text_with_horizontal_padding(
    surface: &UiSurface,
    node: &SlintUiHostNodeModel,
    padding_left: f64,
    padding_right: f64,
) {
    let control_id = node
        .control_id
        .as_deref()
        .unwrap_or_else(|| panic!("node `{}` should have a control id", node.node_id));
    let expected_width = text_width_with_padding(node, padding_left, padding_right);
    let desired_width = surface_desired_width_for_control(surface, control_id);
    assert!(
        f64::from(desired_width) >= expected_width,
        "control `{control_id}` desired width {desired_width} should cover projected text `{}` plus Material padding {}",
        projected_visible_text(node),
        expected_width
    );
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
    let mut surface = ui_runtime.build_shared_surface(document_id).unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();
    let host_projection = ui_runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();

    let material_button = host_projection
        .node_by_control_id("ButtonDemo")
        .expect("component showcase should project ButtonDemo through the Material meta component");
    assert_eq!(material_button.component, "Button");
    assert_material_button_layout_properties(material_button);
    assert_host_bool_property(material_button, "input_interactive", true);
    assert_host_bool_property(material_button, "input_clickable", true);
    assert_host_bool_property(material_button, "input_hoverable", true);
    assert_host_bool_property(material_button, "input_focusable", true);
    assert_desired_size_covers_projected_text_with_horizontal_padding(
        &surface,
        material_button,
        24.0,
        24.0,
    );
    assert_frame_covers_text_with_horizontal_padding(material_button, 24.0, 24.0);
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
    assert_desired_size_covers_projected_text_with_horizontal_padding(
        &surface,
        material_input,
        16.0,
        16.0,
    );
    assert_frame_covers_text_with_horizontal_padding(material_input, 16.0, 16.0);

    let number = host_projection
        .node_by_control_id("NumberFieldDemo")
        .expect("component showcase should project NumberFieldDemo through MaterialSpinBox");
    assert_eq!(number.component, "NumberField");
    assert_material_field_layout_properties(number);
    assert_host_bool_property(number, "input_interactive", true);
    assert_host_bool_property(number, "input_clickable", true);
    assert_host_bool_property(number, "input_hoverable", true);
    assert_host_bool_property(number, "input_focusable", true);
    assert_desired_size_covers_projected_text_with_horizontal_padding(&surface, number, 16.0, 16.0);
    assert_frame_covers_text_with_horizontal_padding(number, 16.0, 16.0);
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
    assert_host_bool_property(combo_box, "input_interactive", true);
    assert_host_bool_property(combo_box, "input_clickable", true);
    assert_host_bool_property(combo_box, "input_hoverable", true);
    assert_host_bool_property(combo_box, "input_focusable", true);
    assert_desired_size_covers_projected_text_with_horizontal_padding(
        &surface, combo_box, 16.0, 16.0,
    );
    assert_frame_covers_text_with_horizontal_padding(combo_box, 16.0, 16.0);

    let material_list = host_projection
        .node_by_control_id("ListRowDemo")
        .expect("component showcase should project ListRowDemo through MaterialListItem");
    assert_eq!(material_list.component, "ListRow");
    assert_material_list_layout_properties(material_list);
    assert_desired_size_covers_projected_text_with_horizontal_padding(
        &surface,
        material_list,
        16.0,
        16.0,
    );
    assert_frame_covers_text_with_horizontal_padding(material_list, 16.0, 16.0);

    let table_row = host_projection
        .node_by_control_id("TableRowDemo")
        .expect("component showcase should project TableRowDemo through MaterialTableRow");
    assert_eq!(table_row.component, "TableRow");
    assert_material_list_layout_properties(table_row);
    assert_desired_size_covers_projected_text_with_horizontal_padding(
        &surface, table_row, 16.0, 16.0,
    );
    assert_frame_covers_text_with_horizontal_padding(table_row, 16.0, 16.0);

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

    let icon_button = host_projection
        .node_by_control_id("IconButtonDemo")
        .expect("component showcase should project IconButtonDemo through MaterialIconButton");
    assert_eq!(icon_button.component, "IconButton");
    assert_host_numeric_property(icon_button, "layout_min_width", 40.0);
    assert_host_numeric_property(icon_button, "layout_min_height", 40.0);
    assert!(
        icon_button.frame.width >= 40.0 && icon_button.frame.height >= 40.0,
        "IconButtonDemo should keep a square Material frame, got {}x{}",
        icon_button.frame.width,
        icon_button.frame.height
    );

    let menu_frame = host_projection
        .node_by_control_id("ContextActionMenuDemo")
        .expect(
            "component showcase should project ContextActionMenuDemo through MaterialMenuFrame",
        );
    assert_eq!(menu_frame.component, "ContextActionMenu");
    assert_material_list_layout_properties(menu_frame);
    assert!(menu_frame.has_popup_anchor);
    assert_eq!(menu_frame.popup_anchor_x, 156.0);
    assert_eq!(menu_frame.popup_anchor_y, 24.0);
    assert!(
        menu_frame.frame.height >= 40.0,
        "ContextActionMenuDemo should carry Material row height, got {}",
        menu_frame.frame.height
    );

    let virtual_list = host_projection
        .node_by_control_id("VirtualListDemo")
        .expect(
            "component showcase should project VirtualListDemo through MaterialStandardTableView",
        );
    assert_eq!(virtual_list.component, "VirtualList");
    assert_material_list_layout_properties(virtual_list);
    assert_host_bool_property(virtual_list, "input_interactive", true);
    assert_host_bool_property(virtual_list, "input_clickable", true);
    assert_host_bool_property(virtual_list, "input_hoverable", true);
    assert_host_bool_property(virtual_list, "input_focusable", true);

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
fn runtime_dialog_and_hud_buttons_participate_in_material_measurement() {
    assert_runtime_material_button_metadata(
        "runtime_hud.ui.toml",
        "test.runtime.sample_hud",
        &["UseAction", "InventoryAction"],
    );
    assert_runtime_material_button_metadata(
        "pause_dialog.ui.toml",
        "test.runtime.pause_dialog",
        &["ResumeAction", "QuitAction"],
    );
    assert_runtime_material_button_metadata(
        "settings_dialog.ui.toml",
        "test.runtime.settings_dialog",
        &["ApplySettings", "CloseSettings"],
    );
    assert_runtime_material_button_metadata(
        "inventory_dialog.ui.toml",
        "test.runtime.inventory_dialog",
        &["EquipItem", "CloseInventory"],
    );
    assert_runtime_material_button_metadata(
        "quest_log_dialog.ui.toml",
        "test.runtime.quest_log_dialog",
        &["TrackQuest", "CloseQuestLog"],
    );
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
    assert_eq!(name_field.component, "InputField");
    assert_eq!(name_field.text.as_deref(), None);
    assert_eq!(
        name_field.properties.get("placeholder"),
        Some(&SlintUiHostValue::String("Name".to_string()))
    );
    assert_host_numeric_property(name_field, "layout_min_height", 32.0);
    assert_host_bool_property(name_field, "input_focusable", true);
    assert!(name_field.routes.iter().any(|route| {
        route.binding_id == "InspectorView/NameField" && route.event_kind == UiEventKind::Change
    }));

    let position_x = host_projection
        .node_by_control_id("PositionXField")
        .expect("inspector surface should project PositionXField");
    assert_eq!(position_x.component, "NumberField");
    assert_eq!(position_x.value_text.as_deref(), Some("0"));
    assert_host_numeric_property(position_x, "layout_min_height", 32.0);
    assert_host_bool_property(position_x, "input_focusable", true);
    assert!(position_x.routes.iter().any(|route| {
        route.binding_id == "InspectorView/PositionXField"
            && route.event_kind == UiEventKind::Change
    }));

    let apply_button = host_projection
        .node_by_control_id("ApplyBatchButton")
        .expect("inspector surface should project ApplyBatchButton");
    assert_eq!(apply_button.component, "Button");
    assert_eq!(apply_button.text.as_deref(), Some("Apply"));
    assert_host_numeric_property(apply_button, "layout_min_height", 32.0);
    assert_host_bool_property(apply_button, "input_clickable", true);
    assert!(apply_button.routes.iter().any(|route| {
        route.binding_id == "InspectorView/ApplyBatchButton"
            && route.event_kind == UiEventKind::Click
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
fn runtime_diagnostics_body_exposes_ui_debug_reflector_section() {
    let source = fs::read_to_string(pane_body_path("runtime_diagnostics_body.ui.toml"))
        .expect("runtime diagnostics pane body asset should be readable");
    let document =
        load_test_ui_asset(&source).expect("runtime diagnostics pane body asset should parse");
    let component = document
        .components
        .get("RuntimeDiagnosticsPaneBody")
        .expect("runtime diagnostics component should exist");

    for control_id in [
        "UiDebugReflectorSummary",
        "UiDebugReflectorExportStatus",
        "UiDebugReflectorDetail",
        "UiDebugReflectorNodeList",
    ] {
        assert!(
            component
                .root
                .children
                .iter()
                .any(|child| child.node.control_id.as_deref() == Some(control_id)),
            "runtime diagnostics body should expose `{control_id}`"
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
