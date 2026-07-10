use zircon_runtime_interface::ui::design_tokens::{EditorDensityTokens, EditorTypographyTokens};

use crate::ui::retained_host as host_contract;

use super::row_model::UiAssetDetailFieldSection;

const UI_ASSET_DETAIL_BINDING_PREFIX: &str = "ui_asset_detail";
const DETAIL_ROW_TOP_INSET: f32 = 18.0;
const DETAIL_ROW_X_INSET: f32 = 8.0;
const DETAIL_LABEL_GAP: f32 = 6.0;

pub(super) fn append_detail_section_nodes(
    nodes: &mut Vec<host_contract::TemplatePaneNodeData>,
    section: &UiAssetDetailFieldSection,
    instance_id: &str,
) {
    let Some(section_index) = nodes
        .iter()
        .position(|node| node.control_id.as_str() == section.section_control_id)
    else {
        return;
    };
    if section.rows.is_empty() {
        return;
    }
    let mut source_section = nodes[section_index].clone();
    if source_section.frame.width <= DETAIL_ROW_X_INSET * 2.0 {
        return;
    }

    let required_section_height = DETAIL_ROW_TOP_INSET
        + section.rows.len() as f32 * EditorDensityTokens::WORKBENCH_ROW_HEIGHT
        + 4.0;
    let section_growth = (required_section_height - source_section.frame.height).max(0.0);
    if section_growth > 0.0 {
        let original_section_bottom = source_section.frame.y + source_section.frame.height;
        nodes[section_index].frame.height += section_growth;
        for (node_index, node) in nodes.iter_mut().enumerate() {
            if node_index != section_index && node.frame.y >= original_section_bottom - 0.5 {
                node.frame.y += section_growth;
            }
        }
        source_section = nodes[section_index].clone();
    }

    let content_x = source_section.frame.x + DETAIL_ROW_X_INSET;
    let content_width = (source_section.frame.width - DETAIL_ROW_X_INSET * 2.0).max(0.0);
    let label_width = (content_width * 0.42).clamp(72.0, 132.0);
    let value_x = content_x + label_width + DETAIL_LABEL_GAP;
    let value_width = (content_width - label_width - DETAIL_LABEL_GAP).max(48.0);

    for (row_index, row) in section.rows.iter().enumerate() {
        let y = source_section.frame.y
            + DETAIL_ROW_TOP_INSET
            + row_index as f32 * EditorDensityTokens::WORKBENCH_ROW_HEIGHT;
        nodes.push(host_contract::TemplatePaneNodeData {
            node_id: format!("{}/detail/{row_index}/label", section.detail_id).into(),
            control_id: row.label_control_id.clone().into(),
            role: "Label".into(),
            text: row.label.clone().into(),
            surface_variant: "transparent".into(),
            text_tone: if row.disabled { "disabled" } else { "muted" }.into(),
            font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
            text_align: "left".into(),
            frame: host_contract::TemplateNodeFrameData {
                x: content_x,
                y,
                width: label_width,
                height: EditorDensityTokens::WORKBENCH_ROW_HEIGHT,
            },
            ..host_contract::TemplatePaneNodeData::default()
        });
        nodes.push(host_contract::TemplatePaneNodeData {
            node_id: format!("{}/detail/{row_index}/value", section.detail_id).into(),
            control_id: row.value_control_id.clone().into(),
            role: "InputField".into(),
            component_role: "input-field".into(),
            dispatch_kind: "commit_only".into(),
            value_text: row.value.clone().into(),
            surface_variant: "inset".into(),
            text_tone: if row.disabled { "disabled" } else { "default" }.into(),
            font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
            text_align: "left".into(),
            corner_radius: 4.0,
            border_width: 1.0,
            disabled: row.disabled,
            edit_action_id: (!row.disabled)
                .then(|| {
                    ui_asset_detail_draft_binding_id(
                        instance_id,
                        section.detail_id,
                        &row.action_id,
                        row_index as i32,
                    )
                })
                .unwrap_or_default()
                .into(),
            commit_action_id: (!row.disabled)
                .then(|| {
                    ui_asset_detail_binding_id(
                        instance_id,
                        section.detail_id,
                        &row.action_id,
                        row_index as i32,
                    )
                })
                .unwrap_or_default()
                .into(),
            frame: host_contract::TemplateNodeFrameData {
                x: value_x,
                y,
                width: value_width,
                height: EditorDensityTokens::WORKBENCH_ROW_HEIGHT,
            },
            ..host_contract::TemplatePaneNodeData::default()
        });
    }
}

fn ui_asset_detail_binding_id(
    instance_id: &str,
    detail_id: &str,
    action_id: &str,
    item_index: i32,
) -> String {
    format!("{UI_ASSET_DETAIL_BINDING_PREFIX}|{instance_id}|{detail_id}|{action_id}|{item_index}")
}

fn ui_asset_detail_draft_binding_id(
    instance_id: &str,
    detail_id: &str,
    action_id: &str,
    item_index: i32,
) -> String {
    format!("ui_asset_detail_draft|{instance_id}|{detail_id}|{action_id}|{item_index}")
}

#[cfg(test)]
mod tests {
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
}
