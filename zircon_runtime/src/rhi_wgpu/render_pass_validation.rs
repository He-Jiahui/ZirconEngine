use std::collections::BTreeSet;

use crate::rhi::{
    PipelineDesc, RenderPassColorAttachmentDesc, RenderPassColorLoadOp, RenderPassDepthLoadOp,
    RenderPassDepthStencilAttachmentDesc, RenderScissorRect, RenderViewportDesc, RhiError,
    TextureDesc, TextureDimension, TextureHandle, TextureUsage,
};

use super::device::DeterministicRhiContractDeviceState;
use super::resource_validation::ensure_texture_usage;

#[derive(Clone, Debug)]
pub(super) struct ActiveRenderPass {
    color_attachments: Vec<TextureHandle>,
    depth_stencil_attachment: Option<TextureHandle>,
    extent_width: u32,
    extent_height: u32,
    sample_count: u32,
}

impl ActiveRenderPass {
    pub(super) fn new(
        color_attachments: &[RenderPassColorAttachmentDesc],
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDesc>,
        attachment_info: RenderPassAttachmentInfo,
    ) -> Self {
        Self {
            color_attachments: color_attachments
                .iter()
                .map(|attachment| attachment.view.texture)
                .collect(),
            depth_stencil_attachment: depth_stencil_attachment
                .map(|attachment| attachment.view.texture),
            extent_width: attachment_info.width,
            extent_height: attachment_info.height,
            sample_count: attachment_info.sample_count,
        }
    }

    pub(super) fn validate_pipeline_attachments(
        &self,
        state: &DeterministicRhiContractDeviceState,
        pipeline: &PipelineDesc,
    ) -> Result<(), RhiError> {
        let raster_state =
            pipeline
                .raster_state
                .as_ref()
                .ok_or_else(|| RhiError::InvalidRasterDraw {
                    reason: "bound raster pipeline has no raster state".to_string(),
                })?;

        if self.color_attachments.len() != raster_state.color_targets.len() {
            return Err(RhiError::InvalidRenderPass {
                reason: format!(
                    "render pass declares {} color attachments but pipeline expects {}",
                    self.color_attachments.len(),
                    raster_state.color_targets.len()
                ),
            });
        }

        for (index, (texture, color_target)) in self
            .color_attachments
            .iter()
            .zip(&raster_state.color_targets)
            .enumerate()
        {
            let texture_desc = state.texture_desc_ref(*texture)?;
            if texture_desc.format != color_target.format {
                return Err(RhiError::InvalidRenderPass {
                    reason: format!(
                        "color attachment {index} format {:?} does not match pipeline target {:?}",
                        texture_desc.format, color_target.format
                    ),
                });
            }
        }

        if raster_state.sample_count != self.sample_count {
            return Err(RhiError::InvalidRenderPass {
                reason: format!(
                    "render pass sample_count {} does not match raster pipeline sample_count {}",
                    self.sample_count, raster_state.sample_count
                ),
            });
        }

        match (self.depth_stencil_attachment, raster_state.depth_stencil) {
            (Some(texture), Some(depth_stencil)) => {
                let texture_desc = state.texture_desc_ref(texture)?;
                if texture_desc.format != depth_stencil.format {
                    return Err(RhiError::InvalidRenderPass {
                        reason: format!(
                            "depth/stencil attachment format {:?} does not match pipeline target {:?}",
                            texture_desc.format, depth_stencil.format
                        ),
                    });
                }
            }
            (None, Some(_)) => {
                return Err(RhiError::InvalidRenderPass {
                    reason: "raster pipeline expects a depth/stencil attachment".to_string(),
                });
            }
            (Some(_), None) => {
                return Err(RhiError::InvalidRenderPass {
                    reason: "render pass declares a depth/stencil attachment but pipeline does not"
                        .to_string(),
                });
            }
            (None, None) => {}
        }

        Ok(())
    }

    pub(super) fn validate_viewport(&self, viewport: RenderViewportDesc) -> Result<(), RhiError> {
        if !viewport.x.is_finite()
            || !viewport.y.is_finite()
            || !viewport.width.is_finite()
            || !viewport.height.is_finite()
            || !viewport.min_depth.is_finite()
            || !viewport.max_depth.is_finite()
        {
            return Err(RhiError::InvalidRenderPass {
                reason: "viewport values must be finite".to_string(),
            });
        }
        if viewport.width <= 0.0 || viewport.height <= 0.0 {
            return Err(RhiError::InvalidRenderPass {
                reason: "viewport width and height must be greater than zero".to_string(),
            });
        }
        if viewport.x < 0.0 || viewport.y < 0.0 {
            return Err(RhiError::InvalidRenderPass {
                reason: "viewport origin must be non-negative".to_string(),
            });
        }
        if viewport.min_depth < 0.0
            || viewport.min_depth > 1.0
            || viewport.max_depth < 0.0
            || viewport.max_depth > 1.0
            || viewport.min_depth > viewport.max_depth
        {
            return Err(RhiError::InvalidRenderPass {
                reason:
                    "viewport depth range must stay within 0.0..=1.0 and min_depth must not exceed max_depth"
                        .to_string(),
            });
        }
        let right = f64::from(viewport.x) + f64::from(viewport.width);
        let bottom = f64::from(viewport.y) + f64::from(viewport.height);
        if !right.is_finite()
            || !bottom.is_finite()
            || right > f64::from(self.extent_width)
            || bottom > f64::from(self.extent_height)
        {
            return Err(RhiError::InvalidRenderPass {
                reason: format!(
                    "viewport exceeds render pass extent {}x{}",
                    self.extent_width, self.extent_height
                ),
            });
        }
        Ok(())
    }

    pub(super) fn validate_scissor_rect(&self, rect: RenderScissorRect) -> Result<(), RhiError> {
        if rect.width == 0 || rect.height == 0 {
            return Err(RhiError::InvalidRenderPass {
                reason: "scissor width and height must be greater than zero".to_string(),
            });
        }
        let Some(right) = rect.x.checked_add(rect.width) else {
            return Err(scissor_exceeds_extent_error(
                self.extent_width,
                self.extent_height,
            ));
        };
        let Some(bottom) = rect.y.checked_add(rect.height) else {
            return Err(scissor_exceeds_extent_error(
                self.extent_width,
                self.extent_height,
            ));
        };
        if right > self.extent_width || bottom > self.extent_height {
            return Err(scissor_exceeds_extent_error(
                self.extent_width,
                self.extent_height,
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RenderPassAttachmentInfo {
    width: u32,
    height: u32,
    sample_count: u32,
}

impl RenderPassAttachmentInfo {
    fn validate_matches(
        &self,
        role: &str,
        view_shape: RenderPassAttachmentInfo,
    ) -> Result<(), RhiError> {
        if view_shape.width != self.width || view_shape.height != self.height {
            return Err(RhiError::InvalidRenderPass {
                reason: format!(
                    "{role} extent {}x{} does not match render pass extent {}x{}",
                    view_shape.width, view_shape.height, self.width, self.height
                ),
            });
        }
        if view_shape.sample_count != self.sample_count {
            return Err(RhiError::InvalidRenderPass {
                reason: format!(
                    "{role} sample_count {} does not match render pass sample_count {}",
                    view_shape.sample_count, self.sample_count
                ),
            });
        }
        Ok(())
    }

    fn validate_resolve_target(
        &self,
        index: usize,
        color_desc: &TextureDesc,
        resolve_desc: &TextureDesc,
        resolve_view_shape: RenderPassAttachmentInfo,
    ) -> Result<(), RhiError> {
        if color_desc.sample_count <= 1 {
            return Err(RhiError::InvalidRenderPass {
                reason: format!(
                    "color attachment {index} resolve target requires a multisampled color attachment"
                ),
            });
        }
        if resolve_desc.sample_count != 1 {
            return Err(RhiError::InvalidRenderPass {
                reason: format!("color attachment {index} resolve target must be single-sampled"),
            });
        }
        if resolve_desc.format != color_desc.format {
            return Err(RhiError::InvalidRenderPass {
                reason: format!(
                    "color attachment {index} resolve target format {:?} does not match color attachment format {:?}",
                    resolve_desc.format, color_desc.format
                ),
            });
        }
        if resolve_view_shape.width != self.width || resolve_view_shape.height != self.height {
            return Err(RhiError::InvalidRenderPass {
                reason: format!(
                    "color attachment {index} resolve target extent {}x{} does not match render pass extent {}x{}",
                    resolve_view_shape.width, resolve_view_shape.height, self.width, self.height
                ),
            });
        }
        Ok(())
    }
}

pub(super) fn validate_render_pass_attachments(
    state: &DeterministicRhiContractDeviceState,
    color_attachments: &[RenderPassColorAttachmentDesc],
    depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDesc>,
) -> Result<RenderPassAttachmentInfo, RhiError> {
    if color_attachments.is_empty() && depth_stencil_attachment.is_none() {
        return Err(RhiError::InvalidRenderPass {
            reason: "render pass requires at least one color or depth/stencil attachment"
                .to_string(),
        });
    }

    let mut seen = BTreeSet::new();
    let mut attachment_info = None;
    for (index, attachment) in color_attachments.iter().enumerate() {
        if !seen.insert((
            attachment.view.texture.raw(),
            attachment.view.mip_level,
            attachment.view.array_layer,
        )) {
            return Err(RhiError::InvalidRenderPass {
                reason: format!(
                    "texture `{}` mip {} layer {} is bound more than once in the render pass",
                    attachment.view.texture.raw(),
                    attachment.view.mip_level,
                    attachment.view.array_layer
                ),
            });
        }
        let desc = state.texture_desc_ref(attachment.view.texture)?;
        ensure_texture_usage(
            attachment.view.texture.raw(),
            desc,
            TextureUsage::RENDER_ATTACHMENT,
        )?;
        validate_color_attachment_desc(index, attachment, desc)?;
        let view_shape = validate_attachment_view(
            &format!("color attachment {index}"),
            desc,
            attachment.view.mip_level,
            attachment.view.array_layer,
        )?;
        validate_attachment_info(
            &mut attachment_info,
            &format!("color attachment {index}"),
            view_shape,
        )?;
        let info = attachment_info.expect("color attachment info was just initialized");
        if let Some(resolve_target) = attachment.resolve_target {
            if !seen.insert((
                resolve_target.texture.raw(),
                resolve_target.mip_level,
                resolve_target.array_layer,
            )) {
                return Err(RhiError::InvalidRenderPass {
                    reason: format!(
                        "texture `{}` mip {} layer {} is bound more than once in the render pass",
                        resolve_target.texture.raw(),
                        resolve_target.mip_level,
                        resolve_target.array_layer
                    ),
                });
            }
            let resolve_desc = state.texture_desc_ref(resolve_target.texture)?;
            ensure_texture_usage(
                resolve_target.texture.raw(),
                resolve_desc,
                TextureUsage::RENDER_ATTACHMENT,
            )?;
            validate_color_resolve_target_desc(index, resolve_desc)?;
            let resolve_view_shape = validate_attachment_view(
                &format!("color attachment {index} resolve target"),
                resolve_desc,
                resolve_target.mip_level,
                resolve_target.array_layer,
            )?;
            info.validate_resolve_target(index, desc, resolve_desc, resolve_view_shape)?;
        }
    }

    if let Some(depth_stencil) = depth_stencil_attachment {
        if !seen.insert((
            depth_stencil.view.texture.raw(),
            depth_stencil.view.mip_level,
            depth_stencil.view.array_layer,
        )) {
            return Err(RhiError::InvalidRenderPass {
                reason: format!(
                    "texture `{}` mip {} layer {} is bound more than once in the render pass",
                    depth_stencil.view.texture.raw(),
                    depth_stencil.view.mip_level,
                    depth_stencil.view.array_layer
                ),
            });
        }
        let desc = state.texture_desc_ref(depth_stencil.view.texture)?;
        ensure_texture_usage(
            depth_stencil.view.texture.raw(),
            desc,
            TextureUsage::RENDER_ATTACHMENT,
        )?;
        validate_depth_stencil_attachment_desc(desc, depth_stencil)?;
        let view_shape = validate_attachment_view(
            "depth/stencil attachment",
            desc,
            depth_stencil.view.mip_level,
            depth_stencil.view.array_layer,
        )?;
        validate_attachment_info(&mut attachment_info, "depth/stencil attachment", view_shape)?;
    }

    attachment_info.ok_or_else(|| RhiError::InvalidRenderPass {
        reason: "render pass requires at least one color or depth/stencil attachment".to_string(),
    })
}

fn validate_color_attachment_desc(
    index: usize,
    attachment: &RenderPassColorAttachmentDesc,
    desc: &TextureDesc,
) -> Result<(), RhiError> {
    if desc.format.is_depth() {
        return Err(RhiError::InvalidRenderPass {
            reason: format!("color attachment {index} must use a color texture format"),
        });
    }
    if desc.sample_count == 0 {
        return Err(RhiError::InvalidRenderPass {
            reason: format!("color attachment {index} sample_count must be greater than zero"),
        });
    }
    if let RenderPassColorLoadOp::Clear(color) = attachment.load {
        if !color.is_finite() {
            return Err(RhiError::InvalidRenderPass {
                reason: format!("color attachment {index} clear color values must be finite"),
            });
        }
    }
    Ok(())
}

fn validate_color_resolve_target_desc(index: usize, desc: &TextureDesc) -> Result<(), RhiError> {
    if desc.format.is_depth() {
        return Err(RhiError::InvalidRenderPass {
            reason: format!(
                "color attachment {index} resolve target must use a color texture format"
            ),
        });
    }
    if desc.sample_count == 0 {
        return Err(RhiError::InvalidRenderPass {
            reason: format!(
                "color attachment {index} resolve target sample_count must be greater than zero"
            ),
        });
    }
    Ok(())
}

fn validate_depth_stencil_attachment_desc(
    desc: &TextureDesc,
    attachment: RenderPassDepthStencilAttachmentDesc,
) -> Result<(), RhiError> {
    if !desc.format.is_depth() {
        return Err(RhiError::InvalidRenderPass {
            reason: "depth/stencil attachment must use a depth texture format".to_string(),
        });
    }
    if attachment.stencil_load.is_some() != attachment.stencil_store.is_some() {
        return Err(RhiError::InvalidRenderPass {
            reason: "stencil load/store operations must be declared together".to_string(),
        });
    }
    if attachment.stencil_load.is_some() && !desc.format.has_stencil() {
        return Err(RhiError::InvalidRenderPass {
            reason: "stencil operations require a stencil-capable depth format".to_string(),
        });
    }
    if let RenderPassDepthLoadOp::Clear(value) = attachment.depth_load {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(RhiError::InvalidRenderPass {
                reason: "depth clear value must stay within 0.0..=1.0".to_string(),
            });
        }
    }
    Ok(())
}

fn validate_attachment_view(
    role: &str,
    desc: &TextureDesc,
    mip_level: u32,
    array_layer: u32,
) -> Result<RenderPassAttachmentInfo, RhiError> {
    if mip_level >= desc.mip_levels {
        return Err(RhiError::InvalidRenderPass {
            reason: format!(
                "{role} mip level {mip_level} is outside texture mip_levels {}",
                desc.mip_levels
            ),
        });
    }
    let layer_count = texture_view_layer_count(desc);
    if array_layer >= layer_count {
        return Err(RhiError::InvalidRenderPass {
            reason: format!(
                "{role} array layer {array_layer} is outside texture layer count {layer_count}"
            ),
        });
    }

    Ok(RenderPassAttachmentInfo {
        width: mip_extent(desc.width, mip_level),
        height: match desc.dimension {
            TextureDimension::D1 => 1,
            _ => mip_extent(desc.height, mip_level),
        },
        sample_count: desc.sample_count,
    })
}

fn texture_view_layer_count(desc: &TextureDesc) -> u32 {
    match desc.dimension {
        TextureDimension::D1 | TextureDimension::D2 | TextureDimension::D3 => 1,
        TextureDimension::D2Array | TextureDimension::Cube => desc.depth,
    }
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    let shifted = if level >= u32::BITS {
        0
    } else {
        value >> level
    };
    if shifted == 0 {
        1
    } else {
        shifted
    }
}

fn validate_attachment_info(
    attachment_info: &mut Option<RenderPassAttachmentInfo>,
    role: &str,
    view_shape: RenderPassAttachmentInfo,
) -> Result<(), RhiError> {
    match attachment_info {
        Some(info) => info.validate_matches(role, view_shape),
        None => {
            *attachment_info = Some(view_shape);
            Ok(())
        }
    }
}

fn scissor_exceeds_extent_error(extent_width: u32, extent_height: u32) -> RhiError {
    RhiError::InvalidRenderPass {
        reason: format!(
            "scissor rectangle exceeds render pass extent {extent_width}x{extent_height}"
        ),
    }
}
