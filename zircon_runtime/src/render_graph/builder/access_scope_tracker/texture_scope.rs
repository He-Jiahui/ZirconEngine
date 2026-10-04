use crate::render_graph::{
    RenderGraphError, RenderGraphResource, RenderGraphTextureAspect,
    RenderGraphTextureSubresourceRange,
};
use crate::rhi::{TextureDesc, TextureDimension};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(super) struct TexturePlane {
    pub(super) mip_level: u32,
    pub(super) aspect: RenderGraphTextureAspect,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TextureScope {
    mip_start: u32,
    mip_end: u32,
    layer_start: u32,
    layer_end: u32,
    layer_limit: u32,
    aspects: [RenderGraphTextureAspect; 2],
    aspect_count: usize,
}

impl TextureScope {
    pub(super) fn new(desc: &TextureDesc, range: RenderGraphTextureSubresourceRange) -> Self {
        let mip_end = range
            .mip_level_count
            .map_or(desc.mip_levels, |count| range.base_mip_level + count);
        let layer_end = range
            .array_layer_count
            .map_or(desc.array_layer_count(), |count| {
                range.base_array_layer + count
            });
        let (aspects, aspect_count) = match range.aspect {
            RenderGraphTextureAspect::All if desc.format.has_stencil() => (
                [
                    RenderGraphTextureAspect::Depth,
                    RenderGraphTextureAspect::Stencil,
                ],
                2,
            ),
            RenderGraphTextureAspect::All if desc.format.is_depth() => (
                [
                    RenderGraphTextureAspect::Depth,
                    RenderGraphTextureAspect::Depth,
                ],
                1,
            ),
            RenderGraphTextureAspect::All => (
                [
                    RenderGraphTextureAspect::Color,
                    RenderGraphTextureAspect::Color,
                ],
                1,
            ),
            aspect => ([aspect, aspect], 1),
        };
        Self {
            mip_start: range.base_mip_level,
            mip_end,
            layer_start: range.base_array_layer,
            layer_end,
            layer_limit: desc.array_layer_count(),
            aspects,
            aspect_count,
        }
    }

    pub(super) const fn layer_start(self) -> u64 {
        self.layer_start as u64
    }

    pub(super) const fn layer_end(self) -> u64 {
        self.layer_end as u64
    }

    pub(super) const fn layer_limit(self) -> u64 {
        self.layer_limit as u64
    }

    pub(super) fn planes(self) -> impl Iterator<Item = TexturePlane> {
        let aspects = self.aspects;
        let aspect_count = self.aspect_count;
        (self.mip_start..self.mip_end).flat_map(move |mip_level| {
            (0..aspect_count).map(move |aspect_index| TexturePlane {
                mip_level,
                aspect: aspects[aspect_index],
            })
        })
    }
}

/// Reject malformed shapes before a texture access can create scope history.
pub(super) fn validate_texture_descriptor_for_tracking(
    resource: RenderGraphResource,
    desc: &TextureDesc,
) -> Result<(), RenderGraphError> {
    if let Some(reason) = desc.shape_validation_error() {
        return Err(RenderGraphError::TextureDescriptorInvalid {
            resource: format!("{resource:?}"),
            reason: reason.to_owned(),
        });
    }
    if desc.sample_count == 0 {
        return Err(RenderGraphError::TextureDescriptorInvalid {
            resource: format!("{resource:?}"),
            reason: "sample_count must be greater than zero".to_owned(),
        });
    }
    if desc.sample_count > 1 && desc.dimension != TextureDimension::D2 {
        return Err(RenderGraphError::TextureDescriptorInvalid {
            resource: format!("{resource:?}"),
            reason: "multisampling is only valid for 2D textures".to_owned(),
        });
    }
    if desc.sample_count > 1 && desc.mip_levels > 1 {
        return Err(RenderGraphError::TextureDescriptorInvalid {
            resource: format!("{resource:?}"),
            reason: "multisampled textures cannot declare mip levels".to_owned(),
        });
    }
    if !desc.mip_levels_fit_shape() {
        return Err(RenderGraphError::TextureDescriptorInvalid {
            resource: format!("{resource:?}"),
            reason: "mip_levels must fit the texture extent".to_owned(),
        });
    }
    Ok(())
}
