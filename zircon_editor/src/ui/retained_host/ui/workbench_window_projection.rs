#[cfg(test)]
use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::load_preview_image;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;
#[cfg(test)]
use crate::ui::template_runtime::RetainedUiHostValue;
use crate::ui::template_runtime::{RetainedUiHostNodeModel, RetainedUiHostProjection};
use zircon_runtime::ui::style::resolve_button_style_from_values;
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiFrame};

use super::component_contract_metadata::tokens_for_component_role;
use super::pane_data_conversion::{
    projected_command_palette_options, projected_command_palette_structured_options,
    projected_notification_center_metadata_from_host, projected_notification_center_option_rows,
    structured_menu_items, structured_options_for_node,
};
use super::template_layout_context::apply_table_layout_context_variant;

#[path = "workbench_window_projection/defaults.rs"]
mod defaults;
#[path = "workbench_window_projection/host_value_toml.rs"]
mod host_value_toml;
#[path = "workbench_window_projection/mount.rs"]
mod mount;
#[path = "workbench_window_projection/node_index.rs"]
mod node_index;
#[path = "workbench_window_projection/notification_cache.rs"]
mod notification_cache;
#[path = "workbench_window_projection/previous_node_index.rs"]
mod previous_node_index;
#[path = "workbench_window_projection/properties.rs"]
mod properties;
#[path = "workbench_window_projection/selection_style.rs"]
mod selection_style;
#[path = "workbench_window_projection/status_right.rs"]
mod status_right;
#[path = "workbench_window_projection/typed_canvas.rs"]
mod typed_canvas;

use defaults::{
    default_border_width, default_corner_radius, default_text_tone,
    default_workbench_surface_variant, is_workbench_command_palette_mount,
    is_workbench_notification_center_mount, projected_workbench_text,
    projected_workbench_value_text, resolve_workbench_role,
};
use host_value_toml::{
    toml_values_from_host_properties, toml_values_from_host_properties_without_notifications,
};
use mount::project_node_into_mount;
use node_index::ProjectionNodeIndex;
use notification_cache::reusable_notification_rows;
use previous_node_index::{PreviousWorkbenchNodeIndex, model_with_projection_identity};
use properties::{
    bool_property, color_property, first_string_property, integer_property, normalized_percent,
    numeric_property, preferred_route_action_id, preferred_route_binding, shared_string_list,
    string_array_property, template_frame,
};
use selection_style::{
    clear_button_surface_style_values, is_cleared_inspector_property_row,
    normalize_workbench_selection_control_style_values,
};
use status_right::{
    inherited_status_right_color_property, inherited_status_right_numeric_property,
};
use typed_canvas::projected_typed_canvas_data;

const UI_HOST_WINDOW_ROOT_CONTROL_ID: &str = "UiHostWindowRoot";

pub(crate) fn to_host_contract_workbench_window_nodes(
    projection: Option<&RetainedUiHostProjection>,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    to_host_contract_workbench_window_nodes_with_previous(projection, None)
}

pub(crate) fn to_host_contract_workbench_window_nodes_with_previous(
    projection: Option<&RetainedUiHostProjection>,
    previous_nodes: Option<&ModelRc<host_contract::TemplatePaneNodeData>>,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    to_host_contract_workbench_window_nodes_with_previous_at_mount(projection, previous_nodes, None)
}

pub(crate) fn to_host_contract_workbench_window_nodes_with_previous_at_mount(
    projection: Option<&RetainedUiHostProjection>,
    previous_nodes: Option<&ModelRc<host_contract::TemplatePaneNodeData>>,
    mount_frame: Option<UiFrame>,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    let Some(projection) = projection else {
        return ModelRc::default();
    };

    let node_index = ProjectionNodeIndex::new(&projection.nodes);
    let previous_node_index = previous_nodes.and_then(|nodes| {
        PreviousWorkbenchNodeIndex::for_projection(nodes, projection.document_id.as_str())
    });
    let layout_context_width = projection_layout_context_width(projection);
    model_with_projection_identity(
        projection
            .nodes
            .iter()
            .filter(|node| node_index.render_visible(node))
            .filter_map(|node| {
                let previous = node
                    .control_id
                    .as_deref()
                    .and_then(|control_id| previous_node_index.as_ref()?.get(control_id));
                to_host_contract_workbench_window_node_with_previous(node, &node_index, previous)
            })
            .map(|node| apply_table_layout_context_variant(node, layout_context_width))
            .map(|node| project_node_into_mount(node, mount_frame))
            .collect(),
        projection.document_id.clone(),
    )
}

fn projection_layout_context_width(projection: &RetainedUiHostProjection) -> f32 {
    projection
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some(UI_HOST_WINDOW_ROOT_CONTROL_ID))
        .map(|node| node.frame.width.max(0.0))
        .unwrap_or(0.0)
}

fn to_host_contract_workbench_window_node(
    node: &RetainedUiHostNodeModel,
    node_index: &ProjectionNodeIndex<'_>,
) -> Option<host_contract::TemplatePaneNodeData> {
    to_host_contract_workbench_window_node_with_previous(node, node_index, None)
}

fn to_host_contract_workbench_window_node_with_previous(
    node: &RetainedUiHostNodeModel,
    node_index: &ProjectionNodeIndex<'_>,
    previous: Option<&host_contract::TemplatePaneNodeData>,
) -> Option<host_contract::TemplatePaneNodeData> {
    let control_id = node.control_id.clone()?;
    let mut component_role = node
        .component_role
        .clone()
        .filter(|role| !role.is_empty())
        .or_else(|| {
            let role = crate::ui::layouts::views::resolve_component_role(&node.component);
            (!role.is_empty()).then(|| role.to_string())
        })
        .unwrap_or_default();
    if component_role.is_empty()
        && is_workbench_command_palette_mount(node.component.as_str(), control_id.as_str())
    {
        component_role = "command-palette".to_string();
    }
    if component_role.is_empty()
        && is_workbench_notification_center_mount(node.component.as_str(), control_id.as_str())
    {
        component_role = "notification-center".to_string();
    }
    let notification_metadata =
        projected_notification_center_metadata_from_host(component_role.as_str(), &node.properties)
            .unwrap_or_default();
    let reused_notification_rows = reusable_notification_rows(previous, &notification_metadata);
    let mut button_style_values = if reused_notification_rows.is_some() {
        toml_values_from_host_properties_without_notifications(&node.properties)
    } else {
        toml_values_from_host_properties(&node.properties)
    };
    if is_cleared_inspector_property_row(control_id.as_str(), &node.properties) {
        clear_button_surface_style_values(&mut button_style_values);
    }
    normalize_workbench_selection_control_style_values(
        &mut button_style_values,
        node,
        component_role.as_str(),
    );
    let value_text =
        projected_workbench_value_text(node, component_role.as_str(), &button_style_values);
    let media_source = first_string_property(&node.properties, &["image", "source", "media"])
        .or_else(|| {
            matches!(node.component.as_str(), "Image" | "SvgIcon")
                .then(|| first_string_property(&node.properties, &["value"]))
                .flatten()
        })
        .unwrap_or_default();
    let icon_name = node.icon.clone().unwrap_or_default();
    let preview_image = load_preview_image(&media_source, &icon_name);
    let preview_size = preview_image.size();
    let (options_text, options, structured_options) = if let Some(reused) = reused_notification_rows
    {
        (
            reused.options_text,
            reused.options,
            reused.structured_options,
        )
    } else {
        let (notification_option_values, notification_structured_options) =
            projected_notification_center_option_rows(
                component_role.as_str(),
                &button_style_values,
            )
            .map(|(options, structured_options)| (Some(options), Some(structured_options)))
            .unwrap_or_default();
        let option_values =
            projected_command_palette_options(component_role.as_str(), &button_style_values)
                .or(notification_option_values)
                .unwrap_or_else(|| {
                    string_array_property(&node.properties, "options", &node.options)
                });
        let structured_options = projected_command_palette_structured_options(
            component_role.as_str(),
            &button_style_values,
        )
        .or(notification_structured_options)
        .unwrap_or_else(|| structured_options_for_node(&option_values, &button_style_values));
        (
            option_values.join(", ").into(),
            shared_string_list(option_values),
            model_rc(structured_options),
        )
    };
    let menu_item_values = string_array_property(&node.properties, "menu_items", &node.menu_items);
    let collection_item_values =
        string_array_property(&node.properties, "collection_items", &node.collection_items);
    let action_id = preferred_route_action_id(
        &node.routes,
        [UiEventKind::Click, UiEventKind::Toggle, UiEventKind::Change],
    )
    .unwrap_or_default();
    let edit_action_id =
        preferred_route_action_id(&node.routes, [UiEventKind::Change]).unwrap_or_default();
    let commit_action_id =
        preferred_route_action_id(&node.routes, [UiEventKind::Submit]).unwrap_or_default();
    let structured_menu_item_values = structured_menu_items(&menu_item_values);
    let surface_variant = first_string_property(&node.properties, &["surface_variant"])
        .or_else(|| default_workbench_surface_variant(&node.component, &component_role))
        .unwrap_or_default();
    let component_variant = first_string_property(
        &node.properties,
        &["component_variant", "variant", "mui_variant"],
    )
    .unwrap_or_default();
    let typed_canvas = projected_typed_canvas_data(component_role.as_str(), &button_style_values);
    let component_tokens = tokens_for_component_role(&node.component, component_role.as_str());
    let text_tone = first_string_property(&node.properties, &["text_tone"])
        .unwrap_or_else(|| default_text_tone(&node.component, &component_role, &surface_variant));
    let label_text = first_string_property(&node.properties, &["label_text"]).unwrap_or_default();
    let label_color = color_property(&node.properties, "label_color")
        .or_else(|| color_property(&node.properties, "icon_fill"))
        .or_else(|| color_property(&node.properties, "status_mark_color"))
        .or_else(|| {
            inherited_status_right_color_property(node, node_index, "status_right_label_color")
        })
        .unwrap_or_default();
    let icon_color = color_property(&node.properties, "icon_color")
        .or_else(|| color_property(&node.properties, "thumb_color"))
        .or_else(|| color_property(&node.properties, "icon_stroke"))
        .or_else(|| color_property(&node.properties, "arrow_color"))
        .unwrap_or_default();
    let icon_stroke_width = numeric_property(&node.properties, "icon_stroke_width")
        .or_else(|| numeric_property(&node.properties, "stroke_width"))
        .unwrap_or(0.0) as f32;
    let label_brightness = numeric_property(&node.properties, "label_brightness")
        .or_else(|| numeric_property(&node.properties, "visual_brightness"))
        .unwrap_or(1.0) as f32;
    let layout_offset_x =
        numeric_property(&node.properties, "layout_offset_x").unwrap_or(0.0) as f32;
    let layout_offset_y = numeric_property(&node.properties, "layout_offset_y")
        .or_else(|| {
            inherited_status_right_numeric_property(node, node_index, "status_right_offset_y")
        })
        .unwrap_or(0.0) as f32;
    let layout_icon_size = numeric_property(&node.properties, "layout_icon_size")
        .or_else(|| numeric_property(&node.properties, "thumb_size"))
        .unwrap_or(0.0) as f32;
    let layout_content_offset_x = numeric_property(&node.properties, "layout_content_offset_x")
        .or_else(|| numeric_property(&node.properties, "layout_gap"))
        .or_else(|| numeric_property(&node.properties, "layout_spacing"))
        .or_else(|| numeric_property(&node.properties, "track_offset_x"))
        .unwrap_or(0.0) as f32;
    let layout_content_offset_y = numeric_property(&node.properties, "layout_content_offset_y")
        .or_else(|| numeric_property(&node.properties, "icon_offset_y"))
        .or_else(|| numeric_property(&node.properties, "track_height"))
        .unwrap_or(0.0) as f32;
    let layout_first_cell_offset_x =
        numeric_property(&node.properties, "layout_first_cell_offset_x")
            .or_else(|| numeric_property(&node.properties, "track_width_delta"))
            .unwrap_or(0.0) as f32;
    let layout_second_cell_offset_x =
        numeric_property(&node.properties, "layout_second_cell_offset_x")
            .or_else(|| numeric_property(&node.properties, "range_min"))
            .unwrap_or(0.0) as f32;
    let layout_third_cell_offset_x =
        numeric_property(&node.properties, "layout_third_cell_offset_x")
            .or_else(|| numeric_property(&node.properties, "step_tick_count"))
            .unwrap_or(0.0) as f32;
    let layout_fourth_cell_offset_x =
        numeric_property(&node.properties, "layout_fourth_cell_offset_x").unwrap_or(0.0) as f32;
    let selected_segment_border_width =
        numeric_property(&node.properties, "selected_segment_border_width")
            .or_else(|| numeric_property(&node.properties, "selected_border_width"));
    let selected_segment_underline_height =
        numeric_property(&node.properties, "selected_segment_underline_height")
            .or_else(|| numeric_property(&node.properties, "selected_underline_height"));
    let selected_segment_underline_color =
        color_property(&node.properties, "selected_segment_underline_color")
            .or_else(|| color_property(&node.properties, "selected_underline_color"))
            .unwrap_or_default();

    Some(host_contract::TemplatePaneNodeData {
        node_id: node.node_id.clone().into(),
        parent_node_id: node_index
            .projected_parent_node_id(node)
            .unwrap_or_default()
            .into(),
        control_id: control_id.into(),
        role: resolve_workbench_role(node.component.as_str()).into(),
        text: projected_workbench_text(node, component_role.as_str()).into(),
        label_text: label_text.into(),
        label_color,
        label_brightness,
        layout_offset_x,
        layout_offset_y,
        layout_icon_size,
        layout_content_offset_x,
        layout_content_offset_y,
        layout_first_cell_offset_x,
        layout_second_cell_offset_x,
        layout_third_cell_offset_x,
        layout_fourth_cell_offset_x,
        component_role: component_role.clone().into(),
        component_category: component_tokens.category.into(),
        component_layout_role: component_tokens.layout_role.into(),
        component_variant: component_variant.into(),
        sample_grid: typed_canvas.sample_grid,
        timeline_strip: typed_canvas.timeline_strip,
        weight_heatmap: typed_canvas.weight_heatmap,
        value_text: value_text.into(),
        value_number: numeric_property(&node.properties, "value")
            .or_else(|| numeric_property(&node.properties, "dot_size"))
            .or_else(|| numeric_property(&node.properties, "status_mark_size"))
            .or_else(|| numeric_property(&node.properties, "arrow_size"))
            .or_else(|| numeric_property(&node.properties, "track_width"))
            .or_else(|| numeric_property(&node.properties, "icon_size"))
            .unwrap_or(0.0) as f32,
        value_percent: normalized_percent(&node.properties),
        value_color: color_property(&node.properties, "value_color")
            .or_else(|| color_property(&node.properties, "action_color"))
            .or_else(|| color_property(&node.properties, "arrow_color"))
            .or_else(|| color_property(&node.properties, "dot_color"))
            .or_else(|| color_property(&node.properties, "fourth_cell_text_color"))
            .or_else(|| color_property(&node.properties, "track_fill_color"))
            .or_else(|| color_property(&node.properties, "text_color"))
            .or_else(|| color_property(&node.properties, "foreground_color"))
            .or_else(|| color_property(&node.properties, "color"))
            .unwrap_or_default(),
        icon_color,
        icon_stroke_width,
        has_selected_segment_border_width: selected_segment_border_width.is_some(),
        selected_segment_border_width: selected_segment_border_width.unwrap_or(0.0) as f32,
        selected_segment_underline_height: selected_segment_underline_height.unwrap_or(0.0) as f32,
        selected_segment_underline_color,
        media_source: media_source.into(),
        icon_name: icon_name.into(),
        has_preview_image: preview_size.width > 0 && preview_size.height > 0,
        preview_image,
        validation_level: node.validation_level.clone().unwrap_or_default().into(),
        validation_message: node.validation_message.clone().unwrap_or_default().into(),
        popup_open: node.popup_open,
        has_popup_anchor: node.has_popup_anchor,
        popup_anchor_x: node.popup_anchor_x as f32,
        popup_anchor_y: node.popup_anchor_y as f32,
        selection_state: node.selection_state.clone().unwrap_or_default().into(),
        search_query: first_string_property(&node.properties, &["query"])
            .unwrap_or_default()
            .into(),
        selected: bool_property(&node.properties, "selected")
            || node.selection_state.as_deref() == Some("selected")
            || node.checked,
        tree_depth: integer_property(&node.properties, "tree_depth").unwrap_or(0),
        tree_indent_px: numeric_property(&node.properties, "tree_indent_px").unwrap_or_else(|| {
            f64::from(integer_property(&node.properties, "tree_depth").unwrap_or(0)) * 12.0
        }) as f32,
        options_text,
        options,
        structured_options,
        notification_generation: notification_metadata.generation,
        notification_unread_count: notification_metadata.unread_count,
        notification_overflow_count: notification_metadata.overflow_count,
        notification_selected_id: notification_metadata.selected_id.into(),
        notification_focused_index: notification_metadata
            .focused_index
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1),
        notification_visible_limit: notification_metadata.visible_limit,
        collection_items: shared_string_list(collection_item_values),
        menu_items: shared_string_list(menu_item_values),
        structured_menu_items: model_rc(structured_menu_item_values),
        checked: node.checked,
        expanded: node.expanded,
        focused: node.focused,
        hovered: node.hovered,
        pressed: node.pressed,
        dragging: node.dragging,
        drop_hovered: node.drop_hovered,
        active_drag_target: node.active_drag_target,
        disabled: node.disabled,
        state_layer_color: color_property(&node.properties, "thumb_halo_color").unwrap_or_default(),
        dispatch_kind: (!action_id.is_empty())
            .then_some("workbench")
            .unwrap_or("")
            .into(),
        action_id: action_id.into(),
        binding_id: preferred_route_binding(
            &node.routes,
            [UiEventKind::Click, UiEventKind::Toggle, UiEventKind::Change],
        )
        .unwrap_or_default()
        .into(),
        commit_action_id: commit_action_id.into(),
        edit_action_id: edit_action_id.into(),
        surface_variant: surface_variant.clone().into(),
        text_tone: text_tone.into(),
        button_variant: first_string_property(&node.properties, &["button_variant"])
            .unwrap_or_default()
            .into(),
        button_style: resolve_button_style_from_values(&button_style_values),
        font_size: numeric_property(&node.properties, "font_size").unwrap_or(0.0) as f32,
        font_weight: integer_property(&node.properties, "font_weight").unwrap_or(0),
        text_align: first_string_property(&node.properties, &["text_align", "textAlign"])
            .unwrap_or_default()
            .into(),
        overflow: first_string_property(&node.properties, &["overflow"])
            .unwrap_or_default()
            .into(),
        corner_radius: numeric_property(&node.properties, "corner_radius")
            .or_else(|| numeric_property(&node.properties, "radius"))
            .unwrap_or_else(|| default_corner_radius(&node.component, &component_role))
            as f32,
        border_width: numeric_property(&node.properties, "border_width")
            .or_else(|| default_border_width(&node.component, &component_role, &surface_variant))
            .unwrap_or(0.0) as f32,
        z_index: node.z_index,
        has_clip_frame: node.clip_frame.is_some(),
        clip_frame: node
            .clip_frame
            .map(template_frame)
            .unwrap_or_else(host_contract::TemplateNodeFrameData::default),
        frame: template_frame(node.frame),
        ..host_contract::TemplatePaneNodeData::default()
    })
}

#[cfg(test)]
#[path = "workbench_window_projection/tests.rs"]
mod tests;
