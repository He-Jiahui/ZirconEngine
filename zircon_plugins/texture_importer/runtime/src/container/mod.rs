use zircon_runtime::asset::{AssetImportContext, AssetImportError};
use zircon_runtime::core::framework::render::RenderImageDimension;

mod astc;
mod dds;
mod ktx;
mod support;

#[cfg(test)]
use support::*;

pub(crate) struct TextureContainerInfo {
    pub(crate) format: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) dimension: RenderImageDimension,
    pub(crate) depth_or_array_layers: u32,
    pub(crate) mip_count: u32,
    pub(crate) array_layers: u32,
}

pub(crate) fn parse_container_info(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "dds" => dds::parse(context),
        "ktx" => ktx::parse_ktx1(context),
        "ktx2" => ktx::parse_ktx2(context),
        "astc" => astc::parse(context),
        _ => Err(AssetImportError::UnsupportedFormat(format!(
            "texture container importer does not handle {}",
            context.source_path.display()
        ))),
    }
}

#[cfg(test)]
mod tests;
