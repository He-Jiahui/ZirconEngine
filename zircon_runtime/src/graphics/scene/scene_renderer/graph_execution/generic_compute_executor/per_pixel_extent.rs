use crate::render_graph::{ComputeBindingKind, RenderGraphComputePassMetadata};
use crate::rhi::TextureDesc;

pub(super) fn per_pixel_target_extent(
    metadata: &RenderGraphComputePassMetadata,
    target: &str,
    texture: &TextureDesc,
) -> Result<[u32; 2], String> {
    let mip_level = selected_target_mip_level(metadata, target)?;
    Ok(texture_mip_extent(texture, mip_level))
}

fn selected_target_mip_level(
    metadata: &RenderGraphComputePassMetadata,
    target: &str,
) -> Result<Option<u32>, String> {
    let mut storage_writes = metadata.bindings.iter().filter(|binding| {
        binding.resource == target && binding.kind == ComputeBindingKind::StorageTextureWrite
    });
    if let Some(target_binding) = storage_writes.next() {
        if storage_writes.next().is_some() {
            return Err(format!(
                "compute per-pixel target `{target}` has multiple storage texture write bindings; select one output resource per pass"
            ));
        }
        return Ok(target_binding.texture_mip_level);
    }

    let mut texture_mip_levels = metadata
        .bindings
        .iter()
        .filter(|binding| {
            binding.resource == target && binding.kind == ComputeBindingKind::SampledTexture
        })
        .map(|binding| binding.texture_mip_level);
    let Some(mip_level) = texture_mip_levels.next() else {
        return Ok(None);
    };
    if texture_mip_levels.any(|candidate| candidate != mip_level) {
        return Err(format!(
            "compute per-pixel target `{target}` has sampled texture bindings with different mip selections"
        ));
    }
    Ok(mip_level)
}

fn texture_mip_extent(texture: &TextureDesc, mip_level: Option<u32>) -> [u32; 2] {
    let mip_level = mip_level.unwrap_or_default();
    [
        texture.width.checked_shr(mip_level).unwrap_or(1).max(1),
        texture.height.checked_shr(mip_level).unwrap_or(1).max(1),
    ]
}

#[cfg(test)]
mod tests {
    use crate::render_graph::{
        BindingSchemaEntry, ComputeBindingKind, RenderGraphComputePassMetadata,
        RenderGraphComputeShaderSource,
    };
    use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

    use super::per_pixel_target_extent;

    #[test]
    fn selected_storage_mip_controls_per_pixel_extent() {
        let metadata = RenderGraphComputePassMetadata::new(
            RenderGraphComputeShaderSource::wgsl("hzb", "@compute fn cs_main() {}"),
            "cs_main",
            vec![
                BindingSchemaEntry::new(0, "hzb", ComputeBindingKind::SampledTexture)
                    .with_texture_mip_level(1),
                BindingSchemaEntry::new(1, "hzb", ComputeBindingKind::StorageTextureWrite)
                    .with_texture_mip_level(2),
            ],
        );
        let texture = TextureDesc::new(
            "hzb",
            65,
            33,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_mip_levels(7);

        assert_eq!(
            per_pixel_target_extent(&metadata, "hzb", &texture),
            Ok([16, 8])
        );
    }

    #[test]
    fn per_pixel_target_rejects_multiple_storage_outputs_for_one_resource() {
        let metadata = RenderGraphComputePassMetadata::new(
            RenderGraphComputeShaderSource::wgsl("hzb", "@compute fn cs_main() {}"),
            "cs_main",
            vec![
                BindingSchemaEntry::new(0, "hzb", ComputeBindingKind::StorageTextureWrite),
                BindingSchemaEntry::new(1, "hzb", ComputeBindingKind::StorageTextureWrite)
                    .with_texture_mip_level(1),
            ],
        );
        let texture = TextureDesc::new(
            "hzb",
            64,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_mip_levels(2);

        assert!(per_pixel_target_extent(&metadata, "hzb", &texture)
            .expect_err("multiple storage outputs must be rejected")
            .contains("multiple storage texture write bindings"));
    }

    #[test]
    fn sampled_target_mip_controls_per_pixel_extent_without_a_storage_output() {
        let metadata = RenderGraphComputePassMetadata::new(
            RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
            "cs_main",
            vec![
                BindingSchemaEntry::new(0, "source", ComputeBindingKind::SampledTexture)
                    .with_texture_mip_level(3),
            ],
        );
        let texture = TextureDesc::new(
            "source",
            65,
            33,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED,
        )
        .with_mip_levels(7);

        assert_eq!(
            per_pixel_target_extent(&metadata, "source", &texture),
            Ok([8, 4])
        );
    }

    #[test]
    fn per_pixel_target_rejects_sampled_bindings_with_different_mips() {
        let metadata = RenderGraphComputePassMetadata::new(
            RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
            "cs_main",
            vec![
                BindingSchemaEntry::new(0, "source", ComputeBindingKind::SampledTexture)
                    .with_texture_mip_level(1),
                BindingSchemaEntry::new(1, "source", ComputeBindingKind::SampledTexture)
                    .with_texture_mip_level(2),
            ],
        );
        let texture = TextureDesc::new(
            "source",
            64,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED,
        )
        .with_mip_levels(3);

        assert!(per_pixel_target_extent(&metadata, "source", &texture)
            .expect_err("different sampled mips must be rejected")
            .contains("sampled texture bindings with different mip selections"));
    }
}
