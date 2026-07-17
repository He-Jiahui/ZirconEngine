use crate::ui::workbench::snapshot::AssetSelectionSnapshot;

use super::asset_details_constants::{
    ASSET_DETAILS_DIAGNOSTICS_HEIGHT, ASSET_DETAILS_IDENTITY_HEIGHT, ASSET_DETAILS_LOCATOR_HEIGHT,
    ASSET_DETAILS_METADATA_HEIGHT, ASSET_DETAILS_PADDING, ASSET_DETAILS_PREVIEW_HEIGHT,
    ASSET_DETAILS_SECTION_GAP, ASSET_DETAILS_TYPE_HEIGHT,
};
use super::asset_details_sections_len::asset_details_sections_len;

pub(crate) fn asset_details_content_extent(selection: &AssetSelectionSnapshot) -> f32 {
    const BASE_CONTENT_HEIGHT: f32 = ASSET_DETAILS_PREVIEW_HEIGHT
        + ASSET_DETAILS_LOCATOR_HEIGHT
        + ASSET_DETAILS_TYPE_HEIGHT
        + ASSET_DETAILS_IDENTITY_HEIGHT
        + ASSET_DETAILS_METADATA_HEIGHT;

    let has_diagnostics = !selection.diagnostics.is_empty();
    let content = BASE_CONTENT_HEIGHT
        + if has_diagnostics {
            ASSET_DETAILS_DIAGNOSTICS_HEIGHT
        } else {
            0.0
        };
    let section_count = asset_details_sections_len(selection);
    let gaps = (section_count.saturating_sub(1) as f32) * ASSET_DETAILS_SECTION_GAP;
    ASSET_DETAILS_PADDING * 2.0 + content + gaps
}
