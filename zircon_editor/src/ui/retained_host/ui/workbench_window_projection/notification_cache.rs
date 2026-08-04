use std::rc::Rc;

use crate::ui::retained_host::{
    TemplatePaneNodeData, TemplatePaneOptionData,
    primitives::{ModelRc, SharedString},
    ui::pane_data_conversion::NotificationCenterMetadata,
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
mod tests {
    use super::*;

    #[test]
    fn reuse_requires_every_notification_presentation_key_to_match() {
        let previous = TemplatePaneNodeData {
            component_role: "notification-center".into(),
            notification_generation: 7,
            notification_unread_count: 2,
            notification_overflow_count: 3,
            notification_selected_id: "selected".into(),
            notification_focused_index: 1,
            notification_visible_limit: 8,
            ..TemplatePaneNodeData::default()
        };
        let metadata = NotificationCenterMetadata {
            generation: 7,
            unread_count: 2,
            overflow_count: 3,
            selected_id: "selected".to_string(),
            focused_index: Some(1),
            visible_limit: 8,
        };

        assert!(reusable_notification_rows(Some(&previous), &metadata).is_some());

        for changed in [
            NotificationCenterMetadata {
                generation: 8,
                ..metadata.clone()
            },
            NotificationCenterMetadata {
                unread_count: 1,
                ..metadata.clone()
            },
            NotificationCenterMetadata {
                overflow_count: 4,
                ..metadata.clone()
            },
            NotificationCenterMetadata {
                selected_id: "other".to_string(),
                ..metadata.clone()
            },
            NotificationCenterMetadata {
                focused_index: Some(2),
                ..metadata.clone()
            },
            NotificationCenterMetadata {
                visible_limit: 4,
                ..metadata.clone()
            },
        ] {
            assert!(reusable_notification_rows(Some(&previous), &changed).is_none());
        }

        let zero_generation = NotificationCenterMetadata {
            generation: 0,
            ..metadata
        };
        assert!(reusable_notification_rows(Some(&previous), &zero_generation).is_none());
    }
}
