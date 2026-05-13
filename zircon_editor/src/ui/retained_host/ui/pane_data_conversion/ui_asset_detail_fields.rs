use crate::ui::asset_editor;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;

use super::super::template_node_conversion::to_host_contract_template_node_owned;

const UI_ASSET_DETAIL_BINDING_PREFIX: &str = "ui_asset_detail";
const PROP_STATE_ROW_LIMIT: usize = 6;
const DETAIL_ROW_HEIGHT: f32 = 22.0;
const DETAIL_ROW_TOP_INSET: f32 = 18.0;
const DETAIL_ROW_X_INSET: f32 = 8.0;
const DETAIL_LABEL_GAP: f32 = 6.0;

pub(super) fn to_host_contract_ui_asset_template_nodes(
    items: Vec<ViewTemplateNodeData>,
    data: &asset_editor::UiAssetEditorPanePresentation,
    prop_state_rows: &[asset_editor::UiAssetEditorWidgetPropStateItem],
    instance_id: &str,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    let mut nodes = items
        .into_iter()
        .map(to_host_contract_template_node_owned)
        .collect::<Vec<_>>();
    for section in ui_asset_detail_field_sections(data, prop_state_rows) {
        append_detail_section_nodes(&mut nodes, &section, instance_id);
    }
    model_rc(nodes)
}

struct UiAssetDetailFieldSection {
    section_control_id: &'static str,
    detail_id: &'static str,
    rows: Vec<UiAssetDetailFieldRow>,
}

struct UiAssetDetailFieldRow {
    label: String,
    value: String,
    action_id: String,
    label_control_id: String,
    value_control_id: String,
    disabled: bool,
}

fn ui_asset_detail_field_sections(
    data: &asset_editor::UiAssetEditorPanePresentation,
    prop_state_rows: &[asset_editor::UiAssetEditorWidgetPropStateItem],
) -> Vec<UiAssetDetailFieldSection> {
    vec![
        UiAssetDetailFieldSection {
            section_control_id: "InspectorWidgetSection",
            detail_id: "widget",
            rows: widget_detail_rows(data, prop_state_rows),
        },
        UiAssetDetailFieldSection {
            section_control_id: "InspectorSlotSection",
            detail_id: "slot",
            rows: slot_detail_rows(data),
        },
        UiAssetDetailFieldSection {
            section_control_id: "InspectorLayoutSection",
            detail_id: "layout",
            rows: layout_detail_rows(data),
        },
        UiAssetDetailFieldSection {
            section_control_id: "InspectorBindingSection",
            detail_id: "binding",
            rows: binding_detail_rows(data),
        },
    ]
}

fn widget_detail_rows(
    data: &asset_editor::UiAssetEditorPanePresentation,
    prop_state_rows: &[asset_editor::UiAssetEditorWidgetPropStateItem],
) -> Vec<UiAssetDetailFieldRow> {
    let mut rows = Vec::new();
    push_detail_row(
        &mut rows,
        "Control ID",
        &data.inspector_control_id,
        "widget.control_id.set",
        "UiAssetWidgetFieldControlId",
        !data.inspector_can_edit_control_id,
        data.inspector_can_edit_control_id,
    );
    push_detail_row(
        &mut rows,
        "Text",
        &data.inspector_text_prop,
        "widget.text.set",
        "UiAssetWidgetFieldText",
        !data.inspector_can_edit_text_prop,
        data.inspector_can_edit_text_prop,
    );
    push_detail_row(
        &mut rows,
        "Root class policy",
        &data.inspector_component_root_class_policy,
        "component.root_class_policy.set",
        "UiAssetWidgetFieldRootClassPolicy",
        !data.inspector_can_edit_component_root_class_policy,
        data.inspector_can_edit_component_root_class_policy,
    );

    for (row_index, row) in prop_state_rows
        .iter()
        .take(PROP_STATE_ROW_LIMIT)
        .enumerate()
    {
        let Some(action_id) = prop_state_row_action_id(row) else {
            continue;
        };
        let control_suffix = sanitized_prop_state_control_suffix(row, row_index);
        rows.push(UiAssetDetailFieldRow {
            label: format!("{} {}", row.kind, row.path),
            value: row.value.clone(),
            action_id,
            label_control_id: format!("UiAssetPropStateLabel{control_suffix}"),
            value_control_id: format!("UiAssetPropStateValue{control_suffix}"),
            disabled: false,
        });
    }
    rows
}

fn slot_detail_rows(
    data: &asset_editor::UiAssetEditorPanePresentation,
) -> Vec<UiAssetDetailFieldRow> {
    let mut rows = Vec::new();
    push_detail_row(
        &mut rows,
        "Mount",
        &data.inspector_mount,
        "slot.mount.set",
        "UiAssetSlotFieldMount",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Padding",
        &data.inspector_slot_padding,
        "slot.padding.set",
        "UiAssetSlotFieldPadding",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Width preferred",
        &data.inspector_slot_width_preferred,
        "slot.layout.width.preferred.set",
        "UiAssetSlotFieldWidthPreferred",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Height preferred",
        &data.inspector_slot_height_preferred,
        "slot.layout.height.preferred.set",
        "UiAssetSlotFieldHeightPreferred",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        semantic_label("Semantic", &data.inspector_slot_semantic_path).as_str(),
        &data.inspector_slot_semantic_value,
        "slot.semantic.value.set",
        "UiAssetSlotFieldSemanticValue",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Linear width weight",
        &data.inspector_slot_linear_main_weight,
        "slot.linear.width_weight.set",
        "UiAssetSlotFieldLinearWidthWeight",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Linear width stretch",
        &data.inspector_slot_linear_main_stretch,
        "slot.linear.width_stretch.set",
        "UiAssetSlotFieldLinearWidthStretch",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Linear height weight",
        &data.inspector_slot_linear_cross_weight,
        "slot.linear.height_weight.set",
        "UiAssetSlotFieldLinearHeightWeight",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Linear height stretch",
        &data.inspector_slot_linear_cross_stretch,
        "slot.linear.height_stretch.set",
        "UiAssetSlotFieldLinearHeightStretch",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Overlay anchor x",
        &data.inspector_slot_overlay_anchor_x,
        "slot.overlay.anchor_x.set",
        "UiAssetSlotFieldOverlayAnchorX",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Overlay anchor y",
        &data.inspector_slot_overlay_anchor_y,
        "slot.overlay.anchor_y.set",
        "UiAssetSlotFieldOverlayAnchorY",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Overlay position x",
        &data.inspector_slot_overlay_position_x,
        "slot.overlay.position_x.set",
        "UiAssetSlotFieldOverlayPositionX",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Overlay position y",
        &data.inspector_slot_overlay_position_y,
        "slot.overlay.position_y.set",
        "UiAssetSlotFieldOverlayPositionY",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Overlay z",
        &data.inspector_slot_overlay_z_index,
        "slot.overlay.z_index.set",
        "UiAssetSlotFieldOverlayZ",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Grid row",
        &data.inspector_slot_grid_row,
        "slot.grid.row.set",
        "UiAssetSlotFieldGridRow",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Grid column",
        &data.inspector_slot_grid_column,
        "slot.grid.column.set",
        "UiAssetSlotFieldGridColumn",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Flow break before",
        &data.inspector_slot_flow_break_before,
        "slot.flow.break_before.set",
        "UiAssetSlotFieldFlowBreakBefore",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Flow alignment",
        &data.inspector_slot_flow_alignment,
        "slot.flow.alignment.set",
        "UiAssetSlotFieldFlowAlignment",
        false,
        false,
    );
    rows
}

fn layout_detail_rows(
    data: &asset_editor::UiAssetEditorPanePresentation,
) -> Vec<UiAssetDetailFieldRow> {
    let mut rows = Vec::new();
    push_detail_row(
        &mut rows,
        "Width preferred",
        &data.inspector_layout_width_preferred,
        "layout.width.preferred.set",
        "UiAssetLayoutFieldWidthPreferred",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Height preferred",
        &data.inspector_layout_height_preferred,
        "layout.height.preferred.set",
        "UiAssetLayoutFieldHeightPreferred",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        semantic_label("Semantic", &data.inspector_layout_semantic_path).as_str(),
        &data.inspector_layout_semantic_value,
        "layout.semantic.value.set",
        "UiAssetLayoutFieldSemanticValue",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Box gap",
        &data.inspector_layout_box_gap,
        "layout.box.gap.set",
        "UiAssetLayoutFieldBoxGap",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Scroll axis",
        &data.inspector_layout_scroll_axis,
        "layout.scroll.axis.set",
        "UiAssetLayoutFieldScrollAxis",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Scroll gap",
        &data.inspector_layout_scroll_gap,
        "layout.scroll.gap.set",
        "UiAssetLayoutFieldScrollGap",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Scrollbar",
        &data.inspector_layout_scrollbar_visibility,
        "layout.scroll.scrollbar_visibility.set",
        "UiAssetLayoutFieldScrollbarVisibility",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Virtual item extent",
        &data.inspector_layout_virtualization_item_extent,
        "layout.scroll.virtualization.item_extent.set",
        "UiAssetLayoutFieldVirtualItemExtent",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Virtual overscan",
        &data.inspector_layout_virtualization_overscan,
        "layout.scroll.virtualization.overscan.set",
        "UiAssetLayoutFieldVirtualOverscan",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Clip",
        &data.inspector_layout_clip,
        "layout.scroll.clip.set",
        "UiAssetLayoutFieldClip",
        false,
        false,
    );
    rows
}

fn binding_detail_rows(
    data: &asset_editor::UiAssetEditorPanePresentation,
) -> Vec<UiAssetDetailFieldRow> {
    let mut rows = Vec::new();
    push_detail_row(
        &mut rows,
        "Binding ID",
        &data.inspector_binding_id,
        "binding.id.set",
        "UiAssetBindingFieldId",
        !data.inspector_can_edit_binding,
        data.inspector_can_edit_binding,
    );
    push_detail_row(
        &mut rows,
        "Event",
        &data.inspector_binding_event,
        "binding.event.set",
        "UiAssetBindingFieldEvent",
        !data.inspector_can_edit_binding,
        data.inspector_can_edit_binding,
    );
    push_detail_row(
        &mut rows,
        "Route",
        &data.inspector_binding_route,
        "binding.route.set",
        "UiAssetBindingFieldRoute",
        !data.inspector_can_edit_binding,
        data.inspector_can_edit_binding,
    );
    push_detail_row(
        &mut rows,
        "Route target",
        &data.inspector_binding_route_target,
        "binding.route_target.set",
        "UiAssetBindingFieldRouteTarget",
        !data.inspector_can_edit_binding,
        data.inspector_can_edit_binding,
    );
    push_detail_row(
        &mut rows,
        "Action target",
        &data.inspector_binding_action_target,
        "binding.action_target.set",
        "UiAssetBindingFieldActionTarget",
        !data.inspector_can_edit_binding,
        data.inspector_can_edit_binding,
    );
    rows
}

fn push_detail_row(
    rows: &mut Vec<UiAssetDetailFieldRow>,
    label: &str,
    value: &str,
    action_id: &str,
    control_id_prefix: &str,
    disabled: bool,
    force_visible: bool,
) {
    if !force_visible && value.is_empty() {
        return;
    }
    rows.push(UiAssetDetailFieldRow {
        label: label.to_string(),
        value: value.to_string(),
        action_id: action_id.to_string(),
        label_control_id: format!("{control_id_prefix}Label"),
        value_control_id: format!("{control_id_prefix}Value"),
        disabled,
    });
}

fn append_detail_section_nodes(
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

    let required_section_height =
        DETAIL_ROW_TOP_INSET + section.rows.len() as f32 * DETAIL_ROW_HEIGHT + 4.0;
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
        let y =
            source_section.frame.y + DETAIL_ROW_TOP_INSET + row_index as f32 * DETAIL_ROW_HEIGHT;
        nodes.push(host_contract::TemplatePaneNodeData {
            node_id: format!("{}/detail/{row_index}/label", section.detail_id).into(),
            control_id: row.label_control_id.clone().into(),
            role: "Label".into(),
            text: row.label.clone().into(),
            surface_variant: "transparent".into(),
            text_tone: if row.disabled { "disabled" } else { "muted" }.into(),
            font_size: 11.0,
            text_align: "left".into(),
            frame: host_contract::TemplateNodeFrameData {
                x: content_x,
                y,
                width: label_width,
                height: DETAIL_ROW_HEIGHT,
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
            font_size: 11.0,
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
                height: DETAIL_ROW_HEIGHT,
            },
            ..host_contract::TemplatePaneNodeData::default()
        });
    }
}

fn prop_state_row_action_id(
    row: &asset_editor::UiAssetEditorWidgetPropStateItem,
) -> Option<String> {
    if row.path.is_empty() {
        return None;
    }
    match row.kind.as_str() {
        "prop" | "state" => Some(format!("widget.{}.{}.set", row.kind, row.path)),
        _ => None,
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

fn sanitized_prop_state_control_suffix(
    row: &asset_editor::UiAssetEditorWidgetPropStateItem,
    row_index: usize,
) -> String {
    let mut suffix = format!("{}{}", row.kind, row.path)
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if suffix.is_empty() {
        suffix = row_index.to_string();
    }
    suffix
}

fn semantic_label(prefix: &str, path: &str) -> String {
    if path.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix} {path}")
    }
}
