#[path = "../../notification_center/attributes.rs"]
mod attributes;
#[path = "entries.rs"]
mod entries;
#[path = "entry.rs"]
mod entry;
#[path = "metadata.rs"]
mod metadata;
#[path = "options.rs"]
mod options;
#[path = "parse.rs"]
mod parse;

pub(in crate::ui::retained_host::ui) use self::metadata::{
    projected_notification_center_metadata, projected_notification_center_metadata_from_host,
    NotificationCenterMetadata,
};
pub(in crate::ui::retained_host::ui) use self::options::{
    projected_notification_center_option_rows, projected_notification_center_options,
    projected_notification_center_structured_options, projected_notification_center_value_text,
};

#[cfg(test)]
#[path = "../../tests/notification_center_cases.rs"]
mod tests;
