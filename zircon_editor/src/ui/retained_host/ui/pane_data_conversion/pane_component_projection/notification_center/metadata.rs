use std::collections::BTreeMap;

use toml::Value;

use crate::ui::template_runtime::RetainedUiHostValue;

use super::attributes::{string_attribute, usize_attribute};

const NOTIFICATION_GENERATION: &str = "notification_generation";
const UNREAD_COUNT: &str = "unread_count";
const OVERFLOW_COUNT: &str = "overflow_count";
const SELECTED_NOTIFICATION_ID: &str = "selected_notification_id";
const FOCUSED_INDEX: &str = "focused_index";
const VISIBLE_LIMIT: &str = "visible_limit";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::ui) struct NotificationCenterMetadata {
    pub generation: usize,
    pub unread_count: usize,
    pub overflow_count: usize,
    pub selected_id: String,
    pub focused_index: Option<usize>,
    pub visible_limit: usize,
}

impl Default for NotificationCenterMetadata {
    fn default() -> Self {
        Self {
            generation: 0,
            unread_count: 0,
            overflow_count: 0,
            selected_id: String::new(),
            focused_index: None,
            visible_limit: usize::MAX,
        }
    }
}

pub(in crate::ui::retained_host::ui) fn projected_notification_center_metadata(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> Option<NotificationCenterMetadata> {
    if component_role != "notification-center" {
        return None;
    }

    Some(NotificationCenterMetadata {
        generation: usize_attribute(attributes.get(NOTIFICATION_GENERATION)).unwrap_or(0),
        unread_count: usize_attribute(attributes.get(UNREAD_COUNT)).unwrap_or(0),
        overflow_count: usize_attribute(attributes.get(OVERFLOW_COUNT)).unwrap_or(0),
        selected_id: string_attribute(attributes, SELECTED_NOTIFICATION_ID).unwrap_or_default(),
        focused_index: usize_attribute(attributes.get(FOCUSED_INDEX)),
        visible_limit: usize_attribute(attributes.get(VISIBLE_LIMIT)).unwrap_or(usize::MAX),
    })
}

pub(in crate::ui::retained_host::ui) fn projected_notification_center_metadata_from_host(
    component_role: &str,
    attributes: &BTreeMap<String, RetainedUiHostValue>,
) -> Option<NotificationCenterMetadata> {
    if component_role != "notification-center" {
        return None;
    }

    Some(NotificationCenterMetadata {
        generation: host_usize(attributes.get(NOTIFICATION_GENERATION)).unwrap_or(0),
        unread_count: host_usize(attributes.get(UNREAD_COUNT)).unwrap_or(0),
        overflow_count: host_usize(attributes.get(OVERFLOW_COUNT)).unwrap_or(0),
        selected_id: host_string(attributes.get(SELECTED_NOTIFICATION_ID)).unwrap_or_default(),
        focused_index: host_usize(attributes.get(FOCUSED_INDEX)),
        visible_limit: host_usize(attributes.get(VISIBLE_LIMIT)).unwrap_or(usize::MAX),
    })
}

fn host_usize(value: Option<&RetainedUiHostValue>) -> Option<usize> {
    match value? {
        RetainedUiHostValue::Integer(value) => (*value >= 0).then_some(*value as usize),
        RetainedUiHostValue::Float(value) if value.is_finite() && *value >= 0.0 => {
            Some(*value as usize)
        }
        _ => None,
    }
}

fn host_string(value: Option<&RetainedUiHostValue>) -> Option<String> {
    match value? {
        RetainedUiHostValue::String(value) => {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_string())
        }
        _ => None,
    }
}
