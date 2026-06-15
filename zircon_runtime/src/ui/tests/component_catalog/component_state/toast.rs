use std::collections::BTreeMap;

use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEvent, UiComponentEventKind, UiComponentState, UiValue,
};

#[test]
fn toast_queue_expires_in_order_and_ignores_stale_timers() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let snackbar = registry
        .descriptor("Snackbar")
        .expect("Snackbar descriptor");
    assert!(snackbar.supports_event(UiComponentEventKind::ValueChanged));
    assert!(snackbar.supports_event(UiComponentEventKind::Commit));

    let mut state = UiComponentState::new();
    state
        .apply_event(
            snackbar,
            UiComponentEvent::ValueChanged {
                property: "toast_queue".to_string(),
                value: toast_entries(),
            },
        )
        .unwrap();

    assert_current_toast(&state, "build", "Build failed", "Open log", 5_000);
    assert_eq!(state.value("queue_length"), Some(&UiValue::Int(3)));
    assert_eq!(state.value("open"), Some(&UiValue::Bool(true)));
    assert!(state.flags.popup_open);

    expire_toast(&mut state, snackbar, "build");
    assert_current_toast(
        &state,
        "asset",
        "Asset import complete",
        "Show in browser",
        2_500,
    );
    assert_eq!(
        state.value("expired_toast_id"),
        Some(&UiValue::String("build".to_string()))
    );
    assert_eq!(state.value("queue_length"), Some(&UiValue::Int(2)));
    assert_queue_ids(&state, &["asset", "source"]);

    expire_toast(&mut state, snackbar, "build");
    assert_current_toast(
        &state,
        "asset",
        "Asset import complete",
        "Show in browser",
        2_500,
    );
    assert_eq!(
        state.value("expired_toast_id"),
        Some(&UiValue::String("build".to_string())),
        "late timeout from the previous toast must not close the current toast"
    );

    expire_toast(&mut state, snackbar, "asset");
    assert_current_toast(
        &state,
        "source",
        "Source control synced",
        "View changes",
        3_000,
    );
    assert_eq!(state.value("queue_length"), Some(&UiValue::Int(1)));

    expire_toast(&mut state, snackbar, "source");
    assert_eq!(
        state.value("current_toast_id"),
        Some(&UiValue::String(String::new()))
    );
    assert_eq!(
        state.value("message"),
        Some(&UiValue::String(String::new()))
    );
    assert_eq!(state.value("queue_length"), Some(&UiValue::Int(0)));
    assert_eq!(state.value("open"), Some(&UiValue::Bool(false)));
    assert!(!state.flags.popup_open);
}

#[test]
fn toast_close_popup_discards_current_and_promotes_next() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let snackbar = registry
        .descriptor("Snackbar")
        .expect("Snackbar descriptor");
    assert!(snackbar.supports_event(UiComponentEventKind::ClosePopup));

    let mut state = UiComponentState::new()
        .with_value("toast_queue", toast_entries())
        .with_value("current_toast_id", UiValue::String(String::new()));
    state
        .apply_event(snackbar, UiComponentEvent::OpenPopup)
        .unwrap();
    assert_current_toast(&state, "build", "Build failed", "Open log", 5_000);

    state
        .apply_event(snackbar, UiComponentEvent::ClosePopup)
        .unwrap();

    assert_current_toast(
        &state,
        "asset",
        "Asset import complete",
        "Show in browser",
        2_500,
    );
    assert_eq!(state.value("queue_length"), Some(&UiValue::Int(2)));
    assert!(state.flags.popup_open);
}

fn expire_toast(state: &mut UiComponentState, descriptor: &UiComponentDescriptor, toast_id: &str) {
    state
        .apply_event(
            descriptor,
            UiComponentEvent::Commit {
                property: "expired_toast_id".to_string(),
                value: UiValue::String(toast_id.to_string()),
            },
        )
        .unwrap();
}

fn toast_entries() -> UiValue {
    UiValue::Array(vec![
        toast("build", "Build failed", "Open log", 5_000),
        toast("asset", "Asset import complete", "Show in browser", 2_500),
        toast("source", "Source control synced", "View changes", 3_000),
    ])
}

fn toast(id: &str, message: &str, action_label: &str, duration_ms: i64) -> UiValue {
    UiValue::Map(BTreeMap::from([
        ("id".to_string(), UiValue::String(id.to_string())),
        ("message".to_string(), UiValue::String(message.to_string())),
        (
            "action_label".to_string(),
            UiValue::String(action_label.to_string()),
        ),
        (
            "auto_hide_duration_ms".to_string(),
            UiValue::Int(duration_ms),
        ),
    ]))
}

fn assert_current_toast(
    state: &UiComponentState,
    id: &str,
    message: &str,
    action_label: &str,
    duration_ms: i64,
) {
    assert_eq!(
        state.value("current_toast_id"),
        Some(&UiValue::String(id.to_string()))
    );
    assert_eq!(
        state.value("message"),
        Some(&UiValue::String(message.to_string()))
    );
    assert_eq!(
        state.value("action_label"),
        Some(&UiValue::String(action_label.to_string()))
    );
    assert_eq!(
        state.value("auto_hide_duration_ms"),
        Some(&UiValue::Int(duration_ms))
    );
}

fn assert_queue_ids(state: &UiComponentState, expected: &[&str]) {
    let Some(UiValue::Array(entries)) = state.value("toast_queue") else {
        panic!("toast_queue should stay as an array");
    };
    let actual = entries
        .iter()
        .filter_map(|value| match value {
            UiValue::Map(values) => values.get("id").and_then(string_value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(actual.as_slice(), expected);
}

fn string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.as_str()),
        _ => None,
    }
}
