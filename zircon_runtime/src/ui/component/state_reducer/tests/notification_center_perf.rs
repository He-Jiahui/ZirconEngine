use zircon_runtime_interface::ui::component::{UiComponentKeyboardAction, UiValue};

use super::{collect_visible_notification_entries, next_enabled_notification, NotificationEntry};

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

#[test]
fn keyboard_navigation_filters_disabled_entries_without_materializing_a_second_list() {
    let entries = vec![
        NotificationEntry {
            id: "disabled".to_string(),
            index: 0,
            unread: false,
            disabled: true,
        },
        NotificationEntry {
            id: "first".to_string(),
            index: 2,
            unread: true,
            disabled: false,
        },
        NotificationEntry {
            id: "last".to_string(),
            index: 5,
            unread: false,
            disabled: false,
        },
    ];

    assert_eq!(
        next_enabled_notification(&entries, UiComponentKeyboardAction::First, -1)
            .map(|entry| entry.id.as_str()),
        Some("first")
    );
    assert_eq!(
        next_enabled_notification(&entries, UiComponentKeyboardAction::Last, -1)
            .map(|entry| entry.id.as_str()),
        Some("last")
    );
    assert_eq!(
        next_enabled_notification(&entries, UiComponentKeyboardAction::Next, 2)
            .map(|entry| entry.id.as_str()),
        Some("last")
    );
    assert_eq!(
        next_enabled_notification(&entries, UiComponentKeyboardAction::Previous, 5)
            .map(|entry| entry.id.as_str()),
        Some("first")
    );
}
