use std::collections::BTreeMap;

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValue,
};

const TOAST_QUEUE: &str = "toast_queue";
const QUEUE: &str = "queue";
const CURRENT_TOAST_ID: &str = "current_toast_id";
const EXPIRED_TOAST_ID: &str = "expired_toast_id";
const QUEUE_LENGTH: &str = "queue_length";
const MESSAGE: &str = "message";
const TEXT: &str = "text";
const ACTION_LABEL: &str = "action_label";
const AUTO_HIDE_DURATION_MS: &str = "auto_hide_duration_ms";
const AUTO_HIDE_DURATION_CAMEL: &str = "autoHideDuration";

pub(super) fn sync_after_value_change(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    changed_property: &str,
) -> Result<bool, UiComponentEventError> {
    if !is_toast_control(descriptor) {
        return Ok(false);
    }
    if toast_sync_property(changed_property) {
        sync_toast_state(state, descriptor)?;
    }
    Ok(true)
}

pub(super) fn apply_commit(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    value: &UiValue,
) -> Result<bool, UiComponentEventError> {
    if !is_toast_control(descriptor) {
        return Ok(false);
    }

    let Some(expired_id) = toast_id_from_commit(state, descriptor, property, value) else {
        return Ok(false);
    };
    expire_toast(state, descriptor, &expired_id)?;
    Ok(true)
}

pub(super) fn apply_open_popup(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<bool, UiComponentEventError> {
    if !is_toast_control(descriptor) {
        return Ok(false);
    }
    sync_toast_state(state, descriptor)?;
    if has_current_toast(state, descriptor) {
        super::overlay::open_popup(state, descriptor)?;
    }
    Ok(true)
}

pub(super) fn apply_close_popup(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<bool, UiComponentEventError> {
    if !is_toast_control(descriptor) {
        return Ok(false);
    }
    expire_current_toast(state, descriptor)?;
    Ok(true)
}

fn sync_toast_state(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    let entries = toast_entries(state, descriptor);
    write_queue_length(state, entries.len());

    if entries.is_empty() {
        if has_explicit_queue(state) {
            clear_current_toast(state, descriptor)?;
            return Ok(());
        }
        sync_authored_message_state(state, descriptor)?;
        return Ok(());
    }

    let current_id = string_setting(state, descriptor, CURRENT_TOAST_ID).unwrap_or_default();
    let current = entries
        .iter()
        .find(|entry| entry.matches_id(&current_id))
        .unwrap_or(&entries[0]);
    write_current_toast(state, descriptor, current)?;
    Ok(())
}

fn sync_authored_message_state(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    let message = [MESSAGE, TEXT]
        .into_iter()
        .filter_map(|property| string_setting(state, descriptor, property))
        .find(|value| !value.is_empty())
        .unwrap_or_default();
    if message.is_empty() {
        clear_current_toast(state, descriptor)?;
        return Ok(());
    }

    let current_id = string_setting(state, descriptor, CURRENT_TOAST_ID)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| message.clone());
    super::set_value(
        state,
        CURRENT_TOAST_ID.to_string(),
        UiValue::String(current_id),
    );
    super::overlay::open_popup(state, descriptor)?;
    Ok(())
}

fn expire_current_toast(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    sync_toast_state(state, descriptor)?;
    let current_id = string_setting(state, descriptor, CURRENT_TOAST_ID).unwrap_or_default();
    expire_toast(state, descriptor, &current_id)
}

fn expire_toast(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    expired_id: &str,
) -> Result<(), UiComponentEventError> {
    let current_id = string_setting(state, descriptor, CURRENT_TOAST_ID).unwrap_or_default();
    if !expired_id.is_empty() && !current_id.is_empty() && expired_id != current_id {
        sync_toast_state(state, descriptor)?;
        return Ok(());
    }

    let expired_id = if expired_id.is_empty() {
        current_id
    } else {
        expired_id.to_string()
    };
    if expired_id.is_empty() {
        clear_current_toast(state, descriptor)?;
        return Ok(());
    }

    let entries = toast_entries(state, descriptor);
    if entries.is_empty() {
        write_expired_toast(state, &expired_id);
        clear_current_toast(state, descriptor)?;
        return Ok(());
    }

    let mut removed = false;
    let remaining = entries
        .into_iter()
        .filter_map(|entry| {
            if !removed && entry.matches_id(&expired_id) {
                removed = true;
                None
            } else {
                Some(entry.raw)
            }
        })
        .collect::<Vec<_>>();
    if !removed {
        sync_toast_state(state, descriptor)?;
        return Ok(());
    }

    write_expired_toast(state, &expired_id);
    write_toast_queue(state, remaining);
    sync_toast_state(state, descriptor)
}

fn write_current_toast(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    entry: &ToastEntry,
) -> Result<(), UiComponentEventError> {
    super::set_value(
        state,
        CURRENT_TOAST_ID.to_string(),
        UiValue::String(entry.id.clone()),
    );
    set_optional_string(state, descriptor, MESSAGE, &entry.message);
    set_optional_string(state, descriptor, TEXT, &entry.message);
    set_optional_string(state, descriptor, ACTION_LABEL, &entry.action_label);
    if let Some(duration_ms) = entry.duration_ms {
        set_optional_int(state, descriptor, AUTO_HIDE_DURATION_MS, duration_ms);
        set_optional_int(state, descriptor, AUTO_HIDE_DURATION_CAMEL, duration_ms);
    }
    super::overlay::open_popup(state, descriptor)?;
    Ok(())
}

fn clear_current_toast(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    super::set_value(
        state,
        CURRENT_TOAST_ID.to_string(),
        UiValue::String(String::new()),
    );
    set_optional_string(state, descriptor, MESSAGE, "");
    set_optional_string(state, descriptor, TEXT, "");
    set_optional_string(state, descriptor, ACTION_LABEL, "");
    super::overlay::close_popup(state, descriptor)
}

fn write_expired_toast(state: &mut UiComponentState, expired_id: &str) {
    super::set_value(
        state,
        EXPIRED_TOAST_ID.to_string(),
        UiValue::String(expired_id.to_string()),
    );
}

fn write_toast_queue(state: &mut UiComponentState, values: Vec<UiValue>) {
    super::set_value(state, TOAST_QUEUE.to_string(), UiValue::Array(values));
}

fn write_queue_length(state: &mut UiComponentState, queue_length: usize) {
    super::set_value(
        state,
        QUEUE_LENGTH.to_string(),
        UiValue::Int(queue_length as i64),
    );
}

fn toast_id_from_commit(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    value: &UiValue,
) -> Option<String> {
    match property {
        EXPIRED_TOAST_ID | "toast_timeout" | "timeout" | "auto_hide_timeout" => {
            Some(string_value(value).unwrap_or_else(|| {
                string_setting(state, descriptor, CURRENT_TOAST_ID).unwrap_or_default()
            }))
        }
        CURRENT_TOAST_ID | "toast_id" => Some(string_value(value).unwrap_or_default()),
        _ => None,
    }
}

fn has_current_toast(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> bool {
    string_setting(state, descriptor, CURRENT_TOAST_ID)
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

fn toast_sync_property(property: &str) -> bool {
    matches!(
        property,
        TOAST_QUEUE
            | QUEUE
            | CURRENT_TOAST_ID
            | EXPIRED_TOAST_ID
            | QUEUE_LENGTH
            | MESSAGE
            | TEXT
            | ACTION_LABEL
            | AUTO_HIDE_DURATION_MS
            | AUTO_HIDE_DURATION_CAMEL
    )
}

fn toast_entries(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> Vec<ToastEntry> {
    queue_value(state, descriptor)
        .map(|value| toast_entry_list(value, 0))
        .unwrap_or_default()
}

fn toast_entry_list(value: &UiValue, start_index: i64) -> Vec<ToastEntry> {
    match value {
        UiValue::Array(values) => values
            .iter()
            .enumerate()
            .flat_map(|(offset, value)| toast_entry_list(value, start_index + offset as i64))
            .collect(),
        UiValue::String(value) | UiValue::Enum(value) => {
            toast_entry_from_string(value, start_index)
                .into_iter()
                .collect()
        }
        UiValue::Map(values) => toast_entry_from_map(values, start_index)
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn toast_entry_from_string(value: &str, _index: i64) -> Option<ToastEntry> {
    let mut parts = value.split('|');
    let id = parts.next()?.trim().to_string();
    if id.is_empty() {
        return None;
    }

    let mut entry = ToastEntry {
        id: id.clone(),
        message: id,
        action_label: String::new(),
        duration_ms: None,
        raw: UiValue::String(value.to_string()),
    };
    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        match key.trim() {
            "message" | "text" | "label" | "title" => entry.message = value.trim().to_string(),
            "action" | "action_label" | "actionLabel" => {
                entry.action_label = value.trim().to_string()
            }
            "duration" | "duration_ms" | "auto_hide_duration_ms" | "autoHideDuration" => {
                entry.duration_ms = value.trim().parse::<i64>().ok()
            }
            _ => {}
        }
    }
    Some(entry)
}

fn toast_entry_from_map(values: &BTreeMap<String, UiValue>, _index: i64) -> Option<ToastEntry> {
    let id = first_string_value(values, &["id", "toast_id", "toastId", "value", "key"])?;
    if id.is_empty() {
        return None;
    }

    let message = first_string_value(values, &["message", "text", "label", "title"])
        .unwrap_or_else(|| id.clone());
    Some(ToastEntry {
        id,
        message,
        action_label: first_string_value(values, &["action_label", "actionLabel", "action"])
            .unwrap_or_default(),
        duration_ms: first_int_value(
            values,
            &[
                "duration",
                "duration_ms",
                "auto_hide_duration_ms",
                "autoHideDuration",
            ],
        ),
        raw: UiValue::Map(values.clone()),
    })
}

fn is_toast_control(descriptor: &UiComponentDescriptor) -> bool {
    matches!(descriptor.role.as_str(), "snackbar" | "toast")
        || matches!(descriptor.id.as_str(), "Snackbar" | "Toast")
}

#[derive(Clone, Debug, PartialEq)]
struct ToastEntry {
    id: String,
    message: String,
    action_label: String,
    duration_ms: Option<i64>,
    raw: UiValue,
}

impl ToastEntry {
    fn matches_id(&self, id: &str) -> bool {
        !id.is_empty() && self.id == id
    }
}

fn queue_value<'a>(
    state: &'a UiComponentState,
    descriptor: &'a UiComponentDescriptor,
) -> Option<&'a UiValue> {
    state
        .values
        .get(TOAST_QUEUE)
        .or_else(|| state.values.get(QUEUE))
        .or_else(|| default_value(descriptor, TOAST_QUEUE))
        .or_else(|| default_value(descriptor, QUEUE))
}

fn has_explicit_queue(state: &UiComponentState) -> bool {
    state.values.contains_key(TOAST_QUEUE) || state.values.contains_key(QUEUE)
}

fn value_setting<'a>(
    state: &'a UiComponentState,
    descriptor: &'a UiComponentDescriptor,
    property: &str,
) -> Option<&'a UiValue> {
    state
        .values
        .get(property)
        .or_else(|| default_value(descriptor, property))
}

fn default_value<'a>(descriptor: &'a UiComponentDescriptor, property: &str) -> Option<&'a UiValue> {
    descriptor
        .prop(property)
        .and_then(|schema| schema.default_value.as_ref())
}

fn string_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<String> {
    value_setting(state, descriptor, property).and_then(string_value)
}

fn first_string_value(values: &BTreeMap<String, UiValue>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| values.get(*key).and_then(string_value))
        .find(|value| !value.is_empty())
}

fn first_int_value(values: &BTreeMap<String, UiValue>, keys: &[&str]) -> Option<i64> {
    keys.iter()
        .find_map(|key| values.get(*key).and_then(int_value))
}

fn string_value(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.clone()),
        _ => None,
    }
}

fn int_value(value: &UiValue) -> Option<i64> {
    match value {
        UiValue::Int(value) => Some(*value),
        UiValue::Float(value) => Some(value.round() as i64),
        UiValue::String(value) | UiValue::Enum(value) => value.parse::<i64>().ok(),
        _ => None,
    }
}

fn set_optional_string(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    value: &str,
) {
    if descriptor.prop(property).is_some() || state.values.contains_key(property) {
        super::set_value(
            state,
            property.to_string(),
            UiValue::String(value.to_string()),
        );
    }
}

fn set_optional_int(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    value: i64,
) {
    if descriptor.prop(property).is_some() || state.values.contains_key(property) {
        super::set_value(state, property.to_string(), UiValue::Int(value));
    }
}
