use super::super::super::data::{TemplatePaneNodeData, TemplatePaneOptionData};

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
    let unread_count = notification_rows(node)
        .filter(|option| option.unread)
        .count();
    if unread_count > 0 {
        format!("{title} ({unread_count})")
    } else {
        title.to_string()
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn empty_text(
    node: &TemplatePaneNodeData,
) -> String {
    non_empty(node.value_text.as_str())
        .unwrap_or("No notifications")
        .to_string()
}

fn notification_rows(
    node: &TemplatePaneNodeData,
) -> impl Iterator<Item = TemplatePaneOptionData> + '_ {
    (0..node.structured_options.row_count()).filter_map(|row| node.structured_options.row_data(row))
}

fn non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}
