use std::collections::BTreeSet;

use crate::rhi::{
    BindGroupLayoutHandle, PipelineDesc, PipelineKind, PipelineLayoutDesc, PipelineLayoutHandle,
    RasterPipelineStateDesc, RhiError, ShaderModuleDesc, ShaderModuleHandle, ShaderStage,
    VertexBufferLayoutDesc, VertexInputLayoutDesc,
};

pub(super) trait PipelineResourceLookup {
    fn bind_group_layout_exists(&self, handle: BindGroupLayoutHandle) -> bool;
    fn pipeline_layout_desc(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<&PipelineLayoutDesc, RhiError>;
    fn shader_module_desc(&self, handle: ShaderModuleHandle)
        -> Result<&ShaderModuleDesc, RhiError>;
}

pub(super) fn validate_shader_module_desc(desc: &ShaderModuleDesc) -> Result<(), RhiError> {
    if desc.source.trim().is_empty() {
        return Err(RhiError::InvalidShaderModuleDescriptor {
            label: desc.label.clone(),
            reason: "shader source must not be empty".to_string(),
        });
    }
    if desc.entry_point.trim().is_empty() {
        return Err(RhiError::InvalidShaderModuleDescriptor {
            label: desc.label.clone(),
            reason: "shader entry point must not be empty".to_string(),
        });
    }
    Ok(())
}

pub(super) fn validate_pipeline_layout_desc(
    lookup: &impl PipelineResourceLookup,
    desc: &PipelineLayoutDesc,
) -> Result<(), RhiError> {
    let mut seen = BTreeSet::new();
    for layout in &desc.bind_group_layouts {
        if !lookup.bind_group_layout_exists(*layout) {
            return Err(RhiError::UnknownBindGroupLayout(layout.raw()));
        }
        if !seen.insert(layout.raw()) {
            return Err(RhiError::InvalidPipelineLayoutDescriptor {
                label: desc.label.clone(),
                reason: format!("duplicate bind group layout `{}`", layout.raw()),
            });
        }
    }

    Ok(())
}

pub(super) fn validate_pipeline_desc(
    lookup: &impl PipelineResourceLookup,
    desc: &PipelineDesc,
) -> Result<(), RhiError> {
    let layout = desc
        .layout
        .ok_or_else(|| RhiError::InvalidPipelineDescriptor {
            label: desc.label.clone(),
            reason: "pipeline must reference a pipeline layout".to_string(),
        })?;
    lookup.pipeline_layout_desc(layout)?;

    match desc.kind {
        PipelineKind::Raster => {
            require_shader_stage(
                lookup,
                desc,
                desc.vertex_shader,
                ShaderStage::Vertex,
                "raster pipeline requires a vertex shader",
            )?;
            reject_shader(
                desc,
                desc.compute_shader,
                "raster pipeline must not reference a compute shader",
            )?;
            let Some(raster_state) = desc.raster_state.as_ref() else {
                require_shader_stage(
                    lookup,
                    desc,
                    desc.fragment_shader,
                    ShaderStage::Fragment,
                    "raster pipeline requires a fragment shader",
                )?;
                return Err(RhiError::InvalidPipelineDescriptor {
                    label: desc.label.clone(),
                    reason: "raster pipeline requires raster state".to_string(),
                });
            };
            validate_raster_pipeline_state(desc, raster_state)?;
            if raster_state.color_targets.is_empty() {
                if desc.fragment_shader.is_some() {
                    require_shader_stage(
                        lookup,
                        desc,
                        desc.fragment_shader,
                        ShaderStage::Fragment,
                        "depth-only raster pipeline fragment shader is invalid",
                    )?;
                }
                Ok(())
            } else {
                require_shader_stage(
                    lookup,
                    desc,
                    desc.fragment_shader,
                    ShaderStage::Fragment,
                    "raster pipeline requires a fragment shader",
                )
            }
        }
        PipelineKind::Compute => {
            require_shader_stage(
                lookup,
                desc,
                desc.compute_shader,
                ShaderStage::Compute,
                "compute pipeline requires a compute shader",
            )?;
            reject_shader(
                desc,
                desc.vertex_shader,
                "compute pipeline must not reference a vertex shader",
            )?;
            reject_shader(
                desc,
                desc.fragment_shader,
                "compute pipeline must not reference a fragment shader",
            )?;
            if desc.raster_state.is_some() {
                return Err(RhiError::InvalidPipelineDescriptor {
                    label: desc.label.clone(),
                    reason: "compute pipeline must not declare raster state".to_string(),
                });
            }
            Ok(())
        }
        PipelineKind::RayTracing => Err(RhiError::InvalidPipelineDescriptor {
            label: desc.label.clone(),
            reason: "ray tracing pipelines are not supported by the WGPU backend contract yet"
                .to_string(),
        }),
    }
}

fn validate_raster_pipeline_state(
    pipeline: &PipelineDesc,
    raster_state: &RasterPipelineStateDesc,
) -> Result<(), RhiError> {
    if raster_state.sample_count == 0 {
        return Err(RhiError::InvalidPipelineDescriptor {
            label: pipeline.label.clone(),
            reason: "raster pipeline sample_count must be greater than zero".to_string(),
        });
    }
    if raster_state.color_targets.is_empty() && raster_state.depth_stencil.is_none() {
        return Err(RhiError::InvalidPipelineDescriptor {
            label: pipeline.label.clone(),
            reason: "raster pipeline requires at least one color target or depth/stencil target"
                .to_string(),
        });
    }
    for (index, target) in raster_state.color_targets.iter().enumerate() {
        if target.format.is_depth() {
            return Err(RhiError::InvalidPipelineDescriptor {
                label: pipeline.label.clone(),
                reason: format!("color target {index} must use a color format"),
            });
        }
        if target.write_mask == crate::rhi::ColorWriteMask::NONE {
            return Err(RhiError::InvalidPipelineDescriptor {
                label: pipeline.label.clone(),
                reason: format!("color target {index} write mask must not be empty"),
            });
        }
        if target.write_mask.has_unknown_bits() {
            return Err(RhiError::InvalidPipelineDescriptor {
                label: pipeline.label.clone(),
                reason: format!("color target {index} write mask contains unknown bits"),
            });
        }
    }
    if let Some(depth_stencil) = raster_state.depth_stencil {
        if !depth_stencil.format.is_depth() {
            return Err(RhiError::InvalidPipelineDescriptor {
                label: pipeline.label.clone(),
                reason: "depth/stencil target must use a depth format".to_string(),
            });
        }
        if depth_stencil.stencil_enabled && !depth_stencil.format.has_stencil() {
            return Err(RhiError::InvalidPipelineDescriptor {
                label: pipeline.label.clone(),
                reason: "stencil state requires a stencil-capable depth format".to_string(),
            });
        }
    }
    validate_vertex_input_layout(pipeline, &raster_state.vertex_input)?;
    Ok(())
}

fn validate_vertex_input_layout(
    pipeline: &PipelineDesc,
    vertex_input: &VertexInputLayoutDesc,
) -> Result<(), RhiError> {
    let mut seen_locations = BTreeSet::new();
    for (buffer_index, buffer) in vertex_input.buffers.iter().enumerate() {
        validate_vertex_buffer_layout(pipeline, buffer_index, buffer, &mut seen_locations)?;
    }
    Ok(())
}

fn validate_vertex_buffer_layout(
    pipeline: &PipelineDesc,
    buffer_index: usize,
    buffer: &VertexBufferLayoutDesc,
    seen_locations: &mut BTreeSet<u32>,
) -> Result<(), RhiError> {
    if buffer.attributes.is_empty() {
        return Err(RhiError::InvalidPipelineDescriptor {
            label: pipeline.label.clone(),
            reason: format!("vertex buffer layout {buffer_index} must declare attributes"),
        });
    }
    if buffer.array_stride == 0 {
        return Err(RhiError::InvalidPipelineDescriptor {
            label: pipeline.label.clone(),
            reason: format!("vertex buffer layout {buffer_index} stride must be greater than zero"),
        });
    }
    for (attribute_index, attribute) in buffer.attributes.iter().enumerate() {
        if !seen_locations.insert(attribute.shader_location) {
            return Err(RhiError::InvalidPipelineDescriptor {
                label: pipeline.label.clone(),
                reason: format!(
                    "vertex attribute shader location {} is declared more than once",
                    attribute.shader_location
                ),
            });
        }
        let Some(end) = attribute.offset.checked_add(attribute.format.size_bytes()) else {
            return Err(RhiError::InvalidPipelineDescriptor {
                label: pipeline.label.clone(),
                reason: format!(
                    "vertex attribute {attribute_index} in buffer {buffer_index} byte range overflows"
                ),
            });
        };
        if end > buffer.array_stride {
            return Err(RhiError::InvalidPipelineDescriptor {
                label: pipeline.label.clone(),
                reason: format!(
                    "vertex attribute {attribute_index} in buffer {buffer_index} exceeds array stride"
                ),
            });
        }
    }
    Ok(())
}

fn require_shader_stage(
    lookup: &impl PipelineResourceLookup,
    pipeline: &PipelineDesc,
    shader: Option<ShaderModuleHandle>,
    expected_stage: ShaderStage,
    missing_reason: &str,
) -> Result<(), RhiError> {
    let shader = shader.ok_or_else(|| RhiError::InvalidPipelineDescriptor {
        label: pipeline.label.clone(),
        reason: missing_reason.to_string(),
    })?;
    let shader_desc = lookup.shader_module_desc(shader)?;
    if shader_desc.stage != expected_stage {
        return Err(RhiError::InvalidPipelineDescriptor {
            label: pipeline.label.clone(),
            reason: format!(
                "shader `{}` stage {:?} does not match required stage {:?}",
                shader.raw(),
                shader_desc.stage,
                expected_stage
            ),
        });
    }
    Ok(())
}

fn reject_shader(
    pipeline: &PipelineDesc,
    shader: Option<ShaderModuleHandle>,
    reason: &str,
) -> Result<(), RhiError> {
    if shader.is_some() {
        return Err(RhiError::InvalidPipelineDescriptor {
            label: pipeline.label.clone(),
            reason: reason.to_string(),
        });
    }
    Ok(())
}
