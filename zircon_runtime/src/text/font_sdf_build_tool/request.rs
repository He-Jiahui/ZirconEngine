//! Typed requests for offline font distance-field baking.

use super::FontSdfBakeError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontSdfBakeMode {
    #[default]
    Sdf,
    Msdf,
    Mtsdf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontSdfGlyphSelection {
    AllCmap,
    Codepoints(Vec<u32>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontSdfBakeRequest {
    pub asset_guid: String,
    pub face_index: u32,
    pub variation_hash: [u8; 32],
    pub mode: FontSdfBakeMode,
    pub page_size: u32,
    pub bake_em_px: u32,
    pub spread_px_milli: u32,
    pub selection: FontSdfGlyphSelection,
}

impl FontSdfBakeRequest {
    pub(crate) fn validate(&self) -> Result<(), FontSdfBakeError> {
        if self.page_size == 0 || self.bake_em_px == 0 || self.spread_px_milli == 0 {
            return Err(FontSdfBakeError::AtlasSizeOverflow);
        }
        if matches!(&self.selection, FontSdfGlyphSelection::Codepoints(values) if values.is_empty())
        {
            return Err(FontSdfBakeError::EmptySelection);
        }
        if let FontSdfGlyphSelection::Codepoints(values) = &self.selection {
            if let Some(codepoint) = values
                .iter()
                .copied()
                .find(|codepoint| char::from_u32(*codepoint).is_none())
            {
                return Err(FontSdfBakeError::InvalidCodepoint(codepoint));
            }
        }
        Ok(())
    }
}
