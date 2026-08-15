use std::path::Path;

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    FOUNDATION_MODULE_NAME, module_descriptor as foundation_module_descriptor,
};
use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::host::module::{self, module_descriptor};
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostNodeModel, RetainedUiHostValue,
};

pub(super) fn editor_runtime() -> CoreRuntime {
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

pub(super) fn pane_body_path(file_name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("editor")
        .join("host")
        .join(file_name)
}

pub(super) fn editor_component_showcase_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("editor")
        .join("component_showcase.zui")
}

pub(super) fn runtime_v2_fixture_path(file_name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor lives directly under workspace root")
        .join("zircon_runtime")
        .join("assets")
        .join("ui")
        .join("runtime")
        .join("fixtures")
        .join(file_name)
}

pub(super) fn collect_showcase_prop_schema_mismatches(
    node: &toml::Value,
    registry: &UiComponentDescriptorRegistry,
    mismatches: &mut Vec<String>,
) {
    if let Some(component_type) = node
        .get("type")
        .or_else(|| node.get("component"))
        .and_then(toml::Value::as_str)
    {
        if let Some(descriptor) = registry.descriptor(component_type) {
            if let Some(props) = node.get("props").and_then(toml::Value::as_table) {
                for prop in props.keys() {
                    if prop.starts_with("layout_") {
                        continue;
                    }
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

pub(super) fn host_numeric_property(node: &RetainedUiHostNodeModel, property: &str) -> Option<f64> {
    match node.properties.get(property) {
        Some(RetainedUiHostValue::Float(value)) => Some(*value),
        Some(RetainedUiHostValue::Integer(value)) => Some(*value as f64),
        _ => None,
    }
}

pub(super) fn assert_host_numeric_property(
    node: &RetainedUiHostNodeModel,
    property: &str,
    expected: f64,
) {
    let actual = host_numeric_property(node, property).unwrap_or_else(|| {
        panic!(
            "missing numeric property `{property}` on `{}`",
            node.node_id
        )
    });
    assert_eq!(actual, expected);
}

pub(super) fn assert_host_bool_property(
    node: &RetainedUiHostNodeModel,
    property: &str,
    expected: bool,
) {
    let actual = match node.properties.get(property) {
        Some(RetainedUiHostValue::Bool(value)) => *value,
        _ => panic!("missing bool property `{property}` on `{}`", node.node_id),
    };
    assert_eq!(actual, expected);
}

pub(super) fn assert_host_string_property(
    node: &RetainedUiHostNodeModel,
    property: &str,
    expected: &str,
) {
    let actual = match node.properties.get(property) {
        Some(RetainedUiHostValue::String(value)) => value.as_str(),
        _ => panic!("missing string property `{property}` on `{}`", node.node_id),
    };
    assert_eq!(actual, expected);
}

pub(super) fn assert_material_button_layout_properties(node: &RetainedUiHostNodeModel) {
    assert_host_numeric_property(node, "layout_padding_left", 12.0);
    assert_host_numeric_property(node, "layout_padding_right", 12.0);
    assert_host_numeric_property(node, "layout_padding_top", 6.0);
    assert_host_numeric_property(node, "layout_padding_bottom", 6.0);
    assert_host_numeric_property(node, "layout_spacing", 6.0);
    assert_host_numeric_property(node, "layout_min_width", 32.0);
    assert_host_numeric_property(node, "layout_min_height", 32.0);
    assert_host_numeric_property(node, "layout_icon_size", 16.0);
}

pub(super) fn assert_material_field_layout_properties(node: &RetainedUiHostNodeModel) {
    assert_host_numeric_property(node, "layout_padding_left", 10.0);
    assert_host_numeric_property(node, "layout_padding_right", 10.0);
    assert_host_numeric_property(node, "layout_padding_top", 3.0);
    assert_host_numeric_property(node, "layout_padding_bottom", 3.0);
    assert_host_numeric_property(node, "layout_min_height", 32.0);
}

pub(super) fn assert_material_list_layout_properties(node: &RetainedUiHostNodeModel) {
    assert_host_numeric_property(node, "layout_padding_left", 10.0);
    assert_host_numeric_property(node, "layout_padding_right", 10.0);
    assert_host_numeric_property(node, "layout_spacing", 6.0);
    assert_host_numeric_property(node, "layout_min_height", 32.0);
}

pub(super) fn assert_runtime_v2_button_metadata(
    source_file: &str,
    document_id: &str,
    control_ids: &[&str],
) {
    let mut ui_runtime = EditorUiHostRuntime::default();
    ui_runtime
        .register_document_file(document_id, runtime_v2_fixture_path(source_file))
        .unwrap();
    let mut surface = ui_runtime.build_shared_surface(document_id).unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();

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
        assert!(node.state_flags.clickable);
        assert!(node.state_flags.hoverable);
        assert!(node.state_flags.focusable);
        assert!(
            node.layout_cache.desired_size.width > 0.0,
            "`{control_id}` should have a projected v2 desired width"
        );
        assert!(
            node.layout_cache.desired_size.height > 0.0,
            "`{control_id}` should have a projected v2 desired height"
        );
        assert!(
            metadata.attributes.contains_key("text"),
            "`{control_id}` should keep authored text props in v2 metadata"
        );
    }
}

pub(super) fn assert_runtime_v2_click_route(
    surface: &UiSurface,
    control_id: &str,
    binding_id: &str,
    route: &str,
) {
    let node = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(control_id)
        })
        .unwrap_or_else(|| panic!("runtime v2 fixture should contain `{control_id}`"));
    let metadata = node
        .template_metadata
        .as_ref()
        .unwrap_or_else(|| panic!("`{control_id}` should carry template metadata"));
    assert!(
        metadata.bindings.iter().any(|binding| {
            binding.id == binding_id
                && binding.event == UiEventKind::Click
                && binding.route.as_deref() == Some(route)
        }),
        "`{control_id}` should expose click binding `{binding_id}` -> `{route}`"
    );
}

pub(super) fn assert_frame_covers_text_with_horizontal_padding(
    node: &RetainedUiHostNodeModel,
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

pub(super) fn projected_visible_text(node: &RetainedUiHostNodeModel) -> &str {
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

pub(super) fn text_width_with_padding(
    node: &RetainedUiHostNodeModel,
    padding_left: f64,
    padding_right: f64,
) -> f64 {
    let text = projected_visible_text(node);
    let font_size = host_numeric_property(node, "font_size").unwrap_or(12.0);
    text.chars().count() as f64 * (font_size * 0.5).max(1.0) + padding_left + padding_right
}

pub(super) fn surface_desired_width_for_control(surface: &UiSurface, control_id: &str) -> f32 {
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

pub(super) fn assert_desired_size_covers_projected_text_with_horizontal_padding(
    surface: &UiSurface,
    node: &RetainedUiHostNodeModel,
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
