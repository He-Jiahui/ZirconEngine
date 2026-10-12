use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::RetainedUiHostNodeProjection;

use super::super::super::component_contract_metadata::descriptor_for_component;
use super::clip_frame::projected_clip_frame;
use super::collection_projection::projected_collection;
use super::drag_overlay::projected_drag_overlay_data;
use super::popup_actions::projected_popup_actions;
use super::sample_grid::projected_sample_grid;
use super::selection_options::projected_selection_options;
use super::template_node_data::{template_pane_node_data, ProjectedTemplateNodeParts};
use super::text_layout::projected_text_layout;
use super::timeline_strip::projected_timeline_strip;
use super::validation_state::projected_validation_state;
use super::value_media::projected_value_media;
use super::visual_state::projected_visual_state;
use super::visual_style::projected_visual_style;
use super::weight_heatmap::projected_weight_heatmap;
use super::world_space::projected_world_space;

pub(in super::super) fn host_template_node(
    node: RetainedUiHostNodeProjection,
) -> Option<host_contract::TemplatePaneNodeData> {
    let control_id = node.control_id?;
    let component = node.component;
    let component_descriptor = descriptor_for_component(&component);
    let component_role = component_descriptor
        .map(|descriptor| descriptor.role.clone())
        .filter(|role| !role.is_empty())
        .or_else(|| {
            let role = crate::ui::layouts::views::resolve_component_role(&component);
            (!role.is_empty()).then(|| role.to_string())
        })
        .unwrap_or_default();
    let drag_overlay = projected_drag_overlay_data(component_role.as_str(), &node.attributes);
    let value_media =
        projected_value_media(component_role.as_str(), &node.attributes, &drag_overlay);
    let validation_state = projected_validation_state(
        &node.attributes,
        component_role.as_str(),
        component_descriptor.is_some(),
    );
    let selection_options = projected_selection_options(component_role.as_str(), &node.attributes);
    let sample_grid = projected_sample_grid(component_role.as_str(), &node.attributes);
    let timeline_strip = projected_timeline_strip(component_role.as_str(), &node.attributes);
    let weight_heatmap = projected_weight_heatmap(component_role.as_str(), &node.attributes);
    let collection_projection = projected_collection(&component, &node.attributes, &node.bindings);
    let world_space = projected_world_space(&component, &node.attributes);
    let popup_actions = projected_popup_actions(
        &control_id,
        component_role.as_str(),
        &node.attributes,
        &node.bindings,
        component_descriptor,
        &drag_overlay,
        validation_state.disabled,
        node.frame.x,
        node.frame.y,
        node.frame.width,
        node.frame.height,
    );
    let text_layout = projected_text_layout(
        &control_id,
        component_role.as_str(),
        &node.attributes,
        !node.bindings.is_empty(),
        &drag_overlay,
    );
    let visual_style = projected_visual_style(
        &component,
        component_role.as_str(),
        &node.attributes,
        node.z_index,
        popup_actions.popup_open,
    );
    let visual_state = projected_visual_state(&node.attributes);
    let clip_frame = projected_clip_frame(node.clip_frame.as_ref());

    let authored_field_colors = [
        "background_color",
        "hover_background_color",
        "pressed_background_color",
        "focused_background_color",
        "focus_background_color",
        "border_color",
        "hover_border_color",
        "focus_border_color",
        "foreground_color",
        "text_color",
        "placeholder_color",
        "disabled_background_color",
        "disabled_border_color",
        "disabled_foreground_color",
        "error_border_color",
    ]
    .into_iter()
    .filter_map(|key| {
        let value = node
            .style_overrides
            .get(key)
            .or_else(|| node.attributes.get(key))?
            .clone();
        // Reuse the runtime style color parser; no separate CSS parser or token aliases.
        let values = std::collections::BTreeMap::from([("foreground_color".to_owned(), value)]);
        zircon_runtime::ui::style::resolve_button_style_from_values(&values)
            .element
            .foreground_color
            .map(|color| (key.to_owned(), color))
    })
    .collect();

    let authored_focus_border_width = node
        .style_overrides
        .get("focus_border_width")
        .or_else(|| node.attributes.get("focus_border_width"))
        .and_then(super::super::pane_value_conversion::value_as_f64)
        .map(|value| value as f32)
        .filter(|value| value.is_finite() && *value >= 0.0);
    let surface_node_id = node.surface_node_id;
    let property_field_id = node
        .attributes
        .get("inspector_property_field_id")
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let property_item_key = node
        .attributes
        .get("inspector_property_item_key")
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let mut projected = template_pane_node_data(ProjectedTemplateNodeParts {
        node_id: node.node_id,
        control_id,
        source_path: node.source_path.unwrap_or_default(),
        source_node_id: node.source_node_id.unwrap_or_default(),
        instance_path: serialize_instance_path(node.instance_path),
        parent_source_path: node.parent_source_path.unwrap_or_default(),
        parent_source_node_id: node.parent_source_node_id.unwrap_or_default(),
        parent_instance_path: serialize_instance_path(node.parent_instance_path),
        role: component,
        component_role,
        text_layout,
        value_media,
        validation_state,
        selection_options,
        sample_grid,
        timeline_strip,
        weight_heatmap,
        collection: collection_projection,
        world_space,
        popup_actions,
        drag_overlay,
        visual_state,
        visual_style,
        clip_frame,
    });
    projected.tree_owner_focused = surface_node_id.and_then(|_| {
        node.attributes
            .get("tree_owner_focused")
            .and_then(toml::Value::as_bool)
    });
    projected.authored_field_colors = authored_field_colors;
    projected.authored_focus_border_width = authored_focus_border_width;
    projected.surface_node_id = surface_node_id;
    projected.inspector_property_field_id = property_field_id.into();
    projected.inspector_property_item_key = property_item_key.into();
    Some(projected)
}

fn serialize_instance_path(
    instance_path: Option<Vec<zircon_runtime_interface::ui::v2::UiTemplateNodeInstancePathStep>>,
) -> String {
    instance_path
        .and_then(|steps| serde_json::to_string(&steps).ok())
        .unwrap_or_default()
}
