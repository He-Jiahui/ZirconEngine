use crate::workbench::snapshot::AssetSelectionSnapshot;

use super::asset_details_constants::{
    ASSET_DETAILS_DIAGNOSTICS_HEIGHT, ASSET_DETAILS_IDENTITY_HEIGHT, ASSET_DETAILS_LOCATOR_HEIGHT,
    ASSET_DETAILS_METADATA_HEIGHT, ASSET_DETAILS_PADDING, ASSET_DETAILS_PREVIEW_HEIGHT,
    ASSET_DETAILS_SECTION_GAP, ASSET_DETAILS_TYPE_HEIGHT,
};
use super::asset_details_sections_len::asset_details_sections_len;

pub(crate) fn asset_details_content_extent(selection: &AssetSelectionSnapshot) -> f32 {
    let mut sections = vec![
        ASSET_DETAILS_PREVIEW_HEIGHT,
        ASSET_DETAILS_LOCATOR_HEIGHT,
        ASSET_DETAILS_TYPE_HEIGHT,
        ASSET_DETAILS_IDENTITY_HEIGHT,
        ASSET_DETAILS_METADATA_HEIGHT,
    ];
    if !selection.diagnostics.is_empty() {
        sections.push(ASSET_DETAILS_DIAGNOSTICS_HEIGHT);
    }
    let content = sections.iter().copied().sum::<f32>();
    let gaps = (asset_details_sections_len(selection).saturating_sub(1) as f32)
        * ASSET_DETAILS_SECTION_GAP;
    ASSET_DETAILS_PADDING * 2.0 + content + gaps
}
