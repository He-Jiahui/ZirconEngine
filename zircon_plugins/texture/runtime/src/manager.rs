#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureImportSummary {
    pub width: u32,
    pub height: u32,
    pub mip_count: u32,
    pub texel_count: u64,
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTextureManager;

impl DefaultTextureManager {
    pub fn summarize_texture(
        &self,
        width: u32,
        height: u32,
        mip_count: u32,
    ) -> TextureImportSummary {
        TextureImportSummary {
            width,
            height,
            mip_count: mip_count.max(1),
            texel_count: u64::from(width).saturating_mul(u64::from(height)),
        }
    }
}
