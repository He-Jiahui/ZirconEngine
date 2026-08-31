use crate::ui::asset_editor;

use super::row_model::{push_detail_row, UiAssetDetailFieldRow};

const PROP_STATE_ROW_LIMIT: usize = 6;

pub(super) fn widget_detail_rows(
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

fn sanitized_prop_state_control_suffix(
    row: &asset_editor::UiAssetEditorWidgetPropStateItem,
    row_index: usize,
) -> String {
    let mut suffix = String::with_capacity(row.kind.len() + row.path.len());
    append_sanitized_control_suffix(&mut suffix, &row.kind);
    append_sanitized_control_suffix(&mut suffix, &row.path);
    if suffix.is_empty() {
        suffix = row_index.to_string();
    }
    suffix
}

fn append_sanitized_control_suffix(output: &mut String, value: &str) {
    if value.is_ascii() {
        output.extend(value.bytes().map(|byte| {
            if byte.is_ascii_alphanumeric() {
                char::from(byte)
            } else {
                '_'
            }
        }));
    } else {
        output.extend(value.chars().map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        }));
    }
}

#[cfg(test)]
#[path = "widget/suffix_single_allocation_tests.rs"]
mod suffix_single_allocation_tests;
