//! CLI 和构建工具共用的离线烘焙请求；身份参数声明产物适用哪一个字体实例，选择参数控制需要覆盖的 Unicode 字符。

//! Typed requests for offline font distance-field baking.

use super::FontSdfBakeError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// 公开工具选项到 runtime SdfMode 的映射；通道数和 shader 语义最终由 runtime 模式定义。
pub enum FontSdfBakeMode {
    #[default]
    Sdf,
    Msdf,
    Mtsdf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// AllCmap 枚举可映射字符，Codepoints 只请求指定 Unicode 标量；多个字符映射同一 glyph 时由 bake.rs 的 glyph_map 去重。
pub enum FontSdfGlyphSelection {
    AllCmap,
    Codepoints(Vec<u32>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 一次字体源和字型面的离线产物请求；调用 bake_font_sdf_artifact 时，字体字节必须与这些身份字段指向同一资源。
pub struct FontSdfBakeRequest {
    pub asset_guid: String,
    pub face_index: u32,
    /// Only the canonical default-coordinate hash is supported; non-default identities are rejected before decoding.
    pub variation_hash: [u8; 32],
    pub mode: FontSdfBakeMode,
    pub page_size: u32,
    pub bake_em_px: u32,
    pub spread_px_milli: u32,
    pub selection: FontSdfGlyphSelection,
}

impl FontSdfBakeRequest {
    /// 在解码字体和生成任务前拒绝空选择、非法 Unicode 与零尺寸；字体面、GUID 与产物几何由后续阶段校验。
    pub(crate) fn validate(&self) -> Result<(), FontSdfBakeError> {
        if self.variation_hash != crate::text::sdf::sdf_default_variation_hash().into_bytes() {
            return Err(FontSdfBakeError::UnsupportedVariationHash);
        }
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
