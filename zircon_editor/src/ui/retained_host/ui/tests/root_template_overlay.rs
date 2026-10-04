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
        source_surface_frame: None,
        document_id: "test.root_overlay".to_owned(),
        nodes: vec![RetainedUiHostNodeModel {
            source_path: None,
            source_node_id: None,
            instance_path: None,
            source_surface_frame: None,
            node_id: "overlay_select".to_owned(),
            surface_node_id: None,
            has_workbench_icon_tooltip: false,
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
            focus_visible: false,
            focus_visible_known: false,
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
    assert_eq!(node.preview_image.size().width, 0);
    assert_eq!(node.preview_image.size().height, 0);
    assert_eq!(node.frame.x, 8.0);
    assert_eq!(node.frame.y, 12.0);
    assert_eq!(node.frame.width, 24.0);
    assert_eq!(node.frame.height, 24.0);
}

#[test]
fn root_overlay_projection_scales_logical_geometry_once() {
    let mut properties = BTreeMap::new();
    properties.insert(
        ROOT_TEMPLATE_OVERLAY_PROPERTY.to_owned(),
        RetainedUiHostValue::Bool(true),
    );
    let projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "test.scaled_root_overlay".to_owned(),
        nodes: vec![RetainedUiHostNodeModel {
            source_path: None,
            source_node_id: None,
            instance_path: None,
            source_surface_frame: None,
            node_id: "scaled_overlay".to_owned(),
            surface_node_id: None,
            has_workbench_icon_tooltip: false,
            parent_id: None,
            kind: RetainedUiHostComponentKind::Unknown,
            component: "Image".to_owned(),
            control_id: Some("ScaledRootOverlay".to_owned()),
            frame: UiFrame::new(8.0, 12.0, 24.0, 18.0),
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
            focus_visible: false,
            focus_visible_known: false,
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

    let nodes = to_host_contract_root_template_overlay_nodes_at_scale(Some(&projection), 2.0);
    let node = nodes.get(0).expect("scaled overlay");
    assert_eq!(node.frame.x, 16.0);
    assert_eq!(node.frame.y, 24.0);
    assert_eq!(node.frame.width, 48.0);
    assert_eq!(node.frame.height, 36.0);
}
