use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_notification_center(
    node: &TemplatePaneNodeData,
) -> bool {
    node.role.as_str() == "NotificationCenter"
        || node.component_role.as_str() == "notification-center"
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn header_text(
    node: &TemplatePaneNodeData,
) -> String {
    let title = non_empty(node.text.as_str()).unwrap_or("Notifications");
    match (
        node.notification_unread_count,
        node.notification_overflow_count,
    ) {
        (0, 0) => title.to_string(),
        (unread_count, 0) => format!("{title} ({unread_count})"),
        (0, overflow_count) => format!("{title} +{overflow_count} omitted"),
        (unread_count, overflow_count) => {
            format!("{title} ({unread_count}) +{overflow_count} omitted")
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn empty_text(
    node: &TemplatePaneNodeData,
) -> String {
    non_empty(node.value_text.as_str())
        .unwrap_or("No notifications")
        .to_string()
}

fn non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_uses_generation_metadata_without_scanning_rows() {
        let node = TemplatePaneNodeData {
            text: "Notifications".into(),
            notification_unread_count: 3,
            notification_overflow_count: 12,
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(header_text(&node), "Notifications (3) +12 omitted");

        let source = include_str!("identity.rs");
        let row_collection = ["structured_", "options"].concat();
        let cloning_access = ["row_", "data"].concat();
        assert!(!source.contains(&row_collection));
        assert!(!source.contains(&cloning_access));
    }

    #[test]
    fn header_pixels_keep_the_existing_label_when_no_rows_were_dropped() {
        let unread = TemplatePaneNodeData {
            text: "Notifications".into(),
            notification_unread_count: 2,
            ..TemplatePaneNodeData::default()
        };
        let empty = TemplatePaneNodeData {
            text: "Notifications".into(),
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(header_text(&unread), "Notifications (2)");
        assert_eq!(header_text(&empty), "Notifications");
    }
}
