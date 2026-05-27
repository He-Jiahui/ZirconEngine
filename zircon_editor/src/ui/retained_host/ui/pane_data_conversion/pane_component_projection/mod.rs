use std::{collections::BTreeMap, sync::OnceLock};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::{Color, ModelRc};
use crate::ui::template_runtime::RetainedUiHostNodeProjection;
use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime::ui::style::resolve_button_style_from_values;
use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue};

mod collection_fields;
mod popup_frame;
pub(crate) mod preview_images;
mod progress_value;
mod showcase_actions;
mod surface_defaults;
mod transition_metadata;

use self::collection_fields::collection_fields_for_component;
use self::popup_frame::projected_popup_frame;
use self::preview_images::load_preview_image;
use self::progress_value::projected_value_percent;
use self::showcase_actions::{
    preferred_showcase_action_buttons, preferred_showcase_action_id,
    preferred_showcase_commit_action_id, preferred_showcase_drag_action_id,
    preferred_showcase_edit_action_id, preferred_showcase_pointer_drag_action_id,
};
use self::surface_defaults::{
    projected_border_width, projected_component_variant, projected_corner_radius,
    projected_elevation, projected_surface_variant, projected_text_tone,
    projected_validation_level, projected_z_index,
};
use self::transition_metadata::projected_transition_metadata;
use super::pane_menu_projection::structured_menu_items;
use super::pane_option_projection::structured_options_for_node;
use super::pane_value_conversion::{
    value_as_bool, value_as_color, value_as_f64, value_as_float_array, value_as_options,
    value_as_string,
};

fn to_host_contract_shared_string_list(
    items: Vec<String>,
) -> ModelRc<crate::ui::retained_host::primitives::SharedString> {
    model_rc(
        items
            .into_iter()
            .map(crate::ui::retained_host::primitives::SharedString::from)
            .collect(),
    )
}

pub(super) fn host_template_node(
    node: RetainedUiHostNodeProjection,
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
        .filter(|role| !role.is_empty())
        .or_else(|| {
            let role = crate::ui::layouts::views::resolve_component_role(&component);
            (!role.is_empty()).then(|| role.to_string())
        })
        .unwrap_or_default();
    let value_text = projected_badge_value_text(component_role.as_str(), &node.attributes)
        .or_else(|| node.attributes.get("value_text").and_then(value_as_string))
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
        .or_else(|| node.attributes.get("progress"))
        .and_then(value_as_f64)
        .unwrap_or(0.0);
    let value_percent = projected_value_percent(
        component_role.as_str(),
        value_number,
        node.attributes
            .get("value_percent")
            .or_else(|| node.attributes.get("progress_percent"))
            .and_then(value_as_f64),
        node.attributes.get("min").and_then(value_as_f64),
        node.attributes.get("max").and_then(value_as_f64),
    );
    let value_color = node
        .attributes
        .get("value")
        .and_then(value_as_color)
        .unwrap_or_else(|| Color::from_argb_u8(0, 0, 0, 0));
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
    let validation_level = projected_validation_level(
        &node.attributes,
        component_role.as_str(),
        disabled,
        component_descriptor.is_some(),
    );
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
        .or_else(|| node.attributes.get("open"))
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
    let frame = projected_popup_frame(
        &node.attributes,
        component_role.as_str(),
        popup_open,
        popup_anchor_x,
        popup_anchor_y,
        node.frame.x,
        node.frame.y,
        node.frame.width,
        node.frame.height,
    );
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
        .or_else(|| primary_change_binding_id(&node.bindings))
        .unwrap_or_default();
    let commit_action_id = component_descriptor
        .and_then(|_| preferred_showcase_commit_action_id(&control_id, &node.bindings))
        .or_else(|| primary_submit_binding_id(&node.bindings))
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
    let text_names: &[&str] = match component_role.as_str() {
        "card-header" => &["text", "label", "title", "subheader"],
        "snackbar" | "snackbar-content" => &["text", "label", "message"],
        _ => &["text", "label"],
    };
    let text = first_non_empty_string_attribute(&node.attributes, text_names)
        .or_else(|| {
            (!node.bindings.is_empty() || should_humanize_control_label(&control_id))
                .then(|| humanize_control_id(&control_id))
        })
        .unwrap_or_default();
    let component_variant = projected_component_variant(&node.attributes, component_role.as_str());
    let surface_variant = projected_surface_variant(
        &node.attributes,
        component_role.as_str(),
        &component_variant,
    );
    let text_tone = projected_text_tone(
        &node.attributes,
        component_role.as_str(),
        &component_variant,
    );
    let button_variant = node
        .attributes
        .get("button_variant")
        .and_then(value_as_string)
        .unwrap_or_default();
    let button_style = resolve_button_style_from_values(&node.attributes);
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
        .or_else(|| node.attributes.get("textAlign"))
        .and_then(value_as_string)
        .unwrap_or_else(|| {
            if component_role.as_str() == "divider" {
                "center".to_string()
            } else {
                "left".to_string()
            }
        });
    let overflow = node
        .attributes
        .get("overflow")
        .and_then(value_as_string)
        .unwrap_or_default();
    let corner_radius = projected_corner_radius(&node.attributes, component_role.as_str());
    let border_width = projected_border_width(
        &node.attributes,
        component_role.as_str(),
        &component_variant,
    );
    let elevation = projected_elevation(
        &node.attributes,
        component_role.as_str(),
        &component_variant,
    );
    let z_index = projected_z_index(&node.attributes, component_role.as_str(), node.z_index);
    let transition =
        projected_transition_metadata(&node.attributes, component_role.as_str(), popup_open);
    let state_layer_enabled = node
        .attributes
        .get("state_layer_enabled")
        .or_else(|| node.attributes.get("display_state_layer"))
        .and_then(value_as_bool)
        .unwrap_or(false);
    let state_layer_color = node
        .attributes
        .get("state_layer_color")
        .or_else(|| node.attributes.get("ripple_color"))
        .or_else(|| node.attributes.get("color"))
        .and_then(value_as_color)
        .unwrap_or_else(|| Color::from_argb_u8(0, 0, 0, 0));
    let ripple_enabled = node
        .attributes
        .get("ripple_enabled")
        .or_else(|| node.attributes.get("ripple"))
        .and_then(value_as_bool)
        .unwrap_or(false);
    let ripple_pressed_x = node
        .attributes
        .get("ripple_pressed_x")
        .or_else(|| node.attributes.get("pressed_x"))
        .and_then(value_as_f64)
        .unwrap_or(0.0) as f32;
    let ripple_pressed_y = node
        .attributes
        .get("ripple_pressed_y")
        .or_else(|| node.attributes.get("pressed_y"))
        .and_then(value_as_f64)
        .unwrap_or(0.0) as f32;
    let clip_ripple = node
        .attributes
        .get("clip_ripple")
        .and_then(value_as_bool)
        .unwrap_or(true);
    let enter_pressed = node
        .attributes
        .get("enter_pressed")
        .and_then(value_as_bool)
        .unwrap_or(false);

    Some(host_contract::TemplatePaneNodeData {
        node_id: node.node_id.into(),
        control_id: control_id.into(),
        role: component.into(),
        text: text.into(),
        component_role: component_role.into(),
        component_variant: component_variant.into(),
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
        enter_pressed,
        state_layer_enabled,
        state_layer_color,
        ripple_enabled,
        ripple_pressed_x,
        ripple_pressed_y,
        ripple_unclipped: !clip_ripple,
        transition_kind: transition.kind.into(),
        transition_in: transition.active,
        transition_entered: transition.entered,
        transition_progress: transition.progress,
        transition_duration_ms: transition.duration_ms,
        transition_easing: transition.easing.into(),
        transition_direction: transition.direction.into(),
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
        button_style,
        font_size,
        font_weight,
        text_align: text_align.into(),
        overflow: overflow.into(),
        corner_radius,
        border_width,
        elevation,
        z_index,
        has_clip_frame: node.clip_frame.is_some(),
        clip_frame: node
            .clip_frame
            .map(|clip| host_contract::TemplateNodeFrameData {
                x: clip.x,
                y: clip.y,
                width: clip.width,
                height: clip.height,
            })
            .unwrap_or_default(),
        frame,
    })
}

fn projected_badge_value_text(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> Option<String> {
    if component_role != "badge" {
        return None;
    }
    let variant = attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(value_as_string)
        .unwrap_or_else(|| "standard".to_string());
    if variant == "dot" {
        return Some(String::new());
    }
    let content = attributes
        .get("badgeContent")
        .or_else(|| attributes.get("badge_content"))?;
    let max = attributes.get("max").and_then(value_as_f64).unwrap_or(99.0);
    if badge_content_number(content).is_some_and(|value| value > max) {
        return Some(format!("{}+", max.round() as i64));
    }
    Some(UiValue::from_toml(content).display_text())
}

fn badge_content_number(value: &toml::Value) -> Option<f64> {
    value_as_f64(value).or_else(|| value_as_string(value)?.trim().parse::<f64>().ok())
}

fn value_as_i32(value: &toml::Value) -> Option<i32> {
    value
        .as_integer()
        .and_then(|value| i32::try_from(value).ok())
}

fn first_non_empty_string_attribute(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    names: &[&str],
) -> Option<String> {
    names
        .iter()
        .filter_map(|name| attributes.get(*name))
        .filter_map(value_as_string)
        .find(|value| !value.is_empty())
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
    bindings: &[crate::ui::template_runtime::RetainedUiHostBindingProjection],
) -> Option<String> {
    bindings
        .iter()
        .find(|binding| binding.event_kind == UiEventKind::Click)
        .map(|binding| binding.binding_id.clone())
}

fn primary_change_binding_id(
    bindings: &[crate::ui::template_runtime::RetainedUiHostBindingProjection],
) -> Option<String> {
    bindings
        .iter()
        .find(|binding| binding.event_kind == UiEventKind::Change)
        .map(|binding| binding.binding_id.clone())
}

fn primary_submit_binding_id(
    bindings: &[crate::ui::template_runtime::RetainedUiHostBindingProjection],
) -> Option<String> {
    bindings
        .iter()
        .find(|binding| binding.event_kind == UiEventKind::Submit)
        .map(|binding| binding.binding_id.clone())
}

fn runtime_component_registry() -> &'static UiComponentDescriptorRegistry {
    static UI_COMPONENT_REGISTRY: OnceLock<UiComponentDescriptorRegistry> = OnceLock::new();
    UI_COMPONENT_REGISTRY.get_or_init(|| {
        let mut registry = UiComponentDescriptorRegistry::editor_showcase();
        for descriptor in UiComponentDescriptorRegistry::material_editor_foundation()
            .descriptors()
            .cloned()
        {
            registry
                .register(descriptor)
                .expect("retained host component registry descriptors must validate");
        }
        registry
    })
}

#[cfg(test)]
mod tests;
