use zircon_runtime::graphics::{
    RenderPassBufferUploadSink, RenderPassExecutionContext, RenderPassGpuNativeContext,
    RenderPassGpuResourceFactory,
};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;

use crate::{
    HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS, HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL,
    HYBRID_GI_SCENE_DEPTH_HANDOFF_WORKGROUP_SIZE,
};

use super::scene_hzb_camera_packet::{scene_hzb_camera_packet, SCENE_HZB_CAMERA_WORD_OFFSET};
use super::scene_trace_input_packet::{scene_trace_input_packet, SCENE_TRACE_INPUT_WORD_OFFSET};
use super::{
    HYBRID_GI_SCENE_RESOURCE, SCENE_DEPTH_RESOURCE, SCENE_HZB_RESOURCE, SCENE_NORMAL_RESOURCE,
};

enum SceneDepthHandoffShader {
    SingleSample,
    Multisampled,
}

impl SceneDepthHandoffShader {
    fn for_sample_count(sample_count: u32) -> Self {
        if sample_count > 1 {
            Self::Multisampled
        } else {
            Self::SingleSample
        }
    }

    fn label_suffix(&self) -> &'static str {
        match self {
            Self::SingleSample => "single-sample",
            Self::Multisampled => "msaa",
        }
    }

    fn source(&self) -> &'static str {
        match self {
            Self::SingleSample => {
                include_str!("../hybrid_gi/renderer/shaders/scene_depth_handoff.wgsl")
            }
            Self::Multisampled => {
                include_str!("../hybrid_gi/renderer/shaders/scene_depth_handoff_msaa.wgsl")
            }
        }
    }

    fn texture_multisampled(&self) -> bool {
        matches!(self, Self::Multisampled)
    }
}

pub(super) fn record_scene_depth_handoff(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    let scene_depth_desc =
        gpu.require_texture_desc(SCENE_DEPTH_RESOURCE, RenderGraphResourceAccessKind::Read)?;
    let shader = SceneDepthHandoffShader::for_sample_count(scene_depth_desc.sample_count);
    let scene_depth_view = gpu
        .require_texture_view(SCENE_DEPTH_RESOURCE, RenderGraphResourceAccessKind::Read)?
        .clone();
    let scene_normal_view = gpu
        .require_texture_view(SCENE_NORMAL_RESOURCE, RenderGraphResourceAccessKind::Read)?
        .clone();
    let scene_hzb_view = gpu.require_owned_texture_full_mip_view(
        SCENE_HZB_RESOURCE,
        RenderGraphResourceAccessKind::Read,
    )?;
    let hybrid_gi_scene_buffer = gpu.require_buffer_binding(
        HYBRID_GI_SCENE_RESOURCE,
        RenderGraphResourceAccessKind::Write,
    )?;
    let camera_packet = scene_hzb_camera_packet(gpu.frame_extract(), gpu.viewport_size());
    let scene_trace_packet =
        scene_trace_input_packet(&gpu.plugin_outputs().hybrid_gi.scene_prepare);
    let mut buffer_uploads = gpu.buffer_upload_recorder();
    write_buffer_binding(
        &mut buffer_uploads,
        hybrid_gi_scene_buffer.clone(),
        SCENE_HZB_CAMERA_WORD_OFFSET * std::mem::size_of::<u32>() as u64,
        bytemuck::cast_slice(&camera_packet),
        "scene HZB camera packet",
    )?;
    write_buffer_binding(
        &mut buffer_uploads,
        hybrid_gi_scene_buffer.clone(),
        (SCENE_TRACE_INPUT_WORD_OFFSET * std::mem::size_of::<u32>()) as u64,
        bytemuck::cast_slice(&scene_trace_packet),
        "scene trace input packet",
    )?;
    drop(buffer_uploads);

    let mut native = gpu.native_context();
    encode_scene_depth_handoff(
        &mut native,
        &shader,
        &scene_depth_view,
        &scene_normal_view,
        &scene_hzb_view,
        hybrid_gi_scene_buffer,
    );
    drop(native);
    gpu.record_compute_dispatch(
        pass_name,
        executor_id,
        HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL,
        HYBRID_GI_SCENE_DEPTH_HANDOFF_WORKGROUP_SIZE,
        HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS,
        vec![HYBRID_GI_SCENE_RESOURCE.to_string()],
    );
    Ok(())
}

fn encode_scene_depth_handoff(
    native: &mut RenderPassGpuNativeContext<'_, '_>,
    shader: &SceneDepthHandoffShader,
    scene_depth_view: &wgpu::TextureView,
    scene_normal_view: &wgpu::TextureView,
    scene_hzb_view: &wgpu::TextureView,
    hybrid_gi_scene_buffer: wgpu::BufferBinding<'_>,
) {
    let bind_group_layout = create_scene_depth_handoff_bind_group_layout(native, shader);
    let pipeline = create_scene_depth_handoff_pipeline(native, shader, &bind_group_layout);
    let bind_group = create_scene_depth_handoff_bind_group(
        native,
        &bind_group_layout,
        scene_depth_view,
        scene_hzb_view,
        hybrid_gi_scene_buffer,
        scene_normal_view,
    );
    let mut pass = native
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("HybridGiSceneDepthHandoffPass"),
            timestamp_writes: None,
        });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(
        HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS[0],
        HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS[1],
        HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS[2],
    );
}

fn create_scene_depth_handoff_bind_group_layout(
    factory: &impl RenderPassGpuResourceFactory,
    shader: &SceneDepthHandoffShader,
) -> wgpu::BindGroupLayout {
    factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-hybrid-gi-scene-depth-handoff-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: shader.texture_multisampled(),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: shader.texture_multisampled(),
                },
                count: None,
            },
        ],
    })
}

fn create_scene_depth_handoff_pipeline(
    factory: &impl RenderPassGpuResourceFactory,
    shader: &SceneDepthHandoffShader,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader_label = format!(
        "zircon-hybrid-gi-scene-depth-handoff-{}-shader",
        shader.label_suffix()
    );
    let shader_module = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(shader_label.as_str()),
        source: wgpu::ShaderSource::Wgsl(shader.source().into()),
    });
    let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-hybrid-gi-scene-depth-handoff-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn create_scene_depth_handoff_bind_group(
    factory: &impl RenderPassGpuResourceFactory,
    bind_group_layout: &wgpu::BindGroupLayout,
    scene_depth_view: &wgpu::TextureView,
    scene_hzb_view: &wgpu::TextureView,
    hybrid_gi_scene_buffer: wgpu::BufferBinding<'_>,
    scene_normal_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    factory.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-scene-depth-handoff-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(scene_depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(scene_hzb_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(hybrid_gi_scene_buffer),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(scene_normal_view),
            },
        ],
    })
}

fn write_buffer_binding(
    buffer_uploads: &mut dyn RenderPassBufferUploadSink,
    binding: wgpu::BufferBinding<'_>,
    relative_offset: u64,
    bytes: &[u8],
    label: &str,
) -> Result<(), String> {
    let byte_count = u64::try_from(bytes.len())
        .map_err(|_| format!("hybrid GI {label} payload length does not fit u64"))?;
    let relative_end = relative_offset
        .checked_add(byte_count)
        .ok_or_else(|| format!("hybrid GI {label} offset overflows its compiler buffer window"))?;
    let backing_remaining = binding
        .buffer
        .size()
        .checked_sub(binding.offset)
        .ok_or_else(|| {
            format!("hybrid GI {label} binding offset exceeds its backing buffer size")
        })?;
    let window_size = binding.size.map_or(backing_remaining, |size| size.get());
    if relative_end > window_size {
        return Err(format!(
            "hybrid GI {label} range [{relative_offset}..{relative_end}) exceeds compiler buffer window size {window_size}"
        ));
    }
    let absolute_offset = binding.offset.checked_add(relative_offset).ok_or_else(|| {
        format!("hybrid GI {label} absolute buffer offset overflows its backing buffer")
    })?;
    buffer_uploads.write_buffer(binding.buffer, absolute_offset, bytes);
    Ok(())
}

#[cfg(test)]
#[path = "tests/scene_depth_handoff.rs"]
mod tests;
