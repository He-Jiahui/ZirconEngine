use std::rc::Rc;

use crate::ui::retained_host::{
    primitives::{ModelRc, SharedString},
    ui::pane_data_conversion::NotificationCenterMetadata,
    TemplatePaneNodeData, TemplatePaneOptionData,
};

pub(super) struct ReusedNotificationRows {
    pub options_text: Rc<String>,
    pub options: ModelRc<SharedString>,
    pub structured_options: ModelRc<TemplatePaneOptionData>,
}

pub(super) fn reusable_notification_rows(
    previous: Option<&TemplatePaneNodeData>,
    metadata: &NotificationCenterMetadata,
) -> Option<ReusedNotificationRows> {
    let previous = previous?;
    let focused_index = metadata
        .focused_index
        .and_then(|index| i32::try_from(index).ok())
        .unwrap_or(-1);
    let cache_key_matches = metadata.generation > 0
        && previous.component_role.as_str() == "notification-center"
        && previous.notification_generation == metadata.generation
        && previous.notification_unread_count == metadata.unread_count
        && previous.notification_overflow_count == metadata.overflow_count
        && previous.notification_selected_id.as_str() == metadata.selected_id.as_str()
        && previous.notification_focused_index == focused_index
        && previous.notification_visible_limit == metadata.visible_limit;

    cache_key_matches.then(|| ReusedNotificationRows {
        options_text: previous.options_text.clone(),
        options: previous.options.clone(),
        structured_options: previous.structured_options.clone(),
    })
}

#[cfg(test)]
#[path = "tests/notification_cache.rs"]
mod tests;
