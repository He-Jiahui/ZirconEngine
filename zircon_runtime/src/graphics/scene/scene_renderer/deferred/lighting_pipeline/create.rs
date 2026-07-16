use crate::asset::ProjectAssetManager;
use crate::core::framework::render::ShadingModelDescriptor;
use crate::graphics::material::ShadingModelIncludeSourceSet;
use crate::graphics::types::GraphicsError;

use super::shader_source::{
    assemble_deferred_lighting_shader_source, DeferredLightingShaderSourceRequest,
};

pub(in crate::graphics::scene::scene_renderer::deferred) fn create_lighting_pipeline(
    device: &wgpu::Device,
    asset_manager: &ProjectAssetManager,
    scene_layout: &wgpu::BindGroupLayout,
    lighting_bind_group_layout: &wgpu::BindGroupLayout,
    gpu_scene_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    plugin_shading_models: &[ShadingModelDescriptor],
    subsurface_mrt: bool,
    volumetric_enabled: bool,
) -> Result<wgpu::RenderPipeline, GraphicsError> {
    let lighting_shader_source =
        deferred_lighting_shader_source(asset_manager, plugin_shading_models, volumetric_enabled)?;
    let lighting_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-deferred-lighting-shader"),
        source: wgpu::ShaderSource::Wgsl(lighting_shader_source.into()),
    });
    let lighting_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-deferred-lighting-layout"),
        bind_group_layouts: &[
            Some(scene_layout),
            Some(lighting_bind_group_layout),
            None,
            Some(gpu_scene_layout),
        ],
        immediate_size: 0,
    });
    let mut targets = vec![Some(wgpu::ColorTargetState {
        format: target_format,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    })];
    if subsurface_mrt {
        targets.extend([1, 2].map(|_| {
            Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba16Float,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })
        }));
    }
    Ok(
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(if subsurface_mrt {
                "zircon-deferred-lighting-subsurface-mrt-pipeline"
            } else {
                "zircon-deferred-lighting-pipeline"
            }),
            layout: Some(&lighting_layout),
            vertex: wgpu::VertexState {
                module: &lighting_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &lighting_shader,
                entry_point: Some(if subsurface_mrt {
                    "fs_main_sss"
                } else {
                    "fs_main"
                }),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &targets,
            }),
            multiview_mask: None,
            cache: None,
        }),
    )
}

fn deferred_lighting_shader_source(
    asset_manager: &ProjectAssetManager,
    plugin_shading_models: &[ShadingModelDescriptor],
    volumetric_enabled: bool,
) -> Result<String, GraphicsError> {
    let source_set = ShadingModelIncludeSourceSet::from_project_asset_manager(
        asset_manager,
        plugin_shading_models,
    )
    .map_err(|error| {
        GraphicsError::Asset(format!(
            "deferred lighting shading model include source export failed: {error}"
        ))
    })?;
    let mut request = DeferredLightingShaderSourceRequest::new()
        .with_volumetric_enabled(volumetric_enabled)
        .with_shading_model_deferred_include_sources(&source_set);
    for descriptor in plugin_shading_models.iter().cloned() {
        request = request.with_shading_model_descriptor(descriptor);
    }
    assemble_deferred_lighting_shader_source(request).map_err(|error| {
        GraphicsError::Asset(format!(
            "deferred lighting shading model shader source assembly failed: {error:?}"
        ))
    })
}
