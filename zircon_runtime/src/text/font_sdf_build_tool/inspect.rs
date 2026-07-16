//! Inspection of encoded offline font distance-field artifacts.

use crate::text::sdf::{SdfMode, SdfOfflineArtifact};

use super::{FontSdfBakeError, FontSdfBakeMode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontSdfArtifactInspection {
    pub asset_guid: String,
    pub face_index: u32,
    pub variation_hash: [u8; 32],
    pub source_hash: [u8; 32],
    pub mode: FontSdfBakeMode,
    pub bake_em_px: u32,
    pub spread_px_milli: u32,
    pub page_width: u32,
    pub page_height: u32,
    pub page_count: usize,
    pub glyph_count: usize,
    pub encoded_len: usize,
}

pub fn inspect_font_sdf_artifact(
    bytes: &[u8],
) -> Result<FontSdfArtifactInspection, FontSdfBakeError> {
    let artifact = SdfOfflineArtifact::decode(bytes)
        .map_err(|error| FontSdfBakeError::Artifact(error.to_string()))?;
    let identity = artifact.identity();
    let page_size = artifact.page_size();
    Ok(FontSdfArtifactInspection {
        asset_guid: identity.asset_guid.clone(),
        face_index: identity.face_index,
        variation_hash: identity.variation_hash,
        source_hash: identity.source_hash,
        mode: public_mode(identity.params.mode),
        bake_em_px: identity.params.bake_em_px,
        spread_px_milli: identity.params.spread_px_milli,
        page_width: page_size.x,
        page_height: page_size.y,
        page_count: artifact.pages().len(),
        glyph_count: artifact.glyphs().len(),
        encoded_len: bytes.len(),
    })
}

fn public_mode(mode: SdfMode) -> FontSdfBakeMode {
    match mode {
        SdfMode::Sdf => FontSdfBakeMode::Sdf,
        SdfMode::Msdf => FontSdfBakeMode::Msdf,
        SdfMode::Mtsdf => FontSdfBakeMode::Mtsdf,
    }
}
