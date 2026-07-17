mod attributes;
mod entries;
mod entry;
mod options;
mod parse;

pub(in crate::ui::retained_host::ui) use self::options::{
    projected_notification_center_option_rows, projected_notification_center_options,
    projected_notification_center_structured_options, projected_notification_center_value_text,
};

#[cfg(test)]
mod tests;
