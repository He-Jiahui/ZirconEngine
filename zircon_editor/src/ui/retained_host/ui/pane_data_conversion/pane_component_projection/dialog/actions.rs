use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;

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
    let Some(label) = first_non_empty_attribute(
        attributes,
        &[
            "action",
            "primary_action_text",
            "confirm_text",
            "close_text",
        ],
    ) else {
        return Vec::new();
    };
    let action_id = first_non_empty_attribute(
        attributes,
        &["dialog_action_id", "action_id", "commit_action_id"],
    )
    .unwrap_or_default();
    vec![host_contract::TemplatePaneActionData {
        label: label.into(),
        action_id: action_id.into(),
    }]
}

fn confirm_dialog_actions(
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<host_contract::TemplatePaneActionData> {
    let cancel_label =
        first_non_empty_attribute(attributes, &["cancel_text", "cancelText", "close_text"])
            .unwrap_or("Cancel");
    let confirm_label = first_non_empty_attribute(
        attributes,
        &[
            "confirm_text",
            "confirmText",
            "primary_action_text",
            "action",
        ],
    )
    .unwrap_or("Confirm");

    vec![
        host_contract::TemplatePaneActionData {
            label: cancel_label.into(),
            action_id: first_non_empty_attribute(
                attributes,
                &["cancel_action_id", "cancelActionId"],
            )
            .unwrap_or("cancel")
            .into(),
        },
        host_contract::TemplatePaneActionData {
            label: confirm_label.into(),
            action_id: first_non_empty_attribute(
                attributes,
                &["confirm_action_id", "confirmActionId", "dialog_action_id"],
            )
            .unwrap_or("confirm")
            .into(),
        },
    ]
}

fn first_non_empty_attribute<'a>(
    attributes: &'a BTreeMap<String, toml::Value>,
    names: &[&str],
) -> Option<&'a str> {
    names
        .iter()
        .filter_map(|name| attributes.get(*name))
        .filter_map(toml::Value::as_str)
        .find(|value| !value.is_empty())
}

#[cfg(test)]
#[path = "tests/actions.rs"]
mod tests;
