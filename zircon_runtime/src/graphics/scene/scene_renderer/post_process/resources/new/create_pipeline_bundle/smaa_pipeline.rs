use crate::graphics::scene::scene_renderer::anti_alias::smaa::{
    SMAA_BLEND_ENTRY_POINT, SMAA_EDGE_ENTRY_POINT, SMAA_RESOLVE_ENTRY_POINT,
};
use crate::graphics::scene::scene_renderer::post_process::SMAA_STAGE_FORMAT;

use super::super::super::shader_sources::SMAA_SHADER;

pub(super) struct SmaaPipelineBundle {
    pub(super) edge: wgpu::RenderPipeline,
    pub(super) blend: wgpu::RenderPipeline,
    pub(super) resolve: wgpu::RenderPipeline,
}

pub(super) fn smaa_pipeline_bundle(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    smaa_bind_group_layout: &wgpu::BindGroupLayout,
) -> SmaaPipelineBundle {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-smaa-shader"),
        source: wgpu::ShaderSource::Wgsl(SMAA_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-smaa-pipeline-layout"),
        bind_group_layouts: &[Some(smaa_bind_group_layout)],
        immediate_size: 0,
    });

    SmaaPipelineBundle {
        edge: smaa_pipeline(
            device,
            &shader,
            &pipeline_layout,
            "zircon-smaa-edge-pipeline",
            SMAA_EDGE_ENTRY_POINT,
            SMAA_STAGE_FORMAT,
        ),
        blend: smaa_pipeline(
            device,
            &shader,
            &pipeline_layout,
            "zircon-smaa-blend-pipeline",
            SMAA_BLEND_ENTRY_POINT,
            SMAA_STAGE_FORMAT,
        ),
        resolve: smaa_pipeline(
            device,
            &shader,
            &pipeline_layout,
            "zircon-smaa-resolve-pipeline",
            SMAA_RESOLVE_ENTRY_POINT,
            target_format,
        ),
    }
}

fn smaa_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    pipeline_layout: &wgpu::PipelineLayout,
    label: &'static str,
    fragment_entry_point: &'static str,
    target_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fragment_entry_point),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    use super::super::super::super::shader_sources::SMAA_SHADER;
    use super::SMAA_STAGE_FORMAT;

    #[test]
    fn smaa_shader_parses() {
        let module = naga::front::wgsl::parse_str(SMAA_SHADER)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(SMAA_SHADER)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator
            .validate(&module)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    #[test]
    fn smaa_internal_stage_format_is_sdr_weight_texture() {
        assert_eq!(SMAA_STAGE_FORMAT, wgpu::TextureFormat::Rgba8Unorm);
    }
}
