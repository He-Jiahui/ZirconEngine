use std::sync::OnceLock;

use crate::ui::layouts::common::model_rc;
use crate::ui::slint_host as host_contract;
use crate::ui::template_runtime::SlintUiHostNodeProjection;
use slint::ModelRc;
use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue};

mod collection_fields;
pub(crate) mod preview_images;
mod showcase_actions;

use self::collection_fields::collection_fields_for_component;
use self::preview_images::load_preview_image;
use self::showcase_actions::{
    preferred_showcase_action_buttons, preferred_showcase_action_id,
    preferred_showcase_commit_action_id, preferred_showcase_drag_action_id,
    preferred_showcase_edit_action_id, preferred_showcase_pointer_drag_action_id,
};
use super::pane_menu_projection::structured_menu_items;
use super::pane_option_projection::structured_options_for_node;
use super::pane_value_conversion::{
    normalized_value_percent, value_as_bool, value_as_color, value_as_f64, value_as_float_array,
    value_as_options, value_as_string,
};

fn to_host_contract_shared_string_list(items: Vec<String>) -> ModelRc<slint::SharedString> {
    model_rc(items.into_iter().map(slint::SharedString::from).collect())
}

pub(super) fn host_template_node(
    node: SlintUiHostNodeProjection,
) -> Option<host_contract::TemplatePaneNodeData> {
    let control_id = node.control_id?;
    let component = node.component.clone();
    let component_descriptor = runtime_component_registry().descriptor(&component);
    let disabled = node
        .attributes
        .get("disabled")
        .and_then(value_as_bool)
        .unwrap_or(false)
        || node.attributes.get("enabled").and_then(value_as_bool) == Some(false);
    let component_role = component_descriptor
        .map(|descriptor| descriptor.role.clone())
        .unwrap_or_default();
    let value_text = node
        .attributes
        .get("value_text")
        .and_then(value_as_string)
        .or_else(|| {
            node.attributes
                .get("value")
                .or_else(|| node.attributes.get("items"))
                .or_else(|| node.attributes.get("entries"))
                .map(UiValue::from_toml)
                .map(|value| value.display_text())
        })
        .unwrap_or_default();
    let value_number = node
        .attributes
        .get("value")
        .and_then(value_as_f64)
        .unwrap_or(0.0);
    let value_percent = normalized_value_percent(
        value_number,
        node.attributes.get("min").and_then(value_as_f64),
        node.attributes.get("max").and_then(value_as_f64),
    );
    let value_color = node
        .attributes
        .get("value")
        .and_then(value_as_color)
        .unwrap_or_else(|| slint::Color::from_argb_u8(0, 0, 0, 0));
    let media_source = node
        .attributes
        .get("image")
        .or_else(|| node.attributes.get("source"))
        .or_else(|| node.attributes.get("media"))
        .or_else(|| {
            if matches!(component_role.as_str(), "image" | "svg-icon") {
                node.attributes.get("value")
            } else {
                None
            }
        })
        .and_then(value_as_string)
        .unwrap_or_default();
    let icon_name = node
        .attributes
        .get("icon")
        .or_else(|| {
            if component_role.as_str() == "icon" {
                node.attributes.get("value")
            } else {
                None
            }
        })
        .and_then(value_as_string)
        .unwrap_or_default();
    let preview_image = load_preview_image(&media_source, &icon_name);
    let preview_size = preview_image.size();
    let has_preview_image = preview_size.width > 0 && preview_size.height > 0;
    let vector_components = node
        .attributes
        .get("value")
        .and_then(value_as_float_array)
        .unwrap_or_default();
    let validation_level = node
        .attributes
        .get("validation_level")
        .and_then(value_as_string)
        .or_else(|| {
            component_descriptor.map(|_| if disabled { "disabled" } else { "normal" }.to_string())
        })
        .unwrap_or_default();
    let selection_state = node
        .attributes
        .get("selection_state")
        .and_then(value_as_string)
        .or_else(|| {
            node.attributes
                .get("multiple")
                .and_then(value_as_bool)
                .map(|multiple| if multiple { "multi" } else { "single" }.to_string())
        })
        .unwrap_or_default();
    let search_query = node
        .attributes
        .get("query")
        .and_then(value_as_string)
        .unwrap_or_default();
    let selected = node
        .attributes
        .get("selected")
        .and_then(value_as_bool)
        .unwrap_or(false);
    let tree_depth = node
        .attributes
        .get("tree_depth")
        .and_then(value_as_i32)
        .unwrap_or(0);
    let tree_indent_px = node
        .attributes
        .get("tree_indent_px")
        .and_then(value_as_f64)
        .unwrap_or_else(|| f64::from(tree_depth) * 12.0) as f32;
    let options = node
        .attributes
        .get("options")
        .and_then(value_as_options)
        .unwrap_or_default();
    let options_text = options.join(", ");
    let structured_options = structured_options_for_node(&options, &node.attributes);
    let mut collection_items = node
        .attributes
        .get("collection_items")
        .and_then(value_as_options)
        .unwrap_or_default();
    let collection_fields =
        collection_fields_for_component(&component, &node.attributes, &node.bindings);
    let virtualization_enabled = node
        .attributes
        .get("virtualization_enabled")
        .and_then(value_as_bool)
        .unwrap_or(component == "VirtualList");
    let virtualization_item_extent = node
        .attributes
        .get("item_extent")
        .and_then(value_as_f64)
        .unwrap_or(0.0) as f32;
    let virtualization_overscan = node
        .attributes
        .get("overscan")
        .and_then(value_as_i32)
        .unwrap_or(0);
    let virtualization_total_count = node
        .attributes
        .get("total_count")
        .and_then(value_as_i32)
        .unwrap_or(0);
    let virtualization_visible_start = node
        .attributes
        .get("viewport_start")
        .and_then(value_as_i32)
        .unwrap_or(0);
    let virtualization_visible_count = node
        .attributes
        .get("viewport_count")
        .and_then(value_as_i32)
        .unwrap_or(0);
    if virtualization_enabled {
        collection_items = visible_collection_items(
            collection_items,
            virtualization_visible_start,
            virtualization_visible_count,
            virtualization_overscan,
        );
    }
    let pagination_page_index = node
        .attributes
        .get("page_index")
        .and_then(value_as_i32)
        .unwrap_or(0);
    let pagination_page_size = node
        .attributes
        .get("page_size")
        .and_then(value_as_i32)
        .unwrap_or(0);
    let pagination_page_count = node
        .attributes
        .get("page_count")
        .and_then(value_as_i32)
        .unwrap_or(0);
    let pagination_total_count = node
        .attributes
        .get("total_count")
        .and_then(value_as_i32)
        .unwrap_or(0);
    let world_space_enabled = node
        .attributes
        .get("world_space_enabled")
        .and_then(value_as_bool)
        .unwrap_or(component == "WorldSpaceSurface");
    let world_position = node
        .attributes
        .get("world_position")
        .and_then(value_as_float_array)
        .unwrap_or_default();
    let world_rotation = node
        .attributes
        .get("world_rotation")
        .and_then(value_as_float_array)
        .unwrap_or_default();
    let world_scale = node
        .attributes
        .get("world_scale")
        .and_then(value_as_float_array)
        .unwrap_or_else(|| vec![1.0, 1.0, 1.0]);
    let world_size = node
        .attributes
        .get("world_size")
        .and_then(value_as_float_array)
        .unwrap_or_default();
    let world_pixels_per_meter = node
        .attributes
        .get("pixels_per_meter")
        .and_then(value_as_f64)
        .unwrap_or(0.0) as f32;
    let world_billboard = node
        .attributes
        .get("billboard")
        .and_then(value_as_bool)
        .unwrap_or(false);
    let world_depth_test = node
        .attributes
        .get("depth_test")
        .and_then(value_as_bool)
        .unwrap_or(false);
    let world_render_order = node
        .attributes
        .get("render_order")
        .and_then(value_as_i32)
        .unwrap_or(0);
    let world_camera_target = node
        .attributes
        .get("camera_target")
        .and_then(value_as_string)
        .unwrap_or_default();
    let menu_items = node
        .attributes
        .get("menu_items")
        .and_then(value_as_options)
        .unwrap_or_default();
    let structured_menu_items = structured_menu_items(&menu_items);
    let popup_open = node
        .attributes
        .get("popup_open")
        .and_then(value_as_bool)
        .unwrap_or(false);
    let popup_anchor_x = node
        .attributes
        .get("popup_anchor_x")
        .and_then(value_as_f64)
        .map(|value| value as f32);
    let popup_anchor_y = node
        .attributes
        .get("popup_anchor_y")
        .and_then(value_as_f64)
        .map(|value| value as f32);
    let has_popup_anchor = popup_anchor_x.is_some() && popup_anchor_y.is_some();
    let accepted_drag_payloads = component_descriptor
        .map(|descriptor| {
            descriptor
                .drop_policy
                .accepts
                .iter()
                .map(|kind| kind.as_str())
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();
    let action_id = component_descriptor
        .and_then(|_| preferred_showcase_action_id(&control_id, popup_open, &node.bindings))
        .unwrap_or_default();
    let binding_id = primary_click_binding_id(&node.bindings).unwrap_or_default();
    let drag_action_id = component_descriptor
        .and_then(|_| preferred_showcase_drag_action_id(&control_id, &node.bindings))
        .unwrap_or_default();
    let begin_drag_action_id = component_descriptor
        .and_then(|_| {
            preferred_showcase_pointer_drag_action_id(&control_id, "DragBegin", &node.bindings)
        })
        .unwrap_or_default();
    let end_drag_action_id = component_descriptor
        .and_then(|_| {
            preferred_showcase_pointer_drag_action_id(&control_id, "DragEnd", &node.bindings)
        })
        .unwrap_or_default();
    let edit_action_id = component_descriptor
        .and_then(|_| preferred_showcase_edit_action_id(&control_id, &node.bindings))
        .unwrap_or_default();
    let commit_action_id = component_descriptor
        .and_then(|_| preferred_showcase_commit_action_id(&control_id, &node.bindings))
        .unwrap_or_default();
    let actions = if component_descriptor.is_some() {
        preferred_showcase_action_buttons(&control_id, &node.bindings)
    } else {
        Vec::new()
    };
    let dispatch_kind = if !disabled && !action_id.is_empty() {
        "showcase"
    } else {
        ""
    };
    let text = node
        .attributes
        .get("text")
        .or_else(|| node.attributes.get("label"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            (!node.bindings.is_empty() || should_humanize_control_label(&control_id))
                .then(|| humanize_control_id(&control_id))
        })
        .unwrap_or_default();
    let surface_variant = node
        .attributes
        .get("surface_variant")
        .and_then(value_as_string)
        .unwrap_or_default();
    let text_tone = node
        .attributes
        .get("text_tone")
        .and_then(value_as_string)
        .unwrap_or_default();
    let button_variant = node
        .attributes
        .get("button_variant")
        .and_then(value_as_string)
        .unwrap_or_default();
    let font_size = node
        .attributes
        .get("font_size")
        .and_then(value_as_f64)
        .unwrap_or(0.0) as f32;
    let font_weight = node
        .attributes
        .get("font_weight")
        .and_then(value_as_i32)
        .unwrap_or(0);
    let text_align = node
        .attributes
        .get("text_align")
        .and_then(value_as_string)
        .unwrap_or_else(|| "left".to_string());
    let overflow = node
        .attributes
        .get("overflow")
        .and_then(value_as_string)
        .unwrap_or_default();
    let corner_radius = node
        .attributes
        .get("corner_radius")
        .or_else(|| node.attributes.get("radius"))
        .and_then(value_as_f64)
        .unwrap_or(0.0) as f32;
    let border_width = node
        .attributes
        .get("border_width")
        .and_then(value_as_f64)
        .unwrap_or(0.0) as f32;

    Some(host_contract::TemplatePaneNodeData {
        node_id: node.node_id.into(),
        control_id: control_id.into(),
        role: component.into(),
        text: text.into(),
        component_role: component_role.into(),
        value_text: value_text.into(),
        value_number: value_number as f32,
        value_percent,
        value_color,
        media_source: media_source.into(),
        icon_name: icon_name.into(),
        has_preview_image,
        preview_image,
        vector_components: model_rc(vector_components),
        validation_level: validation_level.into(),
        validation_message: node
            .attributes
            .get("validation_message")
            .and_then(value_as_string)
            .unwrap_or_default()
            .into(),
        popup_open,
        has_popup_anchor,
        popup_anchor_x: popup_anchor_x.unwrap_or(0.0),
        popup_anchor_y: popup_anchor_y.unwrap_or(0.0),
        selection_state: selection_state.into(),
        search_query: search_query.into(),
        selected,
        tree_depth,
        tree_indent_px,
        options_text: options_text.into(),
        options: to_host_contract_shared_string_list(options),
        structured_options: model_rc(structured_options),
        collection_items: to_host_contract_shared_string_list(collection_items),
        collection_fields: model_rc(collection_fields),
        virtualization_enabled,
        virtualization_item_extent,
        virtualization_overscan,
        virtualization_total_count,
        virtualization_visible_start,
        virtualization_visible_count,
        pagination_page_index,
        pagination_page_size,
        pagination_page_count,
        pagination_total_count,
        world_space_enabled,
        world_position_x: vec_component(&world_position, 0, 0.0),
        world_position_y: vec_component(&world_position, 1, 0.0),
        world_position_z: vec_component(&world_position, 2, 0.0),
        world_rotation_x: vec_component(&world_rotation, 0, 0.0),
        world_rotation_y: vec_component(&world_rotation, 1, 0.0),
        world_rotation_z: vec_component(&world_rotation, 2, 0.0),
        world_scale_x: vec_component(&world_scale, 0, 1.0),
        world_scale_y: vec_component(&world_scale, 1, 1.0),
        world_scale_z: vec_component(&world_scale, 2, 1.0),
        world_width: vec_component(&world_size, 0, 0.0),
        world_height: vec_component(&world_size, 1, 0.0),
        world_pixels_per_meter,
        world_billboard,
        world_depth_test,
        world_render_order,
        world_camera_target: world_camera_target.into(),
        menu_items: to_host_contract_shared_string_list(menu_items),
        structured_menu_items: model_rc(structured_menu_items),
        actions: model_rc(actions),
        accepted_drag_payloads: accepted_drag_payloads.into(),
        drop_source_summary: node
            .attributes
            .get("drop_source_summary")
            .and_then(value_as_string)
            .unwrap_or_default()
            .into(),
        checked: node
            .attributes
            .get("checked")
            .or_else(|| node.attributes.get("value"))
            .and_then(value_as_bool)
            .unwrap_or(false),
        expanded: node
            .attributes
            .get("expanded")
            .and_then(value_as_bool)
            .unwrap_or(false),
        focused: node
            .attributes
            .get("focused")
            .and_then(value_as_bool)
            .unwrap_or(false),
        hovered: node
            .attributes
            .get("hovered")
            .and_then(value_as_bool)
            .unwrap_or(false),
        pressed: node
            .attributes
            .get("pressed")
            .and_then(value_as_bool)
            .unwrap_or(false),
        dragging: node
            .attributes
            .get("dragging")
            .and_then(value_as_bool)
            .unwrap_or(false),
        drop_hovered: node
            .attributes
            .get("drop_hovered")
            .and_then(value_as_bool)
            .unwrap_or(false),
        active_drag_target: node
            .attributes
            .get("active_drag_target")
            .and_then(value_as_bool)
            .unwrap_or(false),
        disabled,
        dispatch_kind: dispatch_kind.into(),
        action_id: action_id.into(),
        binding_id: binding_id.into(),
        begin_drag_action_id: begin_drag_action_id.into(),
        drag_action_id: drag_action_id.into(),
        end_drag_action_id: end_drag_action_id.into(),
        commit_action_id: commit_action_id.into(),
        edit_action_id: edit_action_id.into(),
        surface_variant: surface_variant.into(),
        text_tone: text_tone.into(),
        button_variant: button_variant.into(),
        font_size,
        font_weight,
        text_align: text_align.into(),
        overflow: overflow.into(),
        corner_radius,
        border_width,
        frame: host_contract::TemplateNodeFrameData {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        },
    })
}

fn value_as_i32(value: &toml::Value) -> Option<i32> {
    value
        .as_integer()
        .and_then(|value| i32::try_from(value).ok())
}

fn vec_component(values: &[f32], index: usize, default: f32) -> f32 {
    values.get(index).copied().unwrap_or(default)
}

fn visible_collection_items(
    items: Vec<String>,
    visible_start: i32,
    visible_count: i32,
    overscan: i32,
) -> Vec<String> {
    if visible_count <= 0 {
        return Vec::new();
    }

    let visible_start = visible_start.max(0);
    let overscan = overscan.max(0);
    let start = visible_start.saturating_sub(overscan).max(0) as usize;
    let end = visible_start
        .saturating_add(visible_count)
        .saturating_add(overscan)
        .max(0) as usize;

    items
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| (index >= start && index < end).then_some(item))
        .collect()
}

fn humanize_control_id(control_id: &str) -> String {
    let mut text = String::new();
    for (index, character) in control_id.chars().enumerate() {
        if index > 0 && character.is_uppercase() {
            text.push(' ');
        }
        text.push(character);
    }
    text
}

fn should_humanize_control_label(control_id: &str) -> bool {
    control_id.starts_with("Apply")
        || control_id.starts_with("Delete")
        || control_id.ends_with("Button")
        || control_id.ends_with("Action")
}

fn primary_click_binding_id(
    bindings: &[crate::ui::template_runtime::SlintUiHostBindingProjection],
) -> Option<String> {
    bindings
        .iter()
        .find(|binding| binding.event_kind == UiEventKind::Click)
        .map(|binding| binding.binding_id.clone())
}

fn runtime_component_registry() -> &'static UiComponentDescriptorRegistry {
    static UI_COMPONENT_REGISTRY: OnceLock<UiComponentDescriptorRegistry> = OnceLock::new();
    UI_COMPONENT_REGISTRY.get_or_init(UiComponentDescriptorRegistry::editor_showcase)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::ui::template_runtime::SlintUiHostBindingProjection;
    use slint::Model;
    use toml::Value;
    use zircon_runtime_interface::ui::layout::UiFrame;

    #[test]
    fn runtime_component_projection_preserves_virtualization_and_pagination_metadata() {
        let virtual_list = host_template_node(projected_node(
            "VirtualList",
            [
                ("item_extent", Value::Float(32.0)),
                ("overscan", Value::Integer(4)),
                ("total_count", Value::Integer(1000)),
                ("viewport_start", Value::Integer(120)),
                ("viewport_count", Value::Integer(40)),
            ],
        ))
        .expect("VirtualList should project into the host contract");

        assert!(virtual_list.virtualization_enabled);
        assert_eq!(virtual_list.virtualization_item_extent, 32.0);
        assert_eq!(virtual_list.virtualization_overscan, 4);
        assert_eq!(virtual_list.virtualization_total_count, 1000);
        assert_eq!(virtual_list.virtualization_visible_start, 120);
        assert_eq!(virtual_list.virtualization_visible_count, 40);

        let paged_list = host_template_node(projected_node(
            "PagedList",
            [
                ("total_count", Value::Integer(2500)),
                ("page_index", Value::Integer(3)),
                ("page_size", Value::Integer(100)),
                ("page_count", Value::Integer(25)),
            ],
        ))
        .expect("PagedList should project into the host contract");

        assert!(!paged_list.virtualization_enabled);
        assert_eq!(paged_list.pagination_total_count, 2500);
        assert_eq!(paged_list.pagination_page_index, 3);
        assert_eq!(paged_list.pagination_page_size, 100);
        assert_eq!(paged_list.pagination_page_count, 25);
    }

    #[test]
    fn runtime_component_projection_slices_virtualized_visible_collection_items() {
        let virtual_list = host_template_node(projected_node(
            "VirtualList",
            [
                (
                    "collection_items",
                    string_array((0..20).map(|index| format!("Item {index}"))),
                ),
                ("viewport_start", Value::Integer(10)),
                ("viewport_count", Value::Integer(5)),
                ("overscan", Value::Integer(2)),
            ],
        ))
        .expect("VirtualList should project a visible collection window");

        assert_eq!(virtual_list.collection_items.row_count(), 9);
        assert_eq!(
            virtual_list.collection_items.row_data(0).as_deref(),
            Some("Item 8")
        );
        assert_eq!(
            virtual_list.collection_items.row_data(8).as_deref(),
            Some("Item 16")
        );
    }

    #[test]
    fn runtime_component_projection_clamps_virtualized_collection_window_edges() {
        let negative_start = host_template_node(projected_node(
            "VirtualList",
            [
                (
                    "collection_items",
                    string_array((0..5).map(|index| format!("Item {index}"))),
                ),
                ("viewport_start", Value::Integer(-10)),
                ("viewport_count", Value::Integer(2)),
                ("overscan", Value::Integer(1)),
            ],
        ))
        .expect("VirtualList should project a negative start deterministically");

        assert_eq!(negative_start.collection_items.row_count(), 3);
        assert_eq!(
            negative_start.collection_items.row_data(0).as_deref(),
            Some("Item 0")
        );
        assert_eq!(
            negative_start.collection_items.row_data(2).as_deref(),
            Some("Item 2")
        );

        let zero_count = host_template_node(projected_node(
            "VirtualList",
            [
                (
                    "collection_items",
                    string_array((0..5).map(|index| format!("Item {index}"))),
                ),
                ("viewport_start", Value::Integer(1)),
                ("viewport_count", Value::Integer(0)),
                ("overscan", Value::Integer(10)),
            ],
        ))
        .expect("VirtualList should project a zero visible count deterministically");

        assert_eq!(zero_count.collection_items.row_count(), 0);

        let oversized_overscan = host_template_node(projected_node(
            "VirtualList",
            [
                (
                    "collection_items",
                    string_array((0..4).map(|index| format!("Item {index}"))),
                ),
                ("viewport_start", Value::Integer(2)),
                ("viewport_count", Value::Integer(1)),
                ("overscan", Value::Integer(50)),
            ],
        ))
        .expect("VirtualList should project oversized overscan deterministically");

        assert_eq!(oversized_overscan.collection_items.row_count(), 4);
    }

    #[test]
    fn runtime_component_projection_preserves_primary_click_binding_id() {
        let mut button = projected_node("Button", [("text", Value::String("Apply".to_owned()))]);
        button.bindings.push(SlintUiHostBindingProjection {
            binding_id: "InspectorPaneBody/ApplyDraft".to_owned(),
            event_kind: UiEventKind::Click,
            route_id: None,
        });

        let projected = host_template_node(button)
            .expect("button with a primary click binding should project into the host contract");

        assert_eq!(
            projected.binding_id.as_str(),
            "InspectorPaneBody/ApplyDraft"
        );
        assert_eq!(projected.action_id.as_str(), "");
    }

    #[test]
    fn runtime_component_projection_preserves_material_visual_metadata() {
        let button = host_template_node(projected_node(
            "Button",
            [
                ("surface_variant", Value::String("accent".to_owned())),
                ("text_tone", Value::String("muted".to_owned())),
                ("button_variant", Value::String("primary".to_owned())),
                ("font_size", Value::Float(13.0)),
                ("font_weight", Value::Integer(600)),
                ("text_align", Value::String("center".to_owned())),
                ("overflow", Value::String("clip".to_owned())),
                ("corner_radius", Value::Float(5.0)),
                ("border_width", Value::Float(1.0)),
            ],
        ))
        .expect("material button metadata should project into the host contract");

        assert_eq!(button.surface_variant.as_str(), "accent");
        assert_eq!(button.text_tone.as_str(), "muted");
        assert_eq!(button.button_variant.as_str(), "primary");
        assert_eq!(button.font_size, 13.0);
        assert_eq!(button.font_weight, 600);
        assert_eq!(button.text_align.as_str(), "center");
        assert_eq!(button.overflow.as_str(), "clip");
        assert_eq!(button.corner_radius, 5.0);
        assert_eq!(button.border_width, 1.0);
    }

    #[test]
    fn runtime_component_projection_preserves_world_space_metadata() {
        let world_surface = host_template_node(projected_node(
            "WorldSpaceSurface",
            [
                ("world_position", float_array([1.0, 2.0, 3.0])),
                ("world_rotation", float_array([10.0, 20.0, 30.0])),
                ("world_scale", float_array([2.0, 2.5, 3.0])),
                ("world_size", float_array([4.0, 2.0, 0.0])),
                ("pixels_per_meter", Value::Float(128.0)),
                ("billboard", Value::Boolean(true)),
                ("depth_test", Value::Boolean(true)),
                ("render_order", Value::Integer(7)),
                ("camera_target", Value::String("viewport-main".to_owned())),
            ],
        ))
        .expect("WorldSpaceSurface should project into the host contract");

        assert!(world_surface.world_space_enabled);
        assert_eq!(world_surface.world_position_x, 1.0);
        assert_eq!(world_surface.world_position_y, 2.0);
        assert_eq!(world_surface.world_position_z, 3.0);
        assert_eq!(world_surface.world_rotation_x, 10.0);
        assert_eq!(world_surface.world_rotation_y, 20.0);
        assert_eq!(world_surface.world_rotation_z, 30.0);
        assert_eq!(world_surface.world_scale_x, 2.0);
        assert_eq!(world_surface.world_scale_y, 2.5);
        assert_eq!(world_surface.world_scale_z, 3.0);
        assert_eq!(world_surface.world_width, 4.0);
        assert_eq!(world_surface.world_height, 2.0);
        assert_eq!(world_surface.world_pixels_per_meter, 128.0);
        assert!(world_surface.world_billboard);
        assert!(world_surface.world_depth_test);
        assert_eq!(world_surface.world_render_order, 7);
        assert_eq!(world_surface.world_camera_target.as_str(), "viewport-main");
    }

    fn projected_node(
        component: &str,
        attributes: impl IntoIterator<Item = (&'static str, Value)>,
    ) -> SlintUiHostNodeProjection {
        SlintUiHostNodeProjection {
            node_id: format!("{component}Node"),
            parent_id: None,
            component: component.to_owned(),
            control_id: Some(format!("{component}Control")),
            frame: UiFrame::new(0.0, 0.0, 320.0, 240.0),
            clip_frame: None,
            z_index: 0,
            attributes: attributes
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value))
                .collect::<BTreeMap<_, _>>(),
            style_tokens: BTreeMap::new(),
            bindings: Vec::new(),
        }
    }

    fn float_array(values: [f64; 3]) -> Value {
        Value::Array(values.into_iter().map(Value::Float).collect())
    }

    fn string_array(values: impl Iterator<Item = String>) -> Value {
        Value::Array(values.map(Value::String).collect())
    }
}
