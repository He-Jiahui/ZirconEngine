use uuid::Uuid;

use super::SdfOfflineArtifactError;
use crate::core::framework::render::VariationCoords;
use crate::graphics::text::sdf::SdfBakeParams;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SdfOfflineArtifactIdentity {
    pub(crate) asset_guid: String,
    pub(crate) face_index: u32,
    pub(crate) variation_hash: [u8; 32],
    pub(crate) source_hash: [u8; 32],
    pub(crate) params: SdfBakeParams,
}

impl SdfOfflineArtifactIdentity {
    pub(crate) fn normalized(mut self) -> Result<Self, SdfOfflineArtifactError> {
        let parsed = Uuid::parse_str(&self.asset_guid)
            .map_err(|_| SdfOfflineArtifactError::InvalidAssetGuid(self.asset_guid.clone()))?;
        self.asset_guid = parsed.to_string();
        self.params = self.params.normalized();
        Ok(self)
    }

    pub(crate) fn validate_matches(&self, expected: &Self) -> Result<(), SdfOfflineArtifactError> {
        let expected = expected.clone().normalized()?;
        for (matches, field) in [
            (self.asset_guid == expected.asset_guid, "asset_guid"),
            (self.face_index == expected.face_index, "face_index"),
            (
                self.variation_hash == expected.variation_hash,
                "variation_hash",
            ),
            (self.source_hash == expected.source_hash, "source_hash"),
            (self.params.mode == expected.params.mode, "mode"),
            (
                self.params.bake_em_px == expected.params.bake_em_px,
                "bake_em_px",
            ),
            (
                self.params.spread_px_milli == expected.params.spread_px_milli,
                "spread_px_milli",
            ),
        ] {
            if !matches {
                return Err(SdfOfflineArtifactError::IdentityMismatch { field });
            }
        }
        Ok(())
    }
}

pub(crate) fn sdf_default_variation_hash() -> [u8; 32] {
    sdf_variation_hash(&VariationCoords::default())
}

pub(crate) fn sdf_variation_hash(variations: &VariationCoords) -> [u8; 32] {
    let mut coordinates = variations
        .0
        .iter()
        .map(|(tag, value)| (*tag, value.to_bits()))
        .collect::<Vec<_>>();
    coordinates.sort_unstable();
    let mut hasher = blake3::Hasher::new();
    for (tag, value_bits) in coordinates {
        hasher.update(&tag.to_be_bytes());
        hasher.update(&value_bits.to_le_bytes());
    }
    *hasher.finalize().as_bytes()
}

pub(crate) fn sdf_font_source_hash(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}
