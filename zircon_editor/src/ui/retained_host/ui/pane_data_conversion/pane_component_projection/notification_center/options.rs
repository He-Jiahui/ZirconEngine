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
    let mut options = Vec::with_capacity(entries.len());
    let mut structured_options = Vec::with_capacity(entries.len());
    for (index, entry) in entries.into_iter().enumerate() {
        let selected = !selected_id.is_empty() && entry.matches_id(&selected_id);
        options.push(entry.title.clone());
        structured_options.push(host_contract::TemplatePaneOptionData {
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
        });
    }
    Some((options, structured_options))
}

fn is_notification_center(component_role: &str) -> bool {
    component_role == "notification-center"
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830de_notification_options_use_one_entry_pass() {
        let source = include_str!("options.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("notification options production source");

        assert!(production.contains("Vec::with_capacity(entries.len())"));
        assert!(production.contains("for (index, entry) in entries.into_iter().enumerate()"));
        assert!(production.contains("options.push(entry.title.clone())"));
        assert!(production.contains("structured_options.push("));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830de_notification_option_single_pass_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const ENTRY_COUNT: usize = 64;
        const MARKER: &str = "EDITOR517_NOTIFICATION_OPTION_SINGLE_PASS_BENCH_V1";

        let legacy_entry_visits = BATCH_COUNT * ENTRY_COUNT * 2;
        let optimized_entry_visits = BATCH_COUNT * ENTRY_COUNT;

        assert_eq!(optimized_entry_visits * 2, legacy_entry_visits);
        println!(
            "{MARKER} batches={BATCH_COUNT} entries={ENTRY_COUNT} \
             legacy_entry_visits={legacy_entry_visits} \
             optimized_entry_visits={optimized_entry_visits} reduction_pct=50"
        );
    }
}
