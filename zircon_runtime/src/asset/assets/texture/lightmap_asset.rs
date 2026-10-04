use crate::asset::AssetUri;
use crate::core::framework::render::{
    LightmapBakeOutput, LightmapContractValidationError, RenderImageColorSpace,
    RenderImageDimension,
};

use super::{TextureAsset, TextureAssetDescriptor};

pub const LIGHTMAP_RGBA16F_FORMAT: &str = "zircon-lightmap-rgba16f-le-v1";
pub const LIGHTMAP_RGBA16F_GPU_FORMAT: &str = "rgba16float";

pub fn texture_asset_from_lightmap_bake_output(
    uri: AssetUri,
    output: &LightmapBakeOutput,
) -> Result<TextureAsset, LightmapContractValidationError> {
    output.validate()?;
    let payload = ordered_lightmap_payload(output)?;
    let mut descriptor =
        TextureAssetDescriptor::container(LIGHTMAP_RGBA16F_GPU_FORMAT, 1, output.atlas.page_count);
    descriptor.color_space = RenderImageColorSpace::Linear;
    descriptor.dimension = RenderImageDimension::D2;

    Ok(TextureAsset::new_container(
        uri,
        output.atlas.page_size,
        output.atlas.page_size,
        LIGHTMAP_RGBA16F_FORMAT,
        payload,
        1,
        output.atlas.page_count,
    )
    .with_descriptor(descriptor))
}

fn ordered_lightmap_payload(
    output: &LightmapBakeOutput,
) -> Result<Vec<u8>, LightmapContractValidationError> {
    let first_page = output
        .atlas_pages
        .first()
        .expect("validated lightmap output must contain at least one atlas page");
    let payload_capacity = first_page
        .texels_rgba16f_le
        .len()
        .checked_mul(output.atlas_pages.len())
        .ok_or(LightmapContractValidationError::AtlasPayloadSizeOverflow)?;
    let mut ordered_pages = vec![first_page; output.atlas_pages.len()];
    for page in &output.atlas_pages {
        ordered_pages[page.page_index as usize] = page;
    }

    let mut payload = Vec::with_capacity(payload_capacity);
    for page in ordered_pages {
        payload.extend_from_slice(&page.texels_rgba16f_le);
    }
    Ok(payload)
}

#[cfg(test)]
#[path = "tests/lightmap_asset.rs"]
mod tests;
