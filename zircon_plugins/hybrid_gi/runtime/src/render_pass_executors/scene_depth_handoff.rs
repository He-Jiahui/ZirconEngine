use zircon_runtime::graphics::{RenderPassExecutionContext, RenderPassGpuExecutionContext};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;

use crate::{
    HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS, HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL,
    HYBRID_GI_SCENE_DEPTH_HANDOFF_WORKGROUP_SIZE,
};

use super::{HYBRID_GI_SCENE_RESOURCE, SCENE_DEPTH_RESOURCE};

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
    let hybrid_gi_scene_buffer = gpu
        .require_buffer(
            HYBRID_GI_SCENE_RESOURCE,
            RenderGraphResourceAccessKind::Write,
        )?
        .clone();

    encode_scene_depth_handoff(gpu, &shader, &scene_depth_view, &hybrid_gi_scene_buffer);
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
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    shader: &SceneDepthHandoffShader,
    scene_depth_view: &wgpu::TextureView,
    hybrid_gi_scene_buffer: &wgpu::Buffer,
) {
    let bind_group_layout = create_scene_depth_handoff_bind_group_layout(gpu.device, shader);
    let pipeline = create_scene_depth_handoff_pipeline(gpu.device, shader, &bind_group_layout);
    let bind_group = create_scene_depth_handoff_bind_group(
        gpu.device,
        &bind_group_layout,
        scene_depth_view,
        hybrid_gi_scene_buffer,
    );
    let mut pass = gpu
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
    device: &wgpu::Device,
    shader: &SceneDepthHandoffShader,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn create_scene_depth_handoff_pipeline(
    device: &wgpu::Device,
    shader: &SceneDepthHandoffShader,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader_label = format!(
        "zircon-hybrid-gi-scene-depth-handoff-{}-shader",
        shader.label_suffix()
    );
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(shader_label.as_str()),
        source: wgpu::ShaderSource::Wgsl(shader.source().into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-hybrid-gi-scene-depth-handoff-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn create_scene_depth_handoff_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    scene_depth_view: &wgpu::TextureView,
    hybrid_gi_scene_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-scene-depth-handoff-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(scene_depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: hybrid_gi_scene_buffer.as_entire_binding(),
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    const HANDOFF_MAGIC: u32 = 0x48474944;
    const DEPTH_Q24_SCALE: f32 = 16777215.0;

    #[test]
    fn scene_depth_handoff_msaa_shader_resolves_depth_sample_count() {
        let Some((device, queue)) = test_device() else {
            return;
        };
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hybrid-gi-scene-depth-handoff-msaa-test-depth"),
            size: wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        let storage = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hybrid-gi-scene-depth-handoff-msaa-test-storage"),
            size: 5 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hybrid-gi-scene-depth-handoff-msaa-test-readback"),
            size: 5 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("hybrid-gi-scene-depth-handoff-msaa-test"),
        });
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hybrid-gi-scene-depth-handoff-msaa-test-clear"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.25),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }

        let shader = SceneDepthHandoffShader::for_sample_count(4);
        let bind_group_layout = create_scene_depth_handoff_bind_group_layout(&device, &shader);
        let pipeline = create_scene_depth_handoff_pipeline(&device, &shader, &bind_group_layout);
        let bind_group = create_scene_depth_handoff_bind_group(
            &device,
            &bind_group_layout,
            &depth_view,
            &storage,
        );
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("hybrid-gi-scene-depth-handoff-msaa-test-compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        encoder.copy_buffer_to_buffer(
            &storage,
            0,
            &readback,
            0,
            5 * std::mem::size_of::<u32>() as u64,
        );
        queue.submit([encoder.finish()]);

        let words = read_u32_words(&device, &readback, 5);
        assert_eq!(words[0], HANDOFF_MAGIC);
        assert_eq!(words[1], 8);
        assert_eq!(words[2], 8);
        assert_eq!(words[3], ((0.25 * DEPTH_Q24_SCALE) + 0.5) as u32);
        assert_eq!(words[4], 4);
    }

    fn test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::PRIMARY;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok()?;
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("zircon-hybrid-gi-scene-depth-handoff-msaa-test-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .ok()
    }

    fn read_u32_words(device: &wgpu::Device, buffer: &wgpu::Buffer, word_count: usize) -> Vec<u32> {
        let slice = buffer.slice(..(word_count * std::mem::size_of::<u32>()) as u64);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).ok();
        });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("device poll should complete readback mapping");
        receiver
            .recv()
            .expect("readback mapping callback should run")
            .expect("readback mapping should succeed");
        let mapped = slice.get_mapped_range();
        let words = bytemuck::cast_slice(&mapped[..]).to_vec();
        drop(mapped);
        buffer.unmap();
        words
    }
}
