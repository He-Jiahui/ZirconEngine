use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use toml::Value;

use super::attributes::{string_attribute, string_attribute_ref, usize_attribute};
use super::entries::projected_notification_entries;

const SELECTED_NOTIFICATION_ID: &str = "selected_notification_id";
const FOCUSED_INDEX: &str = "focused_index";
const EMPTY_TEXT: &str = "empty_text";

pub(in crate::ui::retained_host::ui) fn projected_notification_center_value_text(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<String> {
    if !is_notification_center(component_role) {
        return None;
    }
    string_attribute(attributes, EMPTY_TEXT)
}

pub(in crate::ui::retained_host::ui) fn projected_notification_center_options(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<Vec<String>> {
    projected_notification_center_option_rows(component_role, attributes)
        .map(|(options, _)| options)
}

pub(in crate::ui::retained_host::ui) fn projected_notification_center_structured_options(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<Vec<host_contract::TemplatePaneOptionData>> {
    projected_notification_center_option_rows(component_role, attributes)
        .map(|(_, structured_options)| structured_options)
}

pub(in crate::ui::retained_host::ui) fn projected_notification_center_option_rows(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<(Vec<String>, Vec<host_contract::TemplatePaneOptionData>)> {
    if !is_notification_center(component_role) {
        return None;
    }

    let selected_id =
        string_attribute_ref(attributes, SELECTED_NOTIFICATION_ID).unwrap_or_default();
    let focused_index = usize_attribute(attributes.get(FOCUSED_INDEX));
    let entries = projected_notification_entries(attributes);
    let options = entries.iter().map(|entry| entry.title.clone()).collect();
    let structured_options = entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let selected = !selected_id.is_empty() && entry.matches_id(&selected_id);
            host_contract::TemplatePaneOptionData {
                id: entry.id.into(),
                label: entry.title.into(),
                description: entry.message.into(),
                tone: entry.tone.into(),
                selected,
                disabled: entry.disabled,
                special: entry.unread,
                unread: entry.unread,
                focused: focused_index == Some(index),
                ..host_contract::TemplatePaneOptionData::default()
            }
        })
        .collect();
    Some((options, structured_options))
}

fn is_notification_center(component_role: &str) -> bool {
    component_role == "notification-center"
}
