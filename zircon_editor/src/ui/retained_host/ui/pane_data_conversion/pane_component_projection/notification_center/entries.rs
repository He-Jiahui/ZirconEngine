use std::collections::BTreeMap;

use toml::Value;

use super::attributes::usize_attribute;
use super::entry::NotificationProjectionEntry;
use super::parse::notification_entry_list;

const NOTIFICATIONS: &str = "notifications";
const VISIBLE_LIMIT: &str = "visible_limit";

pub(super) fn projected_notification_entries(
    attributes: &BTreeMap<String, Value>,
) -> Vec<NotificationProjectionEntry> {
    let visible_limit = attributes
        .get(VISIBLE_LIMIT)
        .and_then(|value| usize_attribute(Some(value)))
        .unwrap_or(usize::MAX);
    attributes
        .get(NOTIFICATIONS)
        .map(notification_entry_list)
        .unwrap_or_default()
        .into_iter()
        .take(visible_limit)
        .collect()
}
