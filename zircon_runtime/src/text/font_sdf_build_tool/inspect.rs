//! 提供不需加载字体或建立任务池的产物检查 API，当前集成测试用它核对烘焙输出；通过格式校验仍须由运行时核对当前字体 identity。

//! Inspection of encoded offline font distance-field artifacts.

use crate::text::sdf::{SdfMode, SdfOfflineArtifact};

use super::{FontSdfBakeError, FontSdfBakeMode};

#[derive(Clone, Debug, PartialEq, Eq)]
/// 解码后的身份与容量摘要；这是文件声明的身份，调用方尚未把它同正在使用的字体源比较。
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

/// 可在没有字体源的工具环境调用；先由 runtime codec 校验文件完整性和几何，再返回摘要。适用某个字体仍需 validate_identity。
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
        variation_hash: identity.variation_hash.into_bytes(),
        source_hash: identity.source_hash.into_bytes(),
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

/// 把 runtime 解码后的模式重新映射为工具公开类型，让 CLI 无需依赖内部 shader 判别值。
fn public_mode(mode: SdfMode) -> FontSdfBakeMode {
    match mode {
        SdfMode::Sdf => FontSdfBakeMode::Sdf,
        SdfMode::Msdf => FontSdfBakeMode::Msdf,
        SdfMode::Mtsdf => FontSdfBakeMode::Mtsdf,
    }
}
