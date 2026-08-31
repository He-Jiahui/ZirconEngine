use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;
#[cfg(test)]
use crate::ui::template_runtime::RetainedUiHostValue;
use crate::ui::template_runtime::{RetainedUiHostNodeModel, RetainedUiHostProjection};
use zircon_runtime::ui::style::resolve_button_style_from_values;
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiFrame};

use super::component_contract_metadata::tokens_for_component_role;
use super::pane_data_conversion::{
    projected_command_palette_option_rows, projected_notification_center_metadata_from_host,
    projected_notification_center_option_rows, projected_settings_window_data,
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
    is_workbench_notification_center_mount, is_workbench_settings_window_mount,
    projected_workbench_text, projected_workbench_value_text, resolve_workbench_role,
};
use host_value_toml::{
    toml_values_from_host_properties, toml_values_from_host_properties_without_notifications,
};
use mount::{
    project_frame_into_physical_mount, project_node_into_physical_mount, scale_visual_metric,
};
use node_index::ProjectionNodeIndex;
use notification_cache::reusable_notification_rows;
use previous_node_index::{model_with_projection_identity, PreviousWorkbenchNodeIndex};
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

pub(crate) struct WorkbenchWindowNodePatch {
    pub(crate) nodes: ModelRc<host_contract::TemplatePaneNodeData>,
    pub(crate) changed_rows: Vec<usize>,
}

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
    to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
        projection,
        previous_nodes,
        mount_frame,
        1.0,
    )
}

pub(crate) fn to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
    projection: Option<&RetainedUiHostProjection>,
    previous_nodes: Option<&ModelRc<host_contract::TemplatePaneNodeData>>,
    mount_frame: Option<UiFrame>,
    scale_factor: f32,
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
                to_host_contract_workbench_window_node_with_previous(
                    node,
                    &node_index,
                    previous,
                    scale_factor,
                )
            })
            .map(|node| apply_table_layout_context_variant(node, layout_context_width))
            .map(|node| project_node_into_physical_mount(node, mount_frame, scale_factor))
            .collect(),
        projection.document_id.clone(),
    )
}

pub(crate) fn patch_host_contract_workbench_window_nodes_at_mount(
    document_id: &str,
    projection_nodes: &[RetainedUiHostNodeModel],
    previous_nodes: &ModelRc<host_contract::TemplatePaneNodeData>,
    mount_frame: Option<UiFrame>,
) -> Option<ModelRc<host_contract::TemplatePaneNodeData>> {
    patch_host_contract_workbench_window_nodes_at_mount_and_scale(
        document_id,
        projection_nodes,
        previous_nodes,
        mount_frame,
        1.0,
    )
}

pub(crate) fn patch_host_contract_workbench_window_nodes_at_mount_and_scale(
    document_id: &str,
    projection_nodes: &[RetainedUiHostNodeModel],
    previous_nodes: &ModelRc<host_contract::TemplatePaneNodeData>,
    mount_frame: Option<UiFrame>,
    scale_factor: f32,
) -> Option<ModelRc<host_contract::TemplatePaneNodeData>> {
    build_host_contract_workbench_window_node_patch_at_mount_and_scale(
        document_id,
        projection_nodes,
        previous_nodes,
        mount_frame,
        scale_factor,
    )
    .map(|patch| patch.nodes)
}

pub(crate) fn build_host_contract_workbench_window_node_patch_at_mount_and_scale(
    document_id: &str,
    projection_nodes: &[RetainedUiHostNodeModel],
    previous_nodes: &ModelRc<host_contract::TemplatePaneNodeData>,
    mount_frame: Option<UiFrame>,
    scale_factor: f32,
) -> Option<WorkbenchWindowNodePatch> {
    if projection_nodes.is_empty() {
        return Some(WorkbenchWindowNodePatch {
            nodes: previous_nodes.clone(),
            changed_rows: Vec::new(),
        });
    }
    let previous_node_index =
        PreviousWorkbenchNodeIndex::for_projection(previous_nodes, document_id)?;
    let node_index = ProjectionNodeIndex::new(projection_nodes);
    let mut row_patches = BTreeMap::new();
    for node in projection_nodes {
        if !node_index.render_visible(node) {
            return None;
        }
        let control_id = node.control_id.as_deref()?;
        let row = previous_node_index.row(control_id)?;
        let previous = previous_node_index.get(control_id)?;
        let mut projected = to_host_contract_workbench_window_node_with_previous(
            node,
            &node_index,
            Some(previous),
            scale_factor,
        )?;
        // The sparse workset does not carry unchanged ancestors. Preserve the already validated
        // host parent identity while replacing only the row's semantic and geometry payload.
        projected.parent_node_id = previous.parent_node_id.clone();
        row_patches.insert(
            row,
            project_node_into_physical_mount(projected, mount_frame, scale_factor),
        );
    }
    let changed_rows = row_patches.keys().copied().collect();
    Some(WorkbenchWindowNodePatch {
        nodes: previous_nodes.with_row_patches(row_patches),
        changed_rows,
    })
}

pub(crate) fn build_host_contract_workbench_window_geometry_patch_at_mount_and_scale(
    projection: &RetainedUiHostProjection,
    projection_node_indices: &[usize],
    previous_nodes: &ModelRc<host_contract::TemplatePaneNodeData>,
    mount_frame: Option<UiFrame>,
    scale_factor: f32,
) -> Option<WorkbenchWindowNodePatch> {
    if projection_node_indices.is_empty() {
        return Some(WorkbenchWindowNodePatch {
            nodes: previous_nodes.clone(),
            changed_rows: Vec::new(),
        });
    }
    let previous_node_index = PreviousWorkbenchNodeIndex::for_projection(
        previous_nodes,
        projection.document_id.as_str(),
    )?;
    let mut row_patches = BTreeMap::new();
    for projection_index in projection_node_indices {
        let node = projection.nodes.get(*projection_index)?;
        let control_id = node.control_id.as_deref()?;
        let row = previous_node_index.row(control_id)?;
        let previous = previous_node_index.get(control_id)?;
        if previous.node_id.as_str() != node.node_id.as_str()
            || previous.control_id.as_str() != control_id
            || previous.parent_node_id.as_str() != node.parent_id.as_deref().unwrap_or_default()
        {
            return None;
        }

        let mut projected = previous.clone();
        projected.frame = project_frame_into_physical_mount(
            template_frame(node.frame),
            mount_frame,
            scale_factor,
        );
        projected.has_clip_frame = node.clip_frame.is_some();
        projected.clip_frame = node
            .clip_frame
            .map(template_frame)
            .map(|frame| project_frame_into_physical_mount(frame, mount_frame, scale_factor))
            .unwrap_or_default();
        projected.z_index = node.z_index;
        row_patches.insert(row, projected);
    }
    let changed_rows = row_patches.keys().copied().collect();
    Some(WorkbenchWindowNodePatch {
        nodes: previous_nodes.with_row_patches(row_patches),
        changed_rows,
    })
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
    to_host_contract_workbench_window_node_with_previous(node, node_index, None, 1.0)
}

fn to_host_contract_workbench_window_node_with_previous(
    node: &RetainedUiHostNodeModel,
    node_index: &ProjectionNodeIndex<'_>,
    previous: Option<&host_contract::TemplatePaneNodeData>,
    scale_factor: f32,
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
    if component_role.is_empty()
        && is_workbench_settings_window_mount(node.component.as_str(), control_id.as_str())
    {
        component_role = "settings-window".to_string();
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
    let settings_window_data =
        projected_settings_window_data(component_role.as_str(), &button_style_values);
    let settings_category_scroll_offset =
        numeric_property(&button_style_values, "settings_category_scroll_offset").unwrap_or(0.0)
            as f32;
    let settings_scroll_offset =
        numeric_property(&button_style_values, "settings_scroll_offset").unwrap_or(0.0) as f32;
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
    let has_preview_image = !media_source.trim().is_empty() || !icon_name.trim().is_empty();
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
        let (command_option_values, command_structured_options) =
            projected_command_palette_option_rows(component_role.as_str(), &button_style_values)
                .map(|(options, structured_options)| (Some(options), Some(structured_options)))
                .unwrap_or_default();
        let option_values = command_option_values
            .or(notification_option_values)
            .unwrap_or_else(|| string_array_property(&node.properties, "options", &node.options));
        let structured_options = command_structured_options
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
    let layout_padding_left =
        numeric_property(&node.properties, "layout_padding_left").unwrap_or(0.0) as f32;
    let layout_padding_right =
        numeric_property(&node.properties, "layout_padding_right").unwrap_or(0.0) as f32;
    let layout_padding_top =
        numeric_property(&node.properties, "layout_padding_top").unwrap_or(0.0) as f32;
    let layout_padding_bottom =
        numeric_property(&node.properties, "layout_padding_bottom").unwrap_or(0.0) as f32;
    let layout_spacing = numeric_property(&node.properties, "layout_spacing").unwrap_or(0.0) as f32;
    let layout_first_cell_offset_x =
        numeric_property(&node.properties, "layout_first_cell_offset_x")
            .or_else(|| numeric_property(&node.properties, "track_width_delta"))
            .unwrap_or(0.0) as f32;
    // These host fields carry either a physical offset or a slider semantic.
    // Resolve the authored property before the generic mount transform loses that distinction.
    let layout_second_cell_offset_x =
        numeric_property(&node.properties, "layout_second_cell_offset_x")
            .map(|value| scale_visual_metric(value as f32, scale_factor))
            .or_else(|| numeric_property(&node.properties, "range_min").map(|value| value as f32))
            .unwrap_or(0.0);
    let layout_third_cell_offset_x =
        numeric_property(&node.properties, "layout_third_cell_offset_x")
            .map(|value| scale_visual_metric(value as f32, scale_factor))
            .or_else(|| {
                numeric_property(&node.properties, "step_tick_count").map(|value| value as f32)
            })
            .unwrap_or(0.0);
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
        surface_node_id: node.surface_node_id,
        has_workbench_icon_tooltip: node.has_workbench_icon_tooltip,
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
        layout_padding_left,
        layout_padding_right,
        layout_padding_top,
        layout_padding_bottom,
        layout_spacing,
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
            .map(|value| value as f32)
            .or_else(|| {
                [
                    "dot_size",
                    "status_mark_size",
                    "arrow_size",
                    "track_width",
                    "icon_size",
                ]
                .into_iter()
                .find_map(|property| numeric_property(&node.properties, property))
                .map(|value| scale_visual_metric(value as f32, scale_factor))
            })
            .unwrap_or(0.0),
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
        has_preview_image,
        preview_image: Default::default(),
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
        settings_title: settings_window_data.title.into(),
        selected_settings_category_id: settings_window_data.selected_category_id.into(),
        settings_editor_open_key: settings_window_data.editor_open_key.into(),
        settings_editor_open_kind: settings_window_data.editor_open_kind.into(),
        settings_editor_open_row: settings_window_data.editor_open_row,
        settings_category_scroll_offset,
        settings_scroll_offset,
        settings_persistence_health_generation: settings_window_data.persistence_health_generation,
        settings_persistence_retry_scope: settings_window_data.persistence_retry_scope.into(),
        settings_persistence_status_text: settings_window_data.persistence_status_text.into(),
        settings_categories: model_rc(settings_window_data.categories),
        settings_entries: model_rc(settings_window_data.entries),
        collection_items: shared_string_list(collection_item_values),
        menu_items: shared_string_list(menu_item_values),
        structured_menu_items: model_rc(structured_menu_item_values),
        checked: node.checked,
        expanded: node.expanded,
        focused: node.focused,
        focus_visible: node.focus_visible,
        focus_visible_known: node.focus_visible_known,
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
