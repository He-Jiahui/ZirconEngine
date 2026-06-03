use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::load_preview_image;
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
    let Some(projection) = projection else {
        return ModelRc::default();
    };

    model_rc(
        projection
            .nodes
            .iter()
            .filter(|node| bool_property(&node.properties, ROOT_TEMPLATE_OVERLAY_PROPERTY))
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
        // source template clip here would split equivalent authored overlays
        // across separate painter paths.
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

fn bool_property(properties: &BTreeMap<String, RetainedUiHostValue>, key: &str) -> bool {
    matches!(properties.get(key), Some(RetainedUiHostValue::Bool(true)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::callback_dispatch::BuiltinHostWindowTemplateBridge;
    use crate::ui::template_runtime::RetainedUiHostComponentKind;
    use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

    #[test]
    fn host_projection_does_not_promote_workbench_reference_image_to_root_overlay_node() {
        let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1672.0, 941.0))
            .expect("builtin workbench host template should project");

        let overlay_nodes =
            to_host_contract_root_template_overlay_nodes(Some(bridge.host_projection()));

        assert_eq!(
            overlay_nodes.row_count(),
            0,
            "workbench shell must remain componentized instead of promoting the PNG reference"
        );
    }

    #[test]
    fn host_projection_converts_explicit_root_template_overlay_node() {
        let mut properties = BTreeMap::new();
        properties.insert(
            ROOT_TEMPLATE_OVERLAY_PROPERTY.to_owned(),
            RetainedUiHostValue::Bool(true),
        );
        properties.insert(
            "image".to_owned(),
            RetainedUiHostValue::String("zircon_editor_shell/toolbar/select.svg".to_owned()),
        );
        let projection = RetainedUiHostProjection {
            document_id: "test.root_overlay".to_owned(),
            nodes: vec![RetainedUiHostNodeModel {
                node_id: "overlay_select".to_owned(),
                parent_id: None,
                kind: RetainedUiHostComponentKind::Unknown,
                component: "Image".to_owned(),
                control_id: Some("ExplicitRootOverlay".to_owned()),
                frame: UiFrame::new(8.0, 12.0, 24.0, 24.0),
                clip_frame: None,
                z_index: 0,
                text: None,
                icon: None,
                component_role: None,
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
                properties,
                style_tokens: BTreeMap::new(),
                routes: Vec::new(),
            }],
        };

        let overlay_nodes = to_host_contract_root_template_overlay_nodes(Some(&projection));

        assert_eq!(overlay_nodes.row_count(), 1);
        let node = overlay_nodes
            .row_data(0)
            .expect("explicit root overlay should project");
        assert_eq!(node.control_id.as_str(), "ExplicitRootOverlay");
        assert_eq!(node.role.as_str(), "Image");
        assert_eq!(
            node.media_source.as_str(),
            "zircon_editor_shell/toolbar/select.svg"
        );
        assert!(node.has_preview_image);
        assert_eq!(node.frame.x, 8.0);
        assert_eq!(node.frame.y, 12.0);
        assert_eq!(node.frame.width, 24.0);
        assert_eq!(node.frame.height, 24.0);
    }
}
