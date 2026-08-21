//! Shared visual and pointer geometry for editor asset content surfaces.

mod controls;
mod metrics;
mod paint_metadata;
mod profile;
mod text;
mod thumbnail_grid;

pub(crate) use metrics::AssetContentLayoutMetrics;
pub(crate) use paint_metadata::{
    asset_content_paint_metadata, ActivityContentNodeIdentity, AssetContentNodeIdentity,
    AssetContentPaintMetadata, AssetContentPaintNodeInput, AssetContentRect, AssetContentSurface,
    BrowserContentNodeIdentity,
};
#[cfg(test)]
pub(crate) use paint_metadata::{
    parse_activity_content_identity, parse_browser_content_identity, ActivityContentNodeRole,
};
pub(crate) use profile::AssetContentSurfaceProfile;
pub(crate) use text::{compact_file_like_display_name, RuntimeFileNameCompaction};
pub(crate) use thumbnail_grid::AssetThumbnailGridMetrics;

#[cfg(test)]
mod tests;
pub(crate) use controls::{
    activity_reference_row_index, browser_reference_row_index, browser_source_tree_row_index,
    ActivityAssetReferenceListKind, BrowserAssetReferenceListKind,
    ACTIVITY_CONTENT_EMPTY_CONTROL_ID, ACTIVITY_CONTENT_FOLDER_PREFIX,
    ACTIVITY_CONTENT_ITEM_PREFIX, ACTIVITY_CONTENT_PANEL_CONTROL_ID, BROWSER_CONTENT_ITEM_PREFIX,
    BROWSER_CONTENT_LIST_ROW_HEIGHT, BROWSER_CONTENT_PREVIEW_CONTROL_ID,
    BROWSER_CONTENT_TABLE_CONTROL_ID, BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID,
    BROWSER_CONTENT_TABLE_HEADER_HEIGHT, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
};
