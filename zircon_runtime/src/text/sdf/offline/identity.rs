//! 以资产 GUID、字型面、变体、提取后的独立字型面字节和烘焙参数共同标识离线字形；运行时必须逐项核对，防止把旧像素用于新字体实例。

use uuid::Uuid;

use super::SdfOfflineArtifactError;
use crate::text::sdf::SdfBakeParams;
use crate::text::{StableContentDigest, VariationCoords};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// 像素内容的身份必须含字体字节和真实变体坐标哈希；工具写入的声明值与生成上下文不一致时，运行时无法从 identity 发现假匹配。
pub(crate) struct SdfOfflineArtifactIdentity {
    pub(crate) asset_guid: String,
    pub(crate) face_index: u32,
    pub(crate) variation_hash: StableContentDigest,
    pub(crate) source_hash: StableContentDigest,
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

    /// 运行时从当前字体源重建期望值并逐项比较；路径或文件校验和只能证明定位及完整性，不能证明适用当前字体。
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

pub(crate) fn sdf_default_variation_hash() -> StableContentDigest {
    sdf_variation_hash(&VariationCoords::default())
}

pub(crate) fn sdf_variation_hash(variations: &VariationCoords) -> StableContentDigest {
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
    StableContentDigest::from_bytes(*hasher.finalize().as_bytes())
}

pub(crate) fn sdf_font_source_hash(bytes: &[u8]) -> StableContentDigest {
    StableContentDigest::blake3(bytes)
}
