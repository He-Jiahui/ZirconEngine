//! Shared visual and pointer geometry for editor asset content surfaces.

mod controls;
mod metrics;
mod profile;
mod text;
mod thumbnail_grid;

pub(crate) use metrics::AssetContentLayoutMetrics;
pub(crate) use profile::AssetContentSurfaceProfile;
pub(crate) use text::{compact_file_like_display_name, RuntimeFileNameCompaction};
pub(crate) use thumbnail_grid::AssetThumbnailGridMetrics;

#[cfg(test)]
mod tests;
pub(crate) use controls::{
    ACTIVITY_CONTENT_EMPTY_CONTROL_ID, ACTIVITY_CONTENT_FOLDER_PREFIX,
    ACTIVITY_CONTENT_ITEM_PREFIX, ACTIVITY_CONTENT_PANEL_CONTROL_ID, BROWSER_CONTENT_ITEM_PREFIX,
    BROWSER_CONTENT_LIST_ROW_HEIGHT, BROWSER_CONTENT_PREVIEW_CONTROL_ID,
    BROWSER_CONTENT_TABLE_CONTROL_ID, BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID,
    BROWSER_CONTENT_TABLE_HEADER_HEIGHT, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
};
