use super::pane_ui_asset_conversion::to_host_contract_ui_asset_pane;
use crate::ui::asset_editor;
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};

#[test]
fn ui_asset_pane_conversion_preserves_widget_prop_state_items() {
    let data = asset_editor::UiAssetEditorPanePresentation {
        nodes: vec![
            ViewTemplateNodeData {
                node_id: "ui_asset/inspector/widget_section".into(),
                control_id: "InspectorWidgetSection".into(),
                role: "Panel".into(),
                frame: ViewTemplateFrameData {
                    x: 10.0,
                    y: 20.0,
                    width: 260.0,
                    height: 32.0,
                },
                ..ViewTemplateNodeData::default()
            },
            ViewTemplateNodeData {
                node_id: "ui_asset/inspector/promote_section".into(),
                control_id: "InspectorPromoteSection".into(),
                role: "Panel".into(),
                frame: ViewTemplateFrameData {
                    x: 10.0,
                    y: 56.0,
                    width: 260.0,
                    height: 32.0,
                },
                ..ViewTemplateNodeData::default()
            },
            ViewTemplateNodeData {
                node_id: "ui_asset/inspector/slot_section".into(),
                control_id: "InspectorSlotSection".into(),
                role: "Panel".into(),
                frame: ViewTemplateFrameData {
                    x: 10.0,
                    y: 92.0,
                    width: 260.0,
                    height: 32.0,
                },
                ..ViewTemplateNodeData::default()
            },
            ViewTemplateNodeData {
                node_id: "ui_asset/inspector/layout_section".into(),
                control_id: "InspectorLayoutSection".into(),
                role: "Panel".into(),
                frame: ViewTemplateFrameData {
                    x: 10.0,
                    y: 128.0,
                    width: 260.0,
                    height: 32.0,
                },
                ..ViewTemplateNodeData::default()
            },
            ViewTemplateNodeData {
                node_id: "ui_asset/inspector/binding_section".into(),
                control_id: "InspectorBindingSection".into(),
                role: "Panel".into(),
                frame: ViewTemplateFrameData {
                    x: 10.0,
                    y: 164.0,
                    width: 260.0,
                    height: 32.0,
                },
                ..ViewTemplateNodeData::default()
            },
        ],
        inspector_control_id: "ActionButton".to_string(),
        inspector_can_edit_control_id: true,
        inspector_mount: "content".to_string(),
        inspector_slot_padding: "4".to_string(),
        inspector_slot_width_preferred: "120".to_string(),
        inspector_slot_linear_main_weight: "1".to_string(),
        inspector_layout_width_preferred: "320".to_string(),
        inspector_layout_box_gap: "8".to_string(),
        inspector_binding_id: "SaveButton/onClick".to_string(),
        inspector_binding_event: "Click".to_string(),
        inspector_binding_route: "route.save".to_string(),
        inspector_binding_route_target: "editor".to_string(),
        inspector_binding_action_target: "save_project".to_string(),
        inspector_can_edit_binding: true,
        inspector_widget_prop_state_items: vec![
            "prop variant = \"outlined\"".to_string(),
            "state expanded = true".to_string(),
        ],
        inspector_widget_prop_state_rows: vec![
            asset_editor::UiAssetEditorWidgetPropStateItem {
                kind: "prop".to_string(),
                path: "variant".to_string(),
                value: "\"outlined\"".to_string(),
                display: "prop variant = \"outlined\"".to_string(),
            },
            asset_editor::UiAssetEditorWidgetPropStateItem {
                kind: "state".to_string(),
                path: "expanded".to_string(),
                value: "true".to_string(),
                display: "state expanded = true".to_string(),
            },
        ],
        inspector_items: vec![
            "mode Design".to_string(),
            "prop variant = \"outlined\"".to_string(),
            "state expanded = true".to_string(),
        ],
        ..Default::default()
    };

    let host = to_host_contract_ui_asset_pane(data, "ui-asset-instance");
    let prop_state_items = host
        .inspector
        .widget
        .prop_state_items
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let inspector_items = host
        .inspector
        .widget
        .items
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let prop_state_rows = host
        .inspector
        .widget
        .prop_state_rows
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    assert_eq!(
        prop_state_items,
        vec![
            "prop variant = \"outlined\"".to_string(),
            "state expanded = true".to_string()
        ]
    );
    assert_eq!(prop_state_rows.len(), 2);
    assert_eq!(prop_state_rows[0].kind, "prop");
    assert_eq!(prop_state_rows[0].path, "variant");
    assert_eq!(prop_state_rows[0].value, "\"outlined\"");
    assert_eq!(prop_state_rows[0].display, "prop variant = \"outlined\"");
    assert_eq!(prop_state_rows[1].kind, "state");
    assert_eq!(prop_state_rows[1].path, "expanded");
    assert_eq!(prop_state_rows[1].value, "true");
    assert_eq!(prop_state_rows[1].display, "state expanded = true");
    assert_eq!(
        inspector_items,
        vec![
            "mode Design".to_string(),
            "prop variant = \"outlined\"".to_string(),
            "state expanded = true".to_string()
        ]
    );

    let nodes = host.nodes.iter().cloned().collect::<Vec<_>>();
    let control_id_node = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "UiAssetWidgetFieldControlIdValue")
        .expect("widget control id should append editable value input");
    assert_eq!(control_id_node.value_text.as_str(), "ActionButton");
    assert_eq!(control_id_node.dispatch_kind.as_str(), "commit_only");
    assert_eq!(
        control_id_node.edit_action_id.as_str(),
        "ui_asset_detail_draft|ui-asset-instance|widget|widget.control_id.set|0"
    );
    assert_eq!(
        control_id_node.commit_action_id.as_str(),
        "ui_asset_detail|ui-asset-instance|widget|widget.control_id.set|0"
    );
    let value_node = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "UiAssetPropStateValuepropvariant")
        .expect("prop/state row should append editable value input");
    assert_eq!(value_node.role.as_str(), "InputField");
    assert_eq!(value_node.component_role.as_str(), "input-field");
    assert_eq!(value_node.dispatch_kind.as_str(), "commit_only");
    assert_eq!(value_node.value_text.as_str(), "\"outlined\"");
    assert_eq!(
        value_node.edit_action_id.as_str(),
        "ui_asset_detail_draft|ui-asset-instance|widget|widget.prop.variant.set|1"
    );
    assert_eq!(
        value_node.commit_action_id.as_str(),
        "ui_asset_detail|ui-asset-instance|widget|widget.prop.variant.set|1"
    );
    let label_node = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "UiAssetPropStateLabelpropvariant")
        .expect("prop/state row should append label");
    assert_eq!(label_node.text.as_str(), "prop variant");

    let state_value_node = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "UiAssetPropStateValuestateexpanded")
        .expect("state row should append editable value input");
    let promote_node = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "InspectorPromoteSection")
        .expect("promote section should remain in converted nodes");
    assert!(
        promote_node.frame.y >= state_value_node.frame.y + state_value_node.frame.height,
        "dynamic prop/state fields should grow the widget section instead of overlapping following sections"
    );

    let slot_padding_node = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "UiAssetSlotFieldPaddingValue")
        .expect("slot section should append editable padding input");
    assert_eq!(slot_padding_node.dispatch_kind.as_str(), "commit_only");
    assert_eq!(
        slot_padding_node.edit_action_id.as_str(),
        "ui_asset_detail_draft|ui-asset-instance|slot|slot.padding.set|1"
    );
    assert_eq!(
        slot_padding_node.commit_action_id.as_str(),
        "ui_asset_detail|ui-asset-instance|slot|slot.padding.set|1"
    );
    let layout_width_node = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "UiAssetLayoutFieldWidthPreferredValue")
        .expect("layout section should append editable width input");
    assert_eq!(layout_width_node.dispatch_kind.as_str(), "commit_only");
    assert_eq!(
        layout_width_node.edit_action_id.as_str(),
        "ui_asset_detail_draft|ui-asset-instance|layout|layout.width.preferred.set|0"
    );
    assert_eq!(
        layout_width_node.commit_action_id.as_str(),
        "ui_asset_detail|ui-asset-instance|layout|layout.width.preferred.set|0"
    );
    let binding_id_node = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "UiAssetBindingFieldIdValue")
        .expect("binding section should append editable id input");
    assert_eq!(binding_id_node.dispatch_kind.as_str(), "commit_only");
    assert_eq!(
        binding_id_node.edit_action_id.as_str(),
        "ui_asset_detail_draft|ui-asset-instance|binding|binding.id.set|0"
    );
    assert_eq!(
        binding_id_node.commit_action_id.as_str(),
        "ui_asset_detail|ui-asset-instance|binding|binding.id.set|0"
    );
}
