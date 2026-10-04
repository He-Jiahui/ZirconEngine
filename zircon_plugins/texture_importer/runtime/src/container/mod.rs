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
    /// Rewritten container bytes when the importer expands standard KTX2 supercompression.
    pub(crate) upload_bytes: Option<Vec<u8>>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) dimension: RenderImageDimension,
    pub(crate) depth_or_array_layers: u32,
    pub(crate) mip_count: u32,
    pub(crate) array_layers: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TextureContainerKind {
    Dds,
    Ktx1,
    Ktx2,
    Astc,
}

impl TextureContainerKind {
    fn from_extension(extension: &str) -> Option<Self> {
        if extension.eq_ignore_ascii_case("dds") {
            Some(Self::Dds)
        } else if extension.eq_ignore_ascii_case("ktx") {
            Some(Self::Ktx1)
        } else if extension.eq_ignore_ascii_case("ktx2") {
            Some(Self::Ktx2)
        } else if extension.eq_ignore_ascii_case("astc") {
            Some(Self::Astc)
        } else {
            None
        }
    }
}

pub(crate) fn parse_container_info(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    match TextureContainerKind::from_extension(extension) {
        Some(TextureContainerKind::Dds) => dds::parse(context),
        Some(TextureContainerKind::Ktx1) => ktx::parse_ktx1(context),
        Some(TextureContainerKind::Ktx2) => ktx::parse_ktx2(context),
        Some(TextureContainerKind::Astc) => astc::parse(context),
        None => Err(AssetImportError::UnsupportedFormat(format!(
            "texture container importer does not handle {}",
            context.source_path.display()
        ))),
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests/mod_plugins07_container_hotpath_tests.rs"]
mod plugins07_container_hotpath_tests;
