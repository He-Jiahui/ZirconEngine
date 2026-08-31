use std::collections::BTreeMap;

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentKeyboardAction, UiComponentState,
    UiValidationState, UiValue,
};

const NOTIFICATIONS: &str = "notifications";
const UNREAD_COUNT: &str = "unread_count";
const FOCUSED_INDEX: &str = "focused_index";
const SELECTED_NOTIFICATION_ID: &str = "selected_notification_id";
const VISIBLE_LIMIT: &str = "visible_limit";
const KEYBOARD_NAVIGATION: &str = "keyboard_navigation";

pub(super) fn sync_after_value_change(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    changed_property: &str,
) -> Result<bool, UiComponentEventError> {
    if !is_notification_center(descriptor) {
        return Ok(false);
    }

    if changed_property == SELECTED_NOTIFICATION_ID {
        let selected_id =
            string_setting(state, descriptor, SELECTED_NOTIFICATION_ID).unwrap_or_default();
        if !selected_id.is_empty() {
            select_notification(state, descriptor, &selected_id, true)?;
            return Ok(true);
        }
    }

    sync_notification_state(state, descriptor);
    Ok(true)
}

pub(super) fn apply_selection(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    _property: &str,
    option_id: &str,
    selected: bool,
) -> Result<bool, UiComponentEventError> {
    if !is_notification_center(descriptor) {
        return Ok(false);
    }

    if selected {
        select_notification(state, descriptor, option_id, true)?;
    } else if string_setting(state, descriptor, SELECTED_NOTIFICATION_ID)
        .as_deref()
        .map(|current| current == option_id)
        .unwrap_or(true)
    {
        write_selected_notification(state, "", -1);
        state.flags.selected = false;
        sync_notification_state(state, descriptor);
    }
    Ok(true)
}

pub(super) fn apply_keyboard_action(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    action: UiComponentKeyboardAction,
) -> Result<bool, UiComponentEventError> {
    if !is_notification_center(descriptor) {
        return Ok(false);
    }
    if !bool_setting(state, descriptor, KEYBOARD_NAVIGATION, true) {
        sync_notification_state(state, descriptor);
        return Ok(true);
    }

    match action {
        UiComponentKeyboardAction::Next
        | UiComponentKeyboardAction::Previous
        | UiComponentKeyboardAction::First
        | UiComponentKeyboardAction::Last => {
            navigate_notifications(state, descriptor, action);
            Ok(true)
        }
        UiComponentKeyboardAction::Activate => {
            if let Some(notification_id) = focused_notification_id(state, descriptor) {
                select_notification(state, descriptor, &notification_id, true)?;
            } else {
                sync_notification_state(state, descriptor);
            }
            Ok(true)
        }
        UiComponentKeyboardAction::Cancel => {
            super::overlay::close_popup(state, descriptor)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn sync_notification_state(state: &mut UiComponentState, descriptor: &UiComponentDescriptor) {
    let entries = notification_entries(state, descriptor);
    write_unread_count_from_entries(state, &entries);

    let selected_id =
        string_setting(state, descriptor, SELECTED_NOTIFICATION_ID).unwrap_or_default();
    let selected_entry = (!selected_id.is_empty())
        .then(|| {
            entries
                .iter()
                .find(|entry| entry.matches_id(&selected_id) && !entry.disabled)
        })
        .flatten();

    if selected_id.is_empty() || selected_entry.is_none() {
        super::set_value(
            state,
            SELECTED_NOTIFICATION_ID.to_string(),
            UiValue::String(String::new()),
        );
        state.flags.selected = false;
    } else {
        state.flags.selected = true;
    }

    let focus_index = normalized_focus_index(state, &entries, selected_entry);
    super::set_value(state, FOCUSED_INDEX.to_string(), UiValue::Int(focus_index));
    state.flags.focused = focus_index >= 0;
}

fn select_notification(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    notification_id: &str,
    mark_read: bool,
) -> Result<(), UiComponentEventError> {
    ensure_notification_enabled(state, descriptor, notification_id)?;
    let entries = notification_entries(state, descriptor);
    let Some(entry) = entries
        .iter()
        .find(|entry| entry.matches_id(notification_id))
        .cloned()
    else {
        write_selected_notification(state, "", -1);
        state.flags.selected = false;
        sync_notification_state(state, descriptor);
        return Ok(());
    };

    write_selected_notification(state, &entry.id, entry.index);
    state.flags.focused = true;
    state.flags.selected = true;
    if mark_read {
        mark_notification_read(state, &entry.id);
    }
    write_unread_count(state, descriptor);
    Ok(())
}

fn navigate_notifications(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    action: UiComponentKeyboardAction,
) {
    let entries = visible_notification_entries(state, descriptor);
    let enabled_entries = entries
        .iter()
        .filter(|entry| !entry.disabled)
        .collect::<Vec<_>>();

    if enabled_entries.is_empty() {
        super::set_value(state, FOCUSED_INDEX.to_string(), UiValue::Int(-1));
        state.flags.focused = false;
        write_unread_count(state, descriptor);
        return;
    }

    let current = current_focus_index(state, descriptor);
    let next = match action {
        UiComponentKeyboardAction::First => enabled_entries.first().copied(),
        UiComponentKeyboardAction::Last => enabled_entries.last().copied(),
        UiComponentKeyboardAction::Next if current < 0 => enabled_entries.first().copied(),
        UiComponentKeyboardAction::Previous if current < 0 => enabled_entries.last().copied(),
        UiComponentKeyboardAction::Next => enabled_entries
            .iter()
            .copied()
            .find(|entry| entry.index > current)
            .or_else(|| {
                enabled_entries
                    .iter()
                    .copied()
                    .find(|entry| entry.index == current)
            }),
        UiComponentKeyboardAction::Previous => enabled_entries
            .iter()
            .rev()
            .copied()
            .find(|entry| entry.index < current)
            .or_else(|| {
                enabled_entries
                    .iter()
                    .copied()
                    .find(|entry| entry.index == current)
            }),
        _ => None,
    };

    if let Some(entry) = next {
        super::set_value(state, FOCUSED_INDEX.to_string(), UiValue::Int(entry.index));
        state.flags.focused = true;
    }
    write_unread_count(state, descriptor);
}

fn focused_notification_id(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Option<String> {
    let focused_index = current_focus_index(state, descriptor);
    visible_notification_entries(state, descriptor)
        .into_iter()
        .find(|entry| entry.index == focused_index && !entry.disabled)
        .map(|entry| entry.id)
}

fn ensure_notification_enabled(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    notification_id: &str,
) -> Result<(), UiComponentEventError> {
    if notification_entries(state, descriptor)
        .into_iter()
        .any(|entry| entry.matches_id(notification_id) && entry.disabled)
    {
        state.validation = UiValidationState::error(format!(
            "disabled notification `{notification_id}` cannot be selected"
        ));
        return Err(UiComponentEventError::DisabledOption {
            component_id: descriptor.id.clone(),
            option_id: notification_id.to_string(),
        });
    }
    Ok(())
}

fn write_selected_notification(
    state: &mut UiComponentState,
    notification_id: &str,
    focus_index: i64,
) {
    super::set_value(
        state,
        SELECTED_NOTIFICATION_ID.to_string(),
        UiValue::String(notification_id.to_string()),
    );
    super::set_value(state, FOCUSED_INDEX.to_string(), UiValue::Int(focus_index));
}

fn normalized_focus_index(
    state: &UiComponentState,
    entries: &[NotificationEntry],
    selected_entry: Option<&NotificationEntry>,
) -> i64 {
    if let Some(entry) = selected_entry {
        return entry.index;
    }

    if let Some(index) = int_value(state.values.get(FOCUSED_INDEX)) {
        if entries
            .iter()
            .any(|entry| entry.index == index && !entry.disabled)
        {
            return index;
        }
    }

    entries
        .iter()
        .find(|entry| !entry.disabled)
        .map(|entry| entry.index)
        .unwrap_or(-1)
}

fn mark_notification_read(state: &mut UiComponentState, notification_id: &str) {
    if let Some(value) = state.values.get_mut(NOTIFICATIONS) {
        mark_notification_value_read(value, notification_id);
    }
}

fn mark_notification_value_read(value: &mut UiValue, notification_id: &str) {
    match value {
        UiValue::Array(values) => {
            for value in values {
                mark_notification_value_read(value, notification_id);
            }
        }
        UiValue::Map(values) => {
            if notification_map_matches_id(values, notification_id) {
                values.insert("unread".to_string(), UiValue::Bool(false));
                if values.contains_key("new") {
                    values.insert("new".to_string(), UiValue::Bool(false));
                }
            }
        }
        _ => {}
    }
}

fn notification_map_matches_id(values: &BTreeMap<String, UiValue>, notification_id: &str) -> bool {
    first_string_value(
        values,
        &["id", "notification_id", "notificationId", "value", "key"],
    )
    .is_some_and(|id| id == notification_id)
}

fn write_unread_count(state: &mut UiComponentState, descriptor: &UiComponentDescriptor) {
    let entries = notification_entries(state, descriptor);
    write_unread_count_from_entries(state, &entries);
}

fn write_unread_count_from_entries(state: &mut UiComponentState, entries: &[NotificationEntry]) {
    super::set_value(
        state,
        UNREAD_COUNT.to_string(),
        UiValue::Int(entries.iter().filter(|entry| entry.unread).count() as i64),
    );
}

fn visible_notification_entries(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Vec<NotificationEntry> {
    let visible_limit = int_setting(state, descriptor, VISIBLE_LIMIT)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(usize::MAX);
    let Some(notifications) = value_setting(state, descriptor, NOTIFICATIONS) else {
        return Vec::new();
    };
    let mut entries = Vec::with_capacity(visible_limit.min(notification_root_len(notifications)));
    collect_visible_notification_entries(notifications, 0, visible_limit, &mut entries);
    entries
}

fn notification_entries(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Vec<NotificationEntry> {
    value_setting(state, descriptor, NOTIFICATIONS)
        .map(|value| notification_entry_list(value, 0))
        .unwrap_or_default()
}

fn notification_entry_list(value: &UiValue, start_index: i64) -> Vec<NotificationEntry> {
    let mut entries = Vec::with_capacity(notification_root_len(value));
    collect_visible_notification_entries(value, start_index, usize::MAX, &mut entries);
    entries
}

fn collect_visible_notification_entries(
    value: &UiValue,
    start_index: i64,
    visible_limit: usize,
    entries: &mut Vec<NotificationEntry>,
) {
    if entries.len() >= visible_limit {
        return;
    }
    match value {
        UiValue::Array(values) => {
            for (offset, value) in values.iter().enumerate() {
                collect_visible_notification_entries(
                    value,
                    start_index + offset as i64,
                    visible_limit,
                    entries,
                );
                if entries.len() >= visible_limit {
                    break;
                }
            }
        }
        UiValue::String(value) | UiValue::Enum(value) => {
            entries.extend(notification_entry_from_string(value, start_index));
        }
        UiValue::Map(values) => {
            entries.extend(notification_entry_from_map(values, start_index));
        }
        _ => {}
    }
}

fn notification_root_len(value: &UiValue) -> usize {
    match value {
        UiValue::Array(values) => values.len(),
        UiValue::String(_) | UiValue::Enum(_) | UiValue::Map(_) => 1,
        _ => 0,
    }
}

fn notification_entry_from_string(value: &str, index: i64) -> Option<NotificationEntry> {
    let mut parts = value.split('|');
    let id = parts.next()?.trim().to_string();
    if id.is_empty() {
        return None;
    }

    let mut entry = NotificationEntry {
        id,
        index,
        unread: false,
        disabled: false,
    };
    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        match key.trim() {
            "unread" | "new" => entry.unread = string_bool(value).unwrap_or(false),
            "disabled" => entry.disabled = string_bool(value).unwrap_or(false),
            "enabled" => entry.disabled = string_bool(value) == Some(false),
            _ => {}
        }
    }
    Some(entry)
}

fn notification_entry_from_map(
    values: &BTreeMap<String, UiValue>,
    index: i64,
) -> Option<NotificationEntry> {
    let id = first_string_value(
        values,
        &["id", "notification_id", "notificationId", "value", "key"],
    )?;
    if id.is_empty() {
        return None;
    }

    Some(NotificationEntry {
        id,
        index,
        unread: values
            .get("unread")
            .or_else(|| values.get("new"))
            .and_then(bool_value)
            .unwrap_or(false),
        disabled: values.get("disabled").and_then(bool_value).unwrap_or(false)
            || values.get("enabled").and_then(bool_value) == Some(false),
    })
}

fn is_notification_center(descriptor: &UiComponentDescriptor) -> bool {
    descriptor.role == "notification-center" || descriptor.id == "NotificationCenter"
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NotificationEntry {
    id: String,
    index: i64,
    unread: bool,
    disabled: bool,
}

impl NotificationEntry {
    fn matches_id(&self, id: &str) -> bool {
        !id.is_empty() && self.id == id
    }
}

fn value_setting<'a>(
    state: &'a UiComponentState,
    descriptor: &'a UiComponentDescriptor,
    property: &str,
) -> Option<&'a UiValue> {
    state.values.get(property).or_else(|| {
        descriptor
            .prop(property)
            .and_then(|schema| schema.default_value.as_ref())
    })
}

fn string_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<String> {
    value_setting(state, descriptor, property).and_then(string_value)
}

fn bool_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    default_value: bool,
) -> bool {
    value_setting(state, descriptor, property)
        .and_then(bool_value)
        .unwrap_or(default_value)
}

fn int_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<i64> {
    int_value(value_setting(state, descriptor, property))
}

fn current_focus_index(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> i64 {
    int_setting(state, descriptor, FOCUSED_INDEX).unwrap_or(-1)
}

fn first_string_value(values: &BTreeMap<String, UiValue>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| values.get(*key).and_then(string_value))
        .find(|value| !value.is_empty())
}

fn string_value(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.clone()),
        _ => None,
    }
}

fn bool_value(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        UiValue::String(value) | UiValue::Enum(value) => string_bool(value),
        _ => None,
    }
}

fn int_value(value: Option<&UiValue>) -> Option<i64> {
    match value {
        Some(UiValue::Int(value)) => Some(*value),
        Some(UiValue::Float(value)) => Some(value.round() as i64),
        _ => None,
    }
}

fn string_bool(value: &str) -> Option<bool> {
    match value.trim() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod performance_tests {
    use zircon_runtime_interface::ui::component::UiValue;

    use super::collect_visible_notification_entries;

    #[test]
    fn visible_entries_skip_invalid_values_without_changing_logical_indexes() {
        let notifications = UiValue::Array(vec![
            UiValue::String(String::new()),
            UiValue::String("first".to_string()),
            UiValue::Array(vec![
                UiValue::String("second".to_string()),
                UiValue::String("third".to_string()),
            ]),
        ]);
        let mut entries = Vec::new();

        collect_visible_notification_entries(&notifications, 0, 2, &mut entries);

        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.id.as_str(), entry.index))
                .collect::<Vec<_>>(),
            vec![("first", 1), ("second", 2)]
        );
    }
}
