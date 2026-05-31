use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::load_preview_image;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::template_runtime::{
    RetainedUiHostNodeModel, RetainedUiHostProjection, RetainedUiHostValue,
};
use zircon_runtime_interface::ui::layout::UiFrame;

pub(crate) const WORKBENCH_REFERENCE_IMAGE_CONTROL_ID: &str = "WorkbenchShellReferenceImage";

pub(crate) fn to_host_contract_root_template_overlay_nodes(
    projection: Option<&RetainedUiHostProjection>,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    let Some(projection) = projection else {
        return ModelRc::default();
    };

    model_rc(
        projection
            .nodes
            .iter()
            .filter(|node| node.control_id.as_deref() == Some(WORKBENCH_REFERENCE_IMAGE_CONTROL_ID))
            .map(to_host_contract_root_template_overlay_node)
            .collect(),
    )
}

fn to_host_contract_root_template_overlay_node(
    node: &RetainedUiHostNodeModel,
) -> host_contract::TemplatePaneNodeData {
    let media_source = first_string_property(&node.properties, &["image", "source", "media"])
        .or_else(|| {
            matches!(node.component.as_str(), "Image" | "SvgIcon")
                .then(|| string_property(&node.properties, "value"))
                .flatten()
        })
        .unwrap_or_default();
    let icon_name = first_string_property(&node.properties, &["icon"]).unwrap_or_default();
    let preview_image = load_preview_image(&media_source, &icon_name);
    let preview_size = preview_image.size();

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
        has_preview_image: preview_size.width > 0 && preview_size.height > 0,
        preview_image,
        // Root overlays are clipped by the final host frame. Carrying the
        // source template clip here sends the full-window reference image down
        // a different painter path than the hand-authored native overlay.
        has_clip_frame: false,
        clip_frame: host_contract::TemplateNodeFrameData::default(),
        frame: to_host_contract_template_frame(node.frame),
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

fn to_host_contract_template_frame(frame: UiFrame) -> host_contract::TemplateNodeFrameData {
    host_contract::TemplateNodeFrameData {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::callback_dispatch::BuiltinHostWindowTemplateBridge;
    use zircon_runtime_interface::ui::layout::UiSize;

    const WORKBENCH_REFERENCE_IMAGE_PATH: &str = "ui/editor/reference/workbench.png";

    #[test]
    fn host_projection_converts_workbench_reference_image_to_root_overlay_node() {
        let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1672.0, 941.0))
            .expect("builtin workbench host template should project");

        let overlay_nodes =
            to_host_contract_root_template_overlay_nodes(Some(bridge.host_projection()));

        assert_eq!(overlay_nodes.row_count(), 1);
        let node = overlay_nodes
            .row_data(0)
            .expect("workbench reference overlay node should project");
        assert_eq!(
            node.control_id.as_str(),
            WORKBENCH_REFERENCE_IMAGE_CONTROL_ID
        );
        assert_eq!(node.role.as_str(), "Image");
        assert_eq!(node.component_role.as_str(), "image");
        assert!(node.text.is_empty());
        assert!(node.value_text.is_empty());
        assert!(node.options_text.is_empty());
        assert!(node.icon_name.is_empty());
        assert!(!node.focused);
        assert!(!node.hovered);
        assert!(!node.pressed);
        assert!(!node.disabled);
        assert_eq!(node.media_source.as_str(), WORKBENCH_REFERENCE_IMAGE_PATH);
        assert!(node.has_preview_image);
        assert!(!node.has_clip_frame);
        assert_eq!(node.preview_image.size().width, 1672);
        assert_eq!(node.preview_image.size().height, 941);
        assert_eq!(node.frame.x, 0.0);
        assert_eq!(node.frame.y, 0.0);
        assert_eq!(node.frame.width, 1672.0);
        assert_eq!(node.frame.height, 941.0);
    }
}
