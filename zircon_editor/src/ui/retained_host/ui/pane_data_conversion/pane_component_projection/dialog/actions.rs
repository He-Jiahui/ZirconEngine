use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;

use super::super::attribute_values::first_non_empty_string_attribute;

pub(in super::super) fn projected_dialog_actions(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<host_contract::TemplatePaneActionData> {
    match component_role {
        "dialog" => dialog_actions(attributes),
        "confirm-dialog" | "alert-dialog" => confirm_dialog_actions(attributes),
        _ => Vec::new(),
    }
}

fn dialog_actions(
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<host_contract::TemplatePaneActionData> {
    first_non_empty_string_attribute(
        attributes,
        &[
            "action",
            "primary_action_text",
            "confirm_text",
            "close_text",
        ],
    )
    .map(|label| {
        vec![host_contract::TemplatePaneActionData {
            label: label.into(),
            action_id: first_non_empty_string_attribute(
                attributes,
                &["dialog_action_id", "action_id", "commit_action_id"],
            )
            .unwrap_or_default()
            .into(),
        }]
    })
    .unwrap_or_default()
}

fn confirm_dialog_actions(
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<host_contract::TemplatePaneActionData> {
    let cancel_label =
        first_non_empty_string_attribute(attributes, &["cancel_text", "cancelText", "close_text"])
            .unwrap_or_else(|| "Cancel".to_string());
    let confirm_label = first_non_empty_string_attribute(
        attributes,
        &[
            "confirm_text",
            "confirmText",
            "primary_action_text",
            "action",
        ],
    )
    .unwrap_or_else(|| "Confirm".to_string());

    vec![
        host_contract::TemplatePaneActionData {
            label: cancel_label.into(),
            action_id: first_non_empty_string_attribute(
                attributes,
                &["cancel_action_id", "cancelActionId"],
            )
            .unwrap_or_else(|| "cancel".to_string())
            .into(),
        },
        host_contract::TemplatePaneActionData {
            label: confirm_label.into(),
            action_id: first_non_empty_string_attribute(
                attributes,
                &["confirm_action_id", "confirmActionId", "dialog_action_id"],
            )
            .unwrap_or_else(|| "confirm".to_string())
            .into(),
        },
    ]
}
