use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::template_runtime::{
    RetainedUiHostNodeModel, RetainedUiHostProjection, RetainedUiHostValue,
};
use zircon_runtime_interface::ui::layout::UiFrame;

const ROOT_TEMPLATE_OVERLAY_PROPERTY: &str = "root_template_overlay";

pub(crate) fn to_host_contract_root_template_overlay_nodes(
    projection: Option<&RetainedUiHostProjection>,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    to_host_contract_root_template_overlay_nodes_at_scale(projection, 1.0)
}

pub(crate) fn to_host_contract_root_template_overlay_nodes_at_scale(
    projection: Option<&RetainedUiHostProjection>,
    scale_factor: f32,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    let Some(projection) = projection else {
        return ModelRc::default();
    };
    let scale_factor = normalized_scale_factor(scale_factor);

    model_rc(
        projection
            .nodes
            .iter()
            .filter(|node| bool_property(&node.properties, ROOT_TEMPLATE_OVERLAY_PROPERTY))
            .map(|node| to_host_contract_root_template_overlay_node(node, scale_factor))
            .collect(),
    )
}

fn to_host_contract_root_template_overlay_node(
    node: &RetainedUiHostNodeModel,
    scale_factor: f32,
) -> host_contract::TemplatePaneNodeData {
    let media_source = first_string_property(&node.properties, &["image", "source", "media"])
        .or_else(|| {
            matches!(node.component.as_str(), "Image" | "SvgIcon")
                .then(|| string_property(&node.properties, "value"))
                .flatten()
        })
        .unwrap_or_default();
    let icon_name = first_string_property(&node.properties, &["icon"]).unwrap_or_default();
    let has_preview_image = !media_source.trim().is_empty() || !icon_name.trim().is_empty();

    host_contract::TemplatePaneNodeData {
        node_id: node.node_id.clone(),
        control_id: node.control_id.clone().unwrap_or_default(),
        role: resolve_root_overlay_role(node.component.as_str()).into(),
        component_role: node
            .component_role
            .clone()
            .unwrap_or_else(|| resolve_root_overlay_component_role(node.component.as_str()).into()),
        media_source,
        icon_name,
        has_preview_image,
        preview_image: Default::default(),
        // Root overlays are clipped by the final host frame. Carrying the
        // source template clip here would split equivalent authored overlays
        // across separate painter paths.
        has_clip_frame: false,
        clip_frame: host_contract::TemplateNodeFrameData::default(),
        frame: to_host_contract_template_frame(node.frame, scale_factor),
        ..host_contract::TemplatePaneNodeData::default()
    }
}

fn resolve_root_overlay_role(component: &str) -> &'static str {
    match component {
        "Image" => "Image",
        "SvgIcon" => "SvgIcon",
        "Icon" => "Icon",
        "IconButton" => "IconButton",
        "Button" => "Button",
        "Label" | "Text" => "Label",
        _ => "Mount",
    }
}

fn resolve_root_overlay_component_role(component: &str) -> &'static str {
    match component {
        "Image" => "image",
        "SvgIcon" => "svg-icon",
        "Icon" => "icon",
        "IconButton" => "icon-button",
        "Button" => "button",
        "Label" => "label",
        "Text" => "text",
        _ => "",
    }
}

fn to_host_contract_template_frame(
    frame: UiFrame,
    scale_factor: f32,
) -> host_contract::TemplateNodeFrameData {
    host_contract::TemplateNodeFrameData {
        x: scaled(frame.x, scale_factor),
        y: scaled(frame.y, scale_factor),
        width: scaled(frame.width, scale_factor),
        height: scaled(frame.height, scale_factor),
    }
}

fn normalized_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

fn scaled(value: f32, scale_factor: f32) -> f32 {
    let scaled = value * scale_factor;
    if scaled.is_finite() {
        scaled
    } else {
        0.0
    }
}

fn first_string_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    keys: &[&str],
) -> Option<String> {
    keys.iter().find_map(|key| string_property(properties, key))
}

fn string_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    key: &str,
) -> Option<String> {
    match properties.get(key) {
        Some(RetainedUiHostValue::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn bool_property(properties: &BTreeMap<String, RetainedUiHostValue>, key: &str) -> bool {
    matches!(properties.get(key), Some(RetainedUiHostValue::Bool(true)))
}

#[cfg(test)]
#[path = "tests/root_template_overlay.rs"]
mod tests;
