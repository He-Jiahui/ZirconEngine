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
    let mut pages = output.atlas_pages.iter().collect::<Vec<_>>();
    pages.sort_by_key(|page| page.page_index);
    let payload = pages
        .into_iter()
        .flat_map(|page| page.texels_rgba16f_le.iter().copied())
        .collect();
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

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        LIGHTMAP_CONSUME_CONTRACT_VERSION, LightmapAtlasDescriptor, LightmapAtlasFormat,
        LightmapAtlasPage, LightmapBakeOutput,
    };

    use super::*;

    #[test]
    fn lightmap_bake_output_becomes_sorted_rgba16f_array_asset() {
        let output = LightmapBakeOutput {
            contract_version: LIGHTMAP_CONSUME_CONTRACT_VERSION,
            request_id: 1,
            scene_revision: 1,
            light_set_generation: 1,
            atlas: LightmapAtlasDescriptor {
                page_size: 1,
                page_count: 2,
                format: LightmapAtlasFormat::Rgba16Float,
            },
            atlas_pages: vec![
                LightmapAtlasPage {
                    page_index: 1,
                    texels_rgba16f_le: vec![2; 8],
                },
                LightmapAtlasPage {
                    page_index: 0,
                    texels_rgba16f_le: vec![1; 8],
                },
            ],
            slots: Vec::new(),
            probe_grid: None,
        };

        let asset = texture_asset_from_lightmap_bake_output(
            AssetUri::parse("res://lighting/test.lightmap-array").expect("valid test URI"),
            &output,
        )
        .expect("valid bake output should become a texture asset");

        assert_eq!(asset.width, 1);
        assert_eq!(asset.height, 1);
        assert_eq!(asset.rgba, Vec::<u8>::new());
        let super::super::TexturePayload::Container {
            format,
            bytes,
            mip_count,
            array_layers,
        } = &asset.payload
        else {
            panic!("lightmap atlas must use the raw container payload");
        };
        assert_eq!(format, LIGHTMAP_RGBA16F_FORMAT);
        assert_eq!(bytes, &[vec![1; 8], vec![2; 8]].concat());
        assert_eq!(*mip_count, 1);
        assert_eq!(*array_layers, 2);
        let descriptor = asset.render_image_descriptor();
        assert_eq!(descriptor.format, LIGHTMAP_RGBA16F_GPU_FORMAT);
        assert_eq!(descriptor.color_space, RenderImageColorSpace::Linear);
        assert_eq!(descriptor.array_layer_count, 2);
    }
}
