use std::sync::OnceLock;

use crate::ui::layouts::common::model_rc;
use crate::ui::slint_host as host_contract;
use crate::ui::template_runtime::SlintUiHostNodeProjection;
use slint::ModelRc;
use zircon_runtime::ui::component::{UiComponentDescriptorRegistry, UiValue};

mod collection_fields;
mod preview_images;
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
    let collection_items = node
        .attributes
        .get("collection_items")
        .and_then(value_as_options)
        .unwrap_or_default();
    let collection_fields =
        collection_fields_for_component(&component, &node.attributes, &node.bindings);
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
    Some(host_contract::TemplatePaneNodeData {
        node_id: node.node_id.into(),
        control_id: control_id.into(),
        role: component.into(),
        text: node
            .attributes
            .get("text")
            .or_else(|| node.attributes.get("label"))
            .and_then(value_as_string)
            .unwrap_or_default()
            .into(),
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
        begin_drag_action_id: begin_drag_action_id.into(),
        drag_action_id: drag_action_id.into(),
        end_drag_action_id: end_drag_action_id.into(),
        commit_action_id: commit_action_id.into(),
        edit_action_id: edit_action_id.into(),
        surface_variant: "".into(),
        text_tone: "".into(),
        button_variant: "".into(),
        font_size: 0.0,
        font_weight: 0,
        text_align: "left".into(),
        overflow: "".into(),
        corner_radius: 0.0,
        border_width: 0.0,
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

fn runtime_component_registry() -> &'static UiComponentDescriptorRegistry {
    static UI_COMPONENT_REGISTRY: OnceLock<UiComponentDescriptorRegistry> = OnceLock::new();
    UI_COMPONENT_REGISTRY.get_or_init(UiComponentDescriptorRegistry::editor_showcase)
}
