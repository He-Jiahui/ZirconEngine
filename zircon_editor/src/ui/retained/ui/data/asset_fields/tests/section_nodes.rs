use super::*;
use crate::ui::retained_host::ui::pane_data_conversion::ui_asset_detail_fields::row_model::UiAssetDetailFieldRow;
use zircon_runtime_interface::ui::design_tokens::EditorDensityTokens;

#[test]
fn detail_rows_use_workbench_body_typography_with_sufficient_height() {
    let mut nodes = vec![host_contract::TemplatePaneNodeData {
        control_id: "InspectorSection".into(),
        frame: host_contract::TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 20.0,
        },
        ..host_contract::TemplatePaneNodeData::default()
    }];
    let section = UiAssetDetailFieldSection {
        section_control_id: "InspectorSection",
        detail_id: "inspector",
        rows: vec![UiAssetDetailFieldRow {
            label: "Name".to_string(),
            value: "Player".to_string(),
            action_id: "rename".to_string(),
            label_control_id: "InspectorNameLabel".to_string(),
            value_control_id: "InspectorNameValue".to_string(),
            disabled: false,
        }],
    };

    append_detail_section_nodes(&mut nodes, &section, "asset-1");

    let projected = &nodes[1..];
    let minimum_line_height = EditorTypographyTokens::WORKBENCH_BODY_SIZE
        * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
    assert_eq!(projected.len(), 2);
    assert!(projected.iter().all(|node| {
        node.font_size == EditorTypographyTokens::WORKBENCH_BODY_SIZE
            && node.frame.height >= minimum_line_height
            && node.frame.height == EditorDensityTokens::WORKBENCH_ROW_HEIGHT
    }));
}
