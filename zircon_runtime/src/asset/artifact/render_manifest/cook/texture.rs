use std::ops::Range;
use std::sync::Arc;

use thiserror::Error;

use crate::asset::{
    TextureAsset, TexturePayload, TextureUploadPlan, TextureUploadReadiness, TextureUploadSupport,
};
use crate::core::framework::render::RenderImageDimension;
use crate::core::resource::{ResourceKind, UntypedResourceHandle};

use super::super::{
    RenderArtifactBlockCodec, RenderArtifactBlockDescriptor, RenderArtifactContentId,
    RenderArtifactLayout, RenderArtifactManifest, RenderArtifactManifestError,
    RenderArtifactResidencyClass, RenderArtifactTextureBlockFormat, RenderArtifactTextureLayout,
    RenderSubresourceId,
};
use super::output::{RenderArtifactCookOutput, RenderArtifactCookedBlock};

#[derive(Clone, Debug)]
pub struct RenderArtifactTextureCookSettings {
    target_platform: Arc<str>,
    bootstrap_first_mip: u32,
    block_alignment: u32,
    upload_support: TextureUploadSupport,
}

impl RenderArtifactTextureCookSettings {
    pub fn new(
        target_platform: Arc<str>,
        bootstrap_first_mip: u32,
        block_alignment: u32,
        upload_support: TextureUploadSupport,
    ) -> Self {
        Self {
            target_platform,
            bootstrap_first_mip,
            block_alignment,
            upload_support,
        }
    }

    pub fn target_platform(&self) -> &str {
        self.target_platform.as_ref()
    }

    pub const fn bootstrap_first_mip(&self) -> u32 {
        self.bootstrap_first_mip
    }

    pub const fn block_alignment(&self) -> u32 {
        self.block_alignment
    }

    pub const fn upload_support(&self) -> TextureUploadSupport {
        self.upload_support
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RenderArtifactTextureCookError {
    #[error("texture render artifact cook requires a Texture resource, found {actual:?}")]
    ResourceKindMismatch { actual: ResourceKind },
    #[error("texture render artifact cook does not support {dimension:?} payloads")]
    UnsupportedDimension { dimension: RenderImageDimension },
    #[error(
        "texture render artifact cook requires 2D compression blocks, found depth {block_depth}"
    )]
    UnsupportedBlockDepth { block_depth: u32 },
    #[error("texture render artifact payload is not upload-ready: {reason}")]
    UploadUnsupported { reason: String },
    #[error("texture render artifact upload plan duplicates mip {mip} layer {layer}")]
    DuplicateSubresource { mip: u32, layer: u32 },
    #[error("texture render artifact upload plan contains out-of-range mip {mip} layer {layer}")]
    SubresourceOutOfRange { mip: u32, layer: u32 },
    #[error("texture render artifact subresource count does not fit the current platform")]
    SubresourceCountOverflow,
    #[error("texture render artifact upload plan is missing mip {mip} layer {layer}")]
    MissingSubresource { mip: u32, layer: u32 },
    #[error("texture render artifact payload range for mip {mip} layer {layer} overflows")]
    SubresourceRangeOverflow { mip: u32, layer: u32 },
    #[error(
        "texture render artifact payload range {start}..{end} for mip {mip} layer {layer} exceeds {payload_bytes} bytes"
    )]
    SubresourceRangeOutOfBounds {
        mip: u32,
        layer: u32,
        start: usize,
        end: usize,
        payload_bytes: usize,
    },
    #[error(
        "texture render artifact mip {mip} layer {layer} has {actual} bytes but tight layout requires {expected}"
    )]
    SubresourceByteCountMismatch {
        mip: u32,
        layer: u32,
        expected: u64,
        actual: usize,
    },
    #[error(transparent)]
    Manifest(#[from] RenderArtifactManifestError),
}

pub fn cook_texture_render_artifact(
    resource: UntypedResourceHandle,
    asset_revision: u64,
    texture: TextureAsset,
    settings: RenderArtifactTextureCookSettings,
) -> Result<RenderArtifactCookOutput, RenderArtifactTextureCookError> {
    if resource.kind() != ResourceKind::Texture {
        return Err(RenderArtifactTextureCookError::ResourceKindMismatch {
            actual: resource.kind(),
        });
    }
    let descriptor = texture.render_image_descriptor();
    if !matches!(
        descriptor.dimension,
        RenderImageDimension::D2 | RenderImageDimension::Cube
    ) {
        return Err(RenderArtifactTextureCookError::UnsupportedDimension {
            dimension: descriptor.dimension,
        });
    }
    let plan = match texture.upload_readiness(settings.upload_support()) {
        TextureUploadReadiness::Ready { plan } => plan,
        TextureUploadReadiness::Unsupported { reason } => {
            return Err(RenderArtifactTextureCookError::UploadUnsupported { reason });
        }
    };
    if plan.block_depth != 1 {
        return Err(RenderArtifactTextureCookError::UnsupportedBlockDepth {
            block_depth: plan.block_depth,
        });
    }

    let width = texture.width;
    let height = texture.height;
    let mip_count = descriptor.mip_count.max(1);
    let array_layer_count = descriptor.array_layer_count.max(1);
    let layout = RenderArtifactTextureLayout::new(
        RenderArtifactTextureBlockFormat::new(
            Arc::from(plan.format.as_str()),
            plan.block_width,
            plan.block_height,
            plan.bytes_per_block,
        ),
        width,
        height,
        mip_count,
        array_layer_count,
        settings.bootstrap_first_mip(),
    );
    let ranges = subresource_ranges(&plan, &layout)?;
    let payload = texture_payload(texture);
    let mut cooked_blocks = Vec::with_capacity(ranges.len());
    let mut manifest_blocks = Vec::with_capacity(ranges.len());
    for mip in 0..mip_count {
        for layer in 0..array_layer_count {
            let slot = subresource_slot(&layout, mip, layer)
                .ok_or(RenderArtifactTextureCookError::SubresourceCountOverflow)?;
            let range = ranges
                .get(slot)
                .and_then(Clone::clone)
                .ok_or(RenderArtifactTextureCookError::MissingSubresource { mip, layer })?;
            validate_payload_range(&payload, &layout, mip, layer, &range)?;
            let bytes = &payload[range.clone()];
            let subresource = RenderSubresourceId::TextureMipLayer { mip, layer };
            let dependencies = (mip + 1 < mip_count)
                .then_some(RenderSubresourceId::TextureMipLayer {
                    mip: mip + 1,
                    layer,
                })
                .into_iter()
                .collect();
            let residency = if mip >= settings.bootstrap_first_mip() {
                RenderArtifactResidencyClass::Bootstrap
            } else {
                RenderArtifactResidencyClass::Streamable
            };
            let byte_count = bytes.len() as u64;
            let block = RenderArtifactBlockDescriptor::new(
                subresource,
                RenderArtifactContentId::from_bytes(*blake3::hash(bytes).as_bytes()),
                RenderArtifactBlockCodec::Raw,
                byte_count,
                byte_count,
                settings.block_alignment(),
                Arc::from(plan.format.as_str()),
                residency,
                dependencies,
            );
            manifest_blocks.push(block.clone());
            cooked_blocks.push(RenderArtifactCookedBlock::new(
                block,
                Arc::clone(&payload),
                range,
            ));
        }
    }
    let manifest = RenderArtifactManifest::new(
        resource,
        asset_revision,
        settings.target_platform,
        RenderArtifactLayout::texture(layout),
        Vec::new(),
        manifest_blocks,
    )?;
    Ok(RenderArtifactCookOutput::new(manifest, cooked_blocks))
}

fn texture_payload(texture: TextureAsset) -> Arc<Vec<u8>> {
    match texture.payload {
        TexturePayload::Rgba8 => Arc::new(texture.rgba),
        TexturePayload::Container { bytes, .. } => Arc::new(bytes),
    }
}

fn subresource_ranges(
    plan: &TextureUploadPlan,
    layout: &RenderArtifactTextureLayout,
) -> Result<Vec<Option<Range<usize>>>, RenderArtifactTextureCookError> {
    if plan.subresources.is_empty() {
        return contiguous_subresource_ranges(plan.data_offset, layout);
    }
    let mut ranges = vec![None; subresource_count(layout)?];
    for subresource in &plan.subresources {
        let slot = subresource_slot(layout, subresource.mip_level, subresource.array_layer).ok_or(
            RenderArtifactTextureCookError::SubresourceOutOfRange {
                mip: subresource.mip_level,
                layer: subresource.array_layer,
            },
        )?;
        let end = subresource
            .data_offset
            .checked_add(subresource.data_length)
            .ok_or(RenderArtifactTextureCookError::SubresourceRangeOverflow {
                mip: subresource.mip_level,
                layer: subresource.array_layer,
            })?;
        if ranges[slot].replace(subresource.data_offset..end).is_some() {
            return Err(RenderArtifactTextureCookError::DuplicateSubresource {
                mip: subresource.mip_level,
                layer: subresource.array_layer,
            });
        }
    }
    Ok(ranges)
}

fn contiguous_subresource_ranges(
    data_offset: usize,
    layout: &RenderArtifactTextureLayout,
) -> Result<Vec<Option<Range<usize>>>, RenderArtifactTextureCookError> {
    let mut ranges = Vec::with_capacity(subresource_count(layout)?);
    let mut offset = data_offset;
    for mip in 0..layout.mip_count() {
        for layer in 0..layout.array_layer_count() {
            let byte_count = layout
                .subresource_layout(mip, layer)
                .and_then(|layout| usize::try_from(layout.decoded_bytes()).ok())
                .ok_or(RenderArtifactTextureCookError::SubresourceRangeOverflow { mip, layer })?;
            let end = offset
                .checked_add(byte_count)
                .ok_or(RenderArtifactTextureCookError::SubresourceRangeOverflow { mip, layer })?;
            ranges.push(Some(offset..end));
            offset = end;
        }
    }
    Ok(ranges)
}

fn subresource_count(
    layout: &RenderArtifactTextureLayout,
) -> Result<usize, RenderArtifactTextureCookError> {
    layout
        .mip_count()
        .checked_mul(layout.array_layer_count())
        .and_then(|count| usize::try_from(count).ok())
        .ok_or(RenderArtifactTextureCookError::SubresourceCountOverflow)
}

fn subresource_slot(layout: &RenderArtifactTextureLayout, mip: u32, layer: u32) -> Option<usize> {
    if mip >= layout.mip_count() || layer >= layout.array_layer_count() {
        return None;
    }
    mip.checked_mul(layout.array_layer_count())
        .and_then(|base| base.checked_add(layer))
        .and_then(|slot| usize::try_from(slot).ok())
}

fn validate_payload_range(
    payload: &[u8],
    layout: &RenderArtifactTextureLayout,
    mip: u32,
    layer: u32,
    range: &Range<usize>,
) -> Result<(), RenderArtifactTextureCookError> {
    if range.end > payload.len() || range.start > range.end {
        return Err(
            RenderArtifactTextureCookError::SubresourceRangeOutOfBounds {
                mip,
                layer,
                start: range.start,
                end: range.end,
                payload_bytes: payload.len(),
            },
        );
    }
    let expected = layout
        .subresource_layout(mip, layer)
        .map(|layout| layout.decoded_bytes())
        .ok_or(RenderArtifactTextureCookError::SubresourceRangeOverflow { mip, layer })?;
    let actual = range.end - range.start;
    if actual as u64 != expected {
        return Err(
            RenderArtifactTextureCookError::SubresourceByteCountMismatch {
                mip,
                layer,
                expected,
                actual,
            },
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "texture/tests.rs"]
mod tests;
