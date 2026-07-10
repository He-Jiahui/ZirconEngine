//! Shared visual and pointer geometry for editor asset content surfaces.

mod controls;
mod labels;
mod metrics;
mod profile;
mod text;

pub(crate) use labels::resource_kind_badge_code;
pub(crate) use metrics::AssetContentLayoutMetrics;
pub(crate) use profile::AssetContentSurfaceProfile;
pub(crate) use text::{compact_file_like_display_name, RuntimeFileNameCompaction};

#[cfg(test)]
mod tests;
pub(crate) use controls::{
    ACTIVITY_CONTENT_EMPTY_CONTROL_ID, ACTIVITY_CONTENT_FOLDER_PREFIX,
    ACTIVITY_CONTENT_ITEM_PREFIX, ACTIVITY_CONTENT_PANEL_CONTROL_ID,
};
