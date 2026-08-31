use crate::core::resource::ResourceKind;

use super::{RenderArtifactManifestError, block_at, validate_residency};
use crate::asset::artifact::render_manifest::{
    RenderArtifactManifest, RenderArtifactResidencyClass, RenderArtifactTextureLayout,
    RenderSubresourceId,
};

pub(super) fn validate_texture_layout(
    manifest: &RenderArtifactManifest,
    layout: &RenderArtifactTextureLayout,
) -> Result<(), RenderArtifactManifestError> {
    if manifest.resource().kind() != ResourceKind::Texture {
        return Err(RenderArtifactManifestError::ResourceLayoutKindMismatch {
            resource_kind: manifest.resource().kind(),
            layout_kind: "texture",
        });
    }
    let mip_count = layout.mip_count();
    let array_layer_count = layout.array_layer_count();
    let bootstrap_first_mip = layout.bootstrap_first_mip();
    if layout.width() == 0 || layout.height() == 0 || mip_count == 0 || array_layer_count == 0 {
        return Err(RenderArtifactManifestError::EmptyTextureLayout);
    }
    validate_block_geometry(layout)?;
    validate_mip_count(layout)?;
    if bootstrap_first_mip >= mip_count {
        return Err(RenderArtifactManifestError::TextureBootstrapMipOutOfRange {
            mip_count,
            bootstrap_first_mip,
        });
    }
    let expected_count = u64::from(mip_count)
        .checked_mul(u64::from(array_layer_count))
        .and_then(|count| usize::try_from(count).ok())
        .ok_or(RenderArtifactManifestError::BlockCountOverflow)?;
    if manifest.blocks().len() != expected_count {
        return Err(RenderArtifactManifestError::BlockCountMismatch {
            expected: expected_count,
            actual: manifest.blocks().len(),
        });
    }
    validate_subresources(manifest, layout)
}

fn validate_block_geometry(
    layout: &RenderArtifactTextureLayout,
) -> Result<(), RenderArtifactManifestError> {
    let format = layout.block_format();
    if format.block_width() == 0 || format.block_height() == 0 || format.bytes_per_block() == 0 {
        return Err(RenderArtifactManifestError::InvalidTextureBlockGeometry {
            block_width: format.block_width(),
            block_height: format.block_height(),
            bytes_per_block: format.bytes_per_block(),
        });
    }
    Ok(())
}

fn validate_mip_count(
    layout: &RenderArtifactTextureLayout,
) -> Result<(), RenderArtifactManifestError> {
    let max_dimension = layout.width().max(layout.height());
    let max_mip_count = u32::BITS - max_dimension.leading_zeros();
    if layout.mip_count() > max_mip_count {
        return Err(RenderArtifactManifestError::TextureMipCountOutOfRange {
            width: layout.width(),
            height: layout.height(),
            mip_count: layout.mip_count(),
            max_mip_count,
        });
    }
    Ok(())
}

fn validate_subresources(
    manifest: &RenderArtifactManifest,
    layout: &RenderArtifactTextureLayout,
) -> Result<(), RenderArtifactManifestError> {
    let mut index = 0;
    for mip in 0..layout.mip_count() {
        for layer in 0..layout.array_layer_count() {
            let subresource = RenderSubresourceId::TextureMipLayer { mip, layer };
            let block = block_at(manifest, index, subresource)?;
            let expected_residency = if mip >= layout.bootstrap_first_mip() {
                RenderArtifactResidencyClass::Bootstrap
            } else {
                RenderArtifactResidencyClass::Streamable
            };
            validate_residency(block, expected_residency)?;
            let Some(upload_layout) = layout.subresource_layout(mip, layer) else {
                return Err(
                    RenderArtifactManifestError::TextureSubresourceLayoutOverflow { subresource },
                );
            };
            if block.decoded_bytes() != upload_layout.decoded_bytes() {
                return Err(
                    RenderArtifactManifestError::TextureBlockDecodedSizeMismatch {
                        subresource,
                        expected: upload_layout.decoded_bytes(),
                        actual: block.decoded_bytes(),
                    },
                );
            }
            index += 1;
        }
    }
    Ok(())
}
