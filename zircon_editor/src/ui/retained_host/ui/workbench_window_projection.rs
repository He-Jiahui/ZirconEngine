use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::load_preview_image;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::template_runtime::{
    RetainedUiHostNodeModel, RetainedUiHostProjection, RetainedUiHostRouteProjection,
    RetainedUiHostValue,
};
use zircon_runtime::ui::style::resolve_button_style_from_values;
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiFrame};

use super::component_contract_metadata::tokens_for_component_role;
use super::pane_data_conversion::{
    projected_command_palette_options, projected_command_palette_structured_options,
    projected_notification_center_options, projected_notification_center_structured_options,
    projected_notification_center_value_text, structured_menu_items, structured_options_for_node,
};
use super::template_layout_context::apply_table_layout_context_variant;

const WORKBENCH_STATUS_RIGHT_OFFSET_Y: f64 = -0.5;
const WORKBENCH_STATUS_RIGHT_TEXT_COLOR: host_contract::primitives::Color =
    host_contract::primitives::Color::from_rgb_u8(125, 137, 144);
const WORKBENCH_SELECTION_SELECTED_SURFACE: &str = "#173942";
const WORKBENCH_SELECTION_ACCENT: &str = "#2aa6b8";
const WORKBENCH_RADIO_SELECTED_SURFACE: &str = "#1b272d";
const WORKBENCH_RADIO_SELECTED_BORDER: &str = "#4c5b63";
const WORKBENCH_TOGGLE_SELECTED_THUMB: &str = "#e8ecee";
const UI_HOST_WINDOW_ROOT_CONTROL_ID: &str = "UiHostWindowRoot";

pub(crate) fn to_host_contract_workbench_window_nodes(
    projection: Option<&RetainedUiHostProjection>,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    let Some(projection) = projection else {
        return ModelRc::default();
    };

    let nodes_by_id = projection
        .nodes
        .iter()
        .map(|node| (node.node_id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let layout_context_width = projection_layout_context_width(projection);
    model_rc(
        projection
            .nodes
            .iter()
            .filter(|node| host_projection_node_render_visible(node, &nodes_by_id))
            .filter_map(|node| to_host_contract_workbench_window_node(node, &nodes_by_id))
            .map(|node| apply_table_layout_context_variant(node, layout_context_width))
            .collect(),
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

fn host_projection_node_render_visible(
    node: &RetainedUiHostNodeModel,
    nodes_by_id: &BTreeMap<&str, &RetainedUiHostNodeModel>,
) -> bool {
    let mut current = Some(node);
    while let Some(current_node) = current {
        if !node_properties_render_visible(current_node) {
            return false;
        }
        current = current_node
            .parent_id
            .as_deref()
            .and_then(|parent_id| nodes_by_id.get(parent_id).copied());
    }
    true
}

fn node_properties_render_visible(node: &RetainedUiHostNodeModel) -> bool {
    let visibility = string_property(&node.properties, "visibility")
        .unwrap_or_else(|| "visible".to_string())
        .replace('_', "")
        .to_ascii_lowercase();
    if !legacy_visible_property(&node.properties) && visibility != "collapsed" {
        return false;
    }
    matches!(
        visibility.as_str(),
        "visible" | "hittestinvisible" | "selfhittestinvisible"
    )
}

fn legacy_visible_property(properties: &BTreeMap<String, RetainedUiHostValue>) -> bool {
    match properties.get("visible") {
        Some(RetainedUiHostValue::Bool(value)) => *value,
        Some(RetainedUiHostValue::String(value)) => value.parse().unwrap_or(true),
        _ => true,
    }
}

fn to_host_contract_workbench_window_node(
    node: &RetainedUiHostNodeModel,
    nodes_by_id: &BTreeMap<&str, &RetainedUiHostNodeModel>,
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
    let mut button_style_values = toml_values_from_host_properties(&node.properties);
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
    let option_values =
        projected_command_palette_options(component_role.as_str(), &button_style_values)
            .or_else(|| {
                projected_notification_center_options(component_role.as_str(), &button_style_values)
            })
            .unwrap_or_else(|| string_array_property(&node.properties, "options", &node.options));
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
    let structured_options =
        projected_command_palette_structured_options(component_role.as_str(), &button_style_values)
            .or_else(|| {
                projected_notification_center_structured_options(
                    component_role.as_str(),
                    &button_style_values,
                )
            })
            .unwrap_or_else(|| structured_options_for_node(&option_values, &button_style_values));
    let surface_variant = first_string_property(&node.properties, &["surface_variant"])
        .or_else(|| default_workbench_surface_variant(&node.component, &component_role))
        .unwrap_or_default();
    let component_variant = first_string_property(
        &node.properties,
        &["component_variant", "variant", "mui_variant"],
    )
    .unwrap_or_default();
    let component_tokens = tokens_for_component_role(&node.component, component_role.as_str());
    let text_tone = first_string_property(&node.properties, &["text_tone"])
        .unwrap_or_else(|| default_text_tone(&node.component, &component_role, &surface_variant));
    let label_text = first_string_property(&node.properties, &["label_text"]).unwrap_or_default();
    let label_color = color_property(&node.properties, "label_color")
        .or_else(|| color_property(&node.properties, "icon_fill"))
        .or_else(|| color_property(&node.properties, "status_mark_color"))
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
            inherited_status_right_numeric_property(node, nodes_by_id, "status_right_offset_y")
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
            .or_else(|| {
                inherited_status_right_color_property(node, nodes_by_id, "status_right_text_color")
            })
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
        options_text: option_values.join(", ").into(),
        options: shared_string_list(option_values),
        structured_options: model_rc(structured_options),
        collection_items: shared_string_list(collection_item_values),
        menu_items: shared_string_list(menu_item_values.clone()),
        structured_menu_items: model_rc(structured_menu_items(&menu_item_values)),
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

fn inherited_status_right_numeric_property(
    node: &RetainedUiHostNodeModel,
    nodes_by_id: &BTreeMap<&str, &RetainedUiHostNodeModel>,
    property: &str,
) -> Option<f64> {
    inherited_status_right_parent(node, nodes_by_id)
        .and_then(|parent| numeric_property(&parent.properties, property))
        .or_else(|| {
            (property == "status_right_offset_y" && is_status_right_control(node))
                .then_some(WORKBENCH_STATUS_RIGHT_OFFSET_Y)
        })
}

fn inherited_status_right_color_property(
    node: &RetainedUiHostNodeModel,
    nodes_by_id: &BTreeMap<&str, &RetainedUiHostNodeModel>,
    property: &str,
) -> Option<host_contract::primitives::Color> {
    inherited_status_right_parent(node, nodes_by_id)
        .and_then(|parent| color_property(&parent.properties, property))
        .or_else(|| {
            (property == "status_right_text_color" && is_status_right_control(node))
                .then_some(WORKBENCH_STATUS_RIGHT_TEXT_COLOR)
        })
}

fn inherited_status_right_parent<'a>(
    node: &RetainedUiHostNodeModel,
    nodes_by_id: &'a BTreeMap<&str, &RetainedUiHostNodeModel>,
) -> Option<&'a RetainedUiHostNodeModel> {
    if !is_status_right_control(node) {
        return None;
    }

    let mut parent_id = node.parent_id.as_deref();
    while let Some(current_parent_id) = parent_id {
        let parent = nodes_by_id.get(current_parent_id).copied()?;
        if parent.control_id.as_deref() == Some("WorkbenchWindowStatusBar") {
            return Some(parent);
        }
        parent_id = parent.parent_id.as_deref();
    }
    None
}

fn is_status_right_control(node: &RetainedUiHostNodeModel) -> bool {
    matches!(
        node.control_id.as_deref(),
        Some(
            "WorkbenchStatusGrid"
                | "WorkbenchStatusSnap"
                | "WorkbenchStatusTaskProgress"
                | "WorkbenchStatusTaskLabel"
                | "WorkbenchStatusTaskBar"
                | "WorkbenchStatusSnapToggle"
                | "WorkbenchStatusWorld"
                | "WorkbenchStatusTarget"
                | "WorkbenchStatusZoom"
        )
    )
}

fn is_cleared_inspector_property_row(
    control_id: &str,
    properties: &BTreeMap<String, RetainedUiHostValue>,
) -> bool {
    if !matches!(
        control_id,
        "WorkbenchMeshRow"
            | "WorkbenchMaterialRow"
            | "WorkbenchComponentPropertySlot03Row"
            | "WorkbenchComponentPropertySlot04Row"
    ) && !control_id.starts_with("WorkbenchComponentPropertyVirtualRow")
    {
        return false;
    }

    first_string_property(properties, &["text"]).is_none_or(|text| text.is_empty())
        && first_string_property(properties, &["value_text"]).is_none_or(|value| value.is_empty())
}

fn clear_button_surface_style_values(values: &mut BTreeMap<String, toml::Value>) {
    for key in ["background", "background_color", "border", "border_color"] {
        values.remove(key);
    }
}

fn normalize_workbench_selection_control_style_values(
    values: &mut BTreeMap<String, toml::Value>,
    node: &RetainedUiHostNodeModel,
    component_role: &str,
) {
    if !active_workbench_selection_control(node) {
        return;
    }

    if is_workbench_checkbox_control(node, component_role) {
        set_toml_string_aliases(
            values,
            &["background", "background_color"],
            WORKBENCH_SELECTION_SELECTED_SURFACE,
        );
        set_toml_string_aliases(
            values,
            &["border", "border_color"],
            WORKBENCH_SELECTION_ACCENT,
        );
    } else if is_workbench_radio_control(node, component_role) {
        set_toml_string_aliases(
            values,
            &["background", "background_color"],
            WORKBENCH_RADIO_SELECTED_SURFACE,
        );
        set_toml_string_aliases(
            values,
            &["border", "border_color"],
            WORKBENCH_RADIO_SELECTED_BORDER,
        );
    } else if is_workbench_toggle_control(node, component_role) {
        set_toml_string_aliases(
            values,
            &["background", "background_color"],
            WORKBENCH_SELECTION_SELECTED_SURFACE,
        );
        set_toml_string_aliases(
            values,
            &["border", "border_color"],
            WORKBENCH_SELECTION_ACCENT,
        );
        set_toml_string_aliases(
            values,
            &["foreground", "foreground_color"],
            WORKBENCH_TOGGLE_SELECTED_THUMB,
        );
    }
}

fn active_workbench_selection_control(node: &RetainedUiHostNodeModel) -> bool {
    node.checked
        || bool_property(&node.properties, "checked")
        || bool_property(&node.properties, "selected")
}

fn is_workbench_checkbox_control(node: &RetainedUiHostNodeModel, component_role: &str) -> bool {
    component_role == "checkbox"
        || matches!(node.component.as_str(), "Checkbox" | "WorkbenchCheckbox")
        || node
            .control_id
            .as_deref()
            .is_some_and(|control_id| control_id.contains("Checkbox"))
}

fn is_workbench_radio_control(node: &RetainedUiHostNodeModel, component_role: &str) -> bool {
    component_role == "radio"
        || matches!(node.component.as_str(), "Radio" | "WorkbenchRadio")
        || node
            .control_id
            .as_deref()
            .is_some_and(|control_id| control_id.contains("Radio"))
}

fn is_workbench_toggle_control(node: &RetainedUiHostNodeModel, component_role: &str) -> bool {
    component_role == "toggle"
        || matches!(
            node.component.as_str(),
            "Toggle" | "Switch" | "WorkbenchToggle" | "WorkbenchSwitch"
        )
        || node
            .control_id
            .as_deref()
            .is_some_and(|control_id| control_id.contains("Toggle"))
}

fn set_toml_string_aliases(values: &mut BTreeMap<String, toml::Value>, keys: &[&str], value: &str) {
    for key in keys {
        values.insert((*key).to_string(), toml::Value::String(value.to_string()));
    }
}

fn projected_workbench_text(node: &RetainedUiHostNodeModel, component_role: &str) -> String {
    let authored_text = first_string_property(&node.properties, &["text", "label"]);
    if prefers_authored_text_over_rendered_text(node.component.as_str(), component_role) {
        authored_text
            .or_else(|| node.text.clone())
            .unwrap_or_default()
    } else {
        node.text.clone().or(authored_text).unwrap_or_default()
    }
}

fn prefers_authored_text_over_rendered_text(component: &str, component_role: &str) -> bool {
    matches!(
        component_role,
        "button"
            | "toggle"
            | "tab"
            | "tabs"
            | "tab-list"
            | "segmented-control"
            | "checkbox"
            | "radio"
            | "icon-button"
    ) || matches!(
        component,
        "Button"
            | "Toggle"
            | "ToggleButton"
            | "Switch"
            | "Checkbox"
            | "Radio"
            | "RadioField"
            | "SegmentedControl"
            | "Tab"
            | "Tabs"
            | "TabList"
            | "IconButton"
    )
}

fn projected_workbench_value_text(
    node: &RetainedUiHostNodeModel,
    component_role: &str,
    button_style_values: &BTreeMap<String, toml::Value>,
) -> String {
    display_node_value_text(node, component_role)
        .or_else(|| projected_notification_center_value_text(component_role, button_style_values))
        .or_else(|| first_string_property(&node.properties, &["value_text"]))
        .or_else(|| display_value_property_for_node(node, component_role))
        .unwrap_or_default()
}

fn display_node_value_text(node: &RetainedUiHostNodeModel, component_role: &str) -> Option<String> {
    if !uses_value_property_as_display_text(node.component.as_str(), component_role) {
        return None;
    }

    node.value_text.clone()
}

fn display_value_property_for_node(
    node: &RetainedUiHostNodeModel,
    component_role: &str,
) -> Option<String> {
    if !uses_value_property_as_display_text(node.component.as_str(), component_role) {
        return None;
    }

    first_string_property(&node.properties, &["value"])
}

fn uses_value_property_as_display_text(component: &str, component_role: &str) -> bool {
    matches!(
        component_role,
        "input-field"
            | "number-field"
            | "range-field"
            | "slider"
            | "range-slider"
            | "segmented-control"
            | "combo-box"
            | "dropdown"
            | "enum-field"
            | "flags-field"
            | "search-select"
            | "asset-field"
            | "object-field"
            | "instance-field"
    ) || matches!(
        component,
        "InputField"
            | "TextField"
            | "LineEdit"
            | "NumberField"
            | "RangeField"
            | "Slider"
            | "RangeSlider"
            | "SegmentedControl"
            | "ComboBox"
            | "Dropdown"
            | "EnumField"
            | "FlagsField"
            | "SearchSelect"
            | "AssetField"
            | "ObjectField"
            | "InstanceField"
    )
}

fn resolve_workbench_role(component: &str) -> &'static str {
    match component {
        "Button" => "Button",
        "IconButton" => "IconButton",
        "ComboBox" | "Dropdown" | "SearchSelect" => "Dropdown",
        "ContextActionMenu" | "ContextMenu" | "Menu" | "PopupMenu" => "Menu",
        "InputField" | "TextField" | "NumberField" => "InputField",
        "Checkbox" => "Checkbox",
        "Radio" => "Radio",
        "RangeField" | "Slider" => "Slider",
        "Toggle" | "Switch" => "Toggle",
        "Table" | "EditableTable" => "Table",
        "Image" => "Image",
        "SvgIcon" => "SvgIcon",
        "Icon" => "Icon",
        "Tooltip" => "Tooltip",
        "NotificationCenter" => "NotificationCenter",
        "Label" | "Text" => "Label",
        _ => "Mount",
    }
}

fn default_workbench_surface_variant(component: &str, component_role: &str) -> Option<String> {
    match (component, component_role) {
        (_, "button") | ("Button", _) | ("IconButton", _) => Some("panel".to_string()),
        ("InputField", _) | ("TextField", _) | ("NumberField", _) => Some("inset".to_string()),
        ("Label", _) | ("Text", _) => None,
        _ => Some("panel".to_string()),
    }
}

fn is_workbench_command_palette_mount(component: &str, control_id: &str) -> bool {
    component == "WorkbenchCommandPalette" || control_id == "WorkbenchCommandPalette"
}

fn is_workbench_notification_center_mount(component: &str, control_id: &str) -> bool {
    component == "NotificationCenter"
        || component == "WorkbenchNotificationCenter"
        || control_id == "WorkbenchNotificationCenter"
}

fn default_text_tone(component: &str, component_role: &str, surface_variant: &str) -> String {
    if matches!(component, "Image" | "SvgIcon" | "Icon" | "IconButton") {
        "muted".to_string()
    } else if matches!(component_role, "button") || matches!(surface_variant, "accent" | "primary")
    {
        "primary".to_string()
    } else {
        String::new()
    }
}

fn default_corner_radius(component: &str, component_role: &str) -> f64 {
    match (component, component_role) {
        ("Button", _) | ("IconButton", _) | ("InputField", _) | (_, "button") => 5.0,
        ("Label", _) | ("Text", _) => 0.0,
        _ => 4.0,
    }
}

fn default_border_width(
    component: &str,
    component_role: &str,
    surface_variant: &str,
) -> Option<f64> {
    if matches!(component, "Label" | "Text") && surface_variant.is_empty() {
        return None;
    }
    if matches!(component_role, "button")
        || matches!(
            component,
            "Button" | "IconButton" | "InputField" | "TextField" | "NumberField"
        )
        || !surface_variant.is_empty()
    {
        Some(1.0)
    } else {
        None
    }
}

fn template_frame(frame: UiFrame) -> host_contract::TemplateNodeFrameData {
    host_contract::TemplateNodeFrameData {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

fn shared_string_list(
    values: Vec<String>,
) -> ModelRc<crate::ui::retained_host::primitives::SharedString> {
    model_rc(values.into_iter().map(Into::into).collect())
}

fn first_string_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    keys: &[&str],
) -> Option<String> {
    keys.iter()
        .find_map(|key| string_property(properties, key))
        .filter(|value| !value.is_empty())
}

fn string_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    key: &str,
) -> Option<String> {
    match properties.get(key) {
        Some(RetainedUiHostValue::String(value)) => Some(value.clone()),
        Some(RetainedUiHostValue::Integer(value)) => Some(value.to_string()),
        Some(RetainedUiHostValue::Float(value)) => Some(value.to_string()),
        Some(RetainedUiHostValue::Bool(value)) => Some(value.to_string()),
        _ => None,
    }
}

fn numeric_property(properties: &BTreeMap<String, RetainedUiHostValue>, key: &str) -> Option<f64> {
    match properties.get(key) {
        Some(RetainedUiHostValue::Float(value)) => Some(*value),
        Some(RetainedUiHostValue::Integer(value)) => Some(*value as f64),
        Some(RetainedUiHostValue::String(value)) => value.parse().ok(),
        _ => None,
    }
}

fn integer_property(properties: &BTreeMap<String, RetainedUiHostValue>, key: &str) -> Option<i32> {
    match properties.get(key) {
        Some(RetainedUiHostValue::Integer(value)) => i32::try_from(*value).ok(),
        Some(RetainedUiHostValue::Float(value)) => Some(*value as i32),
        Some(RetainedUiHostValue::String(value)) => value.parse().ok(),
        _ => None,
    }
}

fn bool_property(properties: &BTreeMap<String, RetainedUiHostValue>, key: &str) -> bool {
    match properties.get(key) {
        Some(RetainedUiHostValue::Bool(value)) => *value,
        Some(RetainedUiHostValue::String(value)) => value.parse().unwrap_or(false),
        _ => false,
    }
}

fn normalized_percent(properties: &BTreeMap<String, RetainedUiHostValue>) -> f32 {
    let Some(value) = numeric_property(properties, "value") else {
        return 0.0;
    };
    let min = numeric_property(properties, "min").unwrap_or(0.0);
    let max = numeric_property(properties, "max").unwrap_or(100.0);
    if max <= min {
        0.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0) as f32
    }
}

fn string_array_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    key: &str,
    fallback: &[String],
) -> Vec<String> {
    match properties.get(key) {
        Some(RetainedUiHostValue::Array(values)) => values
            .iter()
            .filter_map(host_value_display_text)
            .filter(|value| !value.is_empty())
            .collect(),
        Some(value) => host_value_display_text(value).into_iter().collect(),
        None => fallback.to_vec(),
    }
}

fn host_value_display_text(value: &RetainedUiHostValue) -> Option<String> {
    match value {
        RetainedUiHostValue::String(value) => Some(value.clone()),
        RetainedUiHostValue::Integer(value) => Some(value.to_string()),
        RetainedUiHostValue::Float(value) => Some(value.to_string()),
        RetainedUiHostValue::Bool(value) => Some(value.to_string()),
        RetainedUiHostValue::Datetime(value) => Some(value.clone()),
        RetainedUiHostValue::Array(_) | RetainedUiHostValue::Table(_) => None,
    }
}

fn preferred_route_binding<const N: usize>(
    routes: &[RetainedUiHostRouteProjection],
    kinds: [UiEventKind; N],
) -> Option<String> {
    kinds
        .iter()
        .find_map(|kind| routes.iter().find(|route| route.event_kind == *kind))
        .or_else(|| routes.first())
        .map(|route| route.binding_id.clone())
}

fn preferred_route_action_id<const N: usize>(
    routes: &[RetainedUiHostRouteProjection],
    kinds: [UiEventKind; N],
) -> Option<String> {
    preferred_route(routes, kinds).map(|route| {
        if route.action_id.is_empty() {
            route.binding_id.clone()
        } else {
            route.action_id.clone()
        }
    })
}

fn preferred_route<const N: usize>(
    routes: &[RetainedUiHostRouteProjection],
    kinds: [UiEventKind; N],
) -> Option<&RetainedUiHostRouteProjection> {
    kinds
        .iter()
        .find_map(|kind| routes.iter().find(|route| route.event_kind == *kind))
        .or_else(|| routes.first())
}

fn color_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    key: &str,
) -> Option<crate::ui::retained_host::primitives::Color> {
    let RetainedUiHostValue::String(value) = properties.get(key)? else {
        return None;
    };
    let rgba = parse_hex_rgba(value)?;
    Some(crate::ui::retained_host::primitives::Color::from_argb_u8(
        rgba[3], rgba[0], rgba[1], rgba[2],
    ))
}

fn parse_hex_rgba(raw: &str) -> Option<[u8; 4]> {
    let hex = raw.trim().strip_prefix('#')?;
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    match hex.len() {
        6 => Some([channel(0..2)?, channel(2..4)?, channel(4..6)?, 255]),
        8 => Some([
            channel(0..2)?,
            channel(2..4)?,
            channel(4..6)?,
            channel(6..8)?,
        ]),
        _ => None,
    }
}

fn toml_values_from_host_properties(
    properties: &BTreeMap<String, RetainedUiHostValue>,
) -> BTreeMap<String, toml::Value> {
    let mut values = properties
        .iter()
        .filter_map(|(key, value)| Some((key.clone(), toml_value_from_host_value(value)?)))
        .collect::<BTreeMap<_, _>>();
    alias_toml_value_key(&mut values, "focus_border_color", "border_color");
    alias_toml_value_key(&mut values, "thumb_outline_color", "border_color");
    alias_toml_value_key(&mut values, "disabled_opacity", "opacity");
    values
}

fn alias_toml_value_key(values: &mut BTreeMap<String, toml::Value>, source: &str, target: &str) {
    if values.contains_key(target) {
        return;
    }
    if let Some(value) = values.get(source).cloned() {
        values.insert(target.to_string(), value);
    }
}

fn toml_value_from_host_value(value: &RetainedUiHostValue) -> Option<toml::Value> {
    match value {
        RetainedUiHostValue::String(value) => Some(toml::Value::String(value.clone())),
        RetainedUiHostValue::Integer(value) => Some(toml::Value::Integer(*value)),
        RetainedUiHostValue::Float(value) => Some(toml::Value::Float(*value)),
        RetainedUiHostValue::Bool(value) => Some(toml::Value::Boolean(*value)),
        RetainedUiHostValue::Datetime(value) => value.parse().ok().map(toml::Value::Datetime),
        RetainedUiHostValue::Array(values) => Some(toml::Value::Array(
            values
                .iter()
                .filter_map(toml_value_from_host_value)
                .collect(),
        )),
        RetainedUiHostValue::Table(values) => Some(toml::Value::Table(
            values
                .iter()
                .filter_map(|(key, value)| Some((key.clone(), toml_value_from_host_value(value)?)))
                .collect(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::template_runtime::RetainedUiHostComponentKind;

    #[test]
    fn workbench_button_text_prefers_authored_label_over_value_render_text() {
        let node = test_host_node(
            "Button",
            "button",
            Some("thumbnail"),
            [("text", "Thumb"), ("value", "thumbnail")],
        );

        assert_eq!(projected_workbench_text(&node, "button"), "Thumb");
    }

    #[test]
    fn workbench_input_text_keeps_rendered_value_display_semantics() {
        let node = test_host_node(
            "TextField",
            "input-field",
            Some("albedo"),
            [("text", "Search"), ("value", "albedo")],
        );

        assert_eq!(projected_workbench_text(&node, "input-field"), "albedo");
    }

    #[test]
    fn workbench_segmented_control_projects_selected_value_text() {
        let node = test_host_node(
            "SegmentedControl",
            "segmented-control",
            None,
            [("value", "grid")],
        );

        assert_eq!(
            projected_workbench_value_text(&node, "segmented-control", &BTreeMap::new()),
            "grid"
        );
    }

    fn test_host_node<const N: usize>(
        component: &str,
        component_role: &str,
        text: Option<&str>,
        properties: [(&str, &str); N],
    ) -> RetainedUiHostNodeModel {
        RetainedUiHostNodeModel {
            node_id: "test-node".to_string(),
            parent_id: None,
            kind: RetainedUiHostComponentKind::from_component(component),
            component: component.to_string(),
            control_id: Some("test-control".to_string()),
            frame: UiFrame::new(0.0, 0.0, 100.0, 24.0),
            clip_frame: None,
            z_index: 0,
            text: text.map(str::to_string),
            icon: None,
            component_role: Some(component_role.to_string()),
            value_text: None,
            validation_level: None,
            validation_message: None,
            popup_open: false,
            has_popup_anchor: false,
            popup_anchor_x: 0.0,
            popup_anchor_y: 0.0,
            selection_state: None,
            options_text: None,
            options: Vec::new(),
            collection_items: Vec::new(),
            menu_items: Vec::new(),
            accepted_drag_payloads: Vec::new(),
            drop_source_summary: None,
            checked: false,
            expanded: false,
            focused: false,
            hovered: false,
            pressed: false,
            dragging: false,
            drop_hovered: false,
            active_drag_target: false,
            disabled: false,
            properties: properties
                .into_iter()
                .map(|(key, value)| {
                    (
                        key.to_string(),
                        RetainedUiHostValue::String(value.to_string()),
                    )
                })
                .collect(),
            style_tokens: BTreeMap::new(),
            routes: Vec::new(),
        }
    }
}
