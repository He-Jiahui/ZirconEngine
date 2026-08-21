use std::fs;

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

use super::support::*;
use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::template_runtime::EditorUiHostRuntime;

#[test]
fn component_showcase_authored_props_are_declared_by_runtime_catalog() {
    let source = fs::read_to_string(editor_component_showcase_path()).unwrap();
    let document: toml::Value = toml::from_str(&source).unwrap();
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let mut mismatches = Vec::new();

    let root_node = document
        .get("root")
        .and_then(|root| root.get("node").and_then(toml::Value::as_str))
        .and_then(|node_id| document.get("nodes").and_then(|nodes| nodes.get(node_id)))
        .or_else(|| {
            document
                .get("root")
                .filter(|root| root.get("node").is_none())
        })
        .expect("component showcase asset should declare a root node");
    collect_showcase_prop_schema_mismatches(root_node, &registry, &mut mismatches);
    if let Some(nodes) = document.get("nodes").and_then(toml::Value::as_table) {
        for node in nodes.values() {
            collect_showcase_prop_schema_mismatches(node, &registry, &mut mismatches);
        }
    }
    mismatches.sort();
    mismatches.dedup();

    assert!(
        mismatches.is_empty(),
        "component_showcase.zui has props missing from the runtime component catalog:\n{}",
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

    let document_id = "res://ui/editor/component_showcase.zui";
    let projection = ui_runtime.project_document(document_id).unwrap();
    let mut surface = ui_runtime.build_shared_surface(document_id).unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();
    let host_projection = ui_runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();

    for control_id in [
        "ComponentShowcaseCommandToolbar",
        "ComponentShowcaseBottomLog",
        "ComponentShowcaseRuntimeBadge",
    ] {
        assert!(
            host_projection.node_by_control_id(control_id).is_some(),
            "component showcase should project imported .zui component control `{control_id}`"
        );
    }

    let material_button = host_projection
        .node_by_control_id("ButtonDemo")
        .expect("component showcase should project ButtonDemo through the Material meta component");
    assert_eq!(material_button.component, "Button");
    assert_host_string_property(material_button, "button_variant", "primary");
    assert_host_string_property(material_button, "surface_variant", "accent");
    assert_host_string_property(material_button, "validation_level", "normal");
    assert_material_button_layout_properties(material_button);
    assert_host_bool_property(material_button, "input_interactive", true);
    assert_host_bool_property(material_button, "input_clickable", true);
    assert_host_bool_property(material_button, "input_hoverable", true);
    assert_host_bool_property(material_button, "input_focusable", true);
    assert_host_bool_property(material_button, "disabled", false);
    assert_desired_size_covers_projected_text_with_horizontal_padding(
        &surface,
        material_button,
        12.0,
        12.0,
    );
    assert_frame_covers_text_with_horizontal_padding(material_button, 12.0, 12.0);
    assert!(
        material_button.frame.width >= 60.0,
        "ButtonDemo arranged width should include text intrinsic width plus Material horizontal padding, got {}",
        material_button.frame.width
    );
    assert!(
        material_button.frame.height >= 32.0,
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
        10.0,
        10.0,
    );
    assert_frame_covers_text_with_horizontal_padding(material_input, 10.0, 10.0);

    let number = host_projection
        .node_by_control_id("NumberFieldDemo")
        .expect("component showcase should project NumberFieldDemo through MaterialSpinBox");
    assert_eq!(number.component, "NumberField");
    assert_material_field_layout_properties(number);
    assert_host_bool_property(number, "input_interactive", true);
    assert_host_bool_property(number, "input_clickable", true);
    assert_host_bool_property(number, "input_hoverable", true);
    assert_host_bool_property(number, "input_focusable", true);
    assert_desired_size_covers_projected_text_with_horizontal_padding(&surface, number, 10.0, 10.0);
    assert_frame_covers_text_with_horizontal_padding(number, 10.0, 10.0);
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
        &surface, combo_box, 10.0, 10.0,
    );
    assert_frame_covers_text_with_horizontal_padding(combo_box, 10.0, 10.0);

    let material_list = host_projection
        .node_by_control_id("ListRowDemo")
        .expect("component showcase should project ListRowDemo through MaterialListItem");
    assert_eq!(material_list.component, "ListRow");
    assert_material_list_layout_properties(material_list);
    assert_desired_size_covers_projected_text_with_horizontal_padding(
        &surface,
        material_list,
        10.0,
        10.0,
    );
    assert_frame_covers_text_with_horizontal_padding(material_list, 10.0, 10.0);

    let table_row = host_projection
        .node_by_control_id("TableRowDemo")
        .expect("component showcase should project TableRowDemo through MaterialTableRow");
    assert_eq!(table_row.component, "TableRow");
    assert_material_list_layout_properties(table_row);
    assert_desired_size_covers_projected_text_with_horizontal_padding(
        &surface, table_row, 10.0, 10.0,
    );
    assert_frame_covers_text_with_horizontal_padding(table_row, 10.0, 10.0);

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
    assert_host_numeric_property(icon_button, "layout_min_width", 32.0);
    assert_host_numeric_property(icon_button, "layout_min_height", 32.0);
    assert!(
        icon_button.frame.width >= 32.0 && icon_button.frame.height >= 32.0,
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
        menu_frame.frame.height >= 32.0,
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
