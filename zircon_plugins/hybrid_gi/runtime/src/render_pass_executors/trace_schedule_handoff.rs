use zircon_runtime::graphics::{RenderPassExecutionContext, RenderPassGpuExecutionContext};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;

use crate::{
    HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS, HYBRID_GI_TRACE_SCHEDULE_PIPELINE_LABEL,
    HYBRID_GI_TRACE_SCHEDULE_WORKGROUP_SIZE,
};

use super::{HYBRID_GI_SCENE_RESOURCE, HYBRID_GI_TRACE_RESOURCE};

pub(super) fn record_trace_schedule_handoff(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    let hybrid_gi_scene_buffer = gpu
        .require_buffer(
            HYBRID_GI_SCENE_RESOURCE,
            RenderGraphResourceAccessKind::Read,
        )?
        .clone();
    let hybrid_gi_trace_buffer = gpu
        .require_buffer(
            HYBRID_GI_TRACE_RESOURCE,
            RenderGraphResourceAccessKind::Write,
        )?
        .clone();

    encode_trace_schedule_handoff(gpu, &hybrid_gi_scene_buffer, &hybrid_gi_trace_buffer);
    gpu.record_compute_dispatch(
        pass_name,
        executor_id,
        HYBRID_GI_TRACE_SCHEDULE_PIPELINE_LABEL,
        HYBRID_GI_TRACE_SCHEDULE_WORKGROUP_SIZE,
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS,
        vec![HYBRID_GI_TRACE_RESOURCE.to_string()],
    );
    Ok(())
}

fn encode_trace_schedule_handoff(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    hybrid_gi_scene_buffer: &wgpu::Buffer,
    hybrid_gi_trace_buffer: &wgpu::Buffer,
) {
    let bind_group_layout = create_trace_schedule_handoff_bind_group_layout(gpu.device);
    let pipeline = create_trace_schedule_handoff_pipeline(gpu.device, &bind_group_layout);
    let bind_group = create_trace_schedule_handoff_bind_group(
        gpu.device,
        &bind_group_layout,
        hybrid_gi_scene_buffer,
        hybrid_gi_trace_buffer,
    );
    let mut pass = gpu
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("HybridGiTraceScheduleHandoffPass"),
            timestamp_writes: None,
        });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS[0],
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS[1],
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS[2],
    );
}

fn create_trace_schedule_handoff_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
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

fn create_trace_schedule_handoff_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../hybrid_gi/renderer/shaders/trace_schedule_handoff.wgsl").into(),
        ),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(HYBRID_GI_TRACE_SCHEDULE_PIPELINE_LABEL),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn create_trace_schedule_handoff_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    hybrid_gi_scene_buffer: &wgpu::Buffer,
    hybrid_gi_trace_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: hybrid_gi_scene_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: hybrid_gi_trace_buffer.as_entire_binding(),
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use wgpu::util::DeviceExt;

    use super::*;

    const SCENE_DEPTH_HANDOFF_MAGIC: u32 = 0x4847_4944;
    const TRACE_SCHEDULE_MAGIC: u32 = 0x4847_4954;
    const DEPTH_Q24_MAX: u32 = 16_777_215;

    #[test]
    fn trace_schedule_shader_promotes_scene_depth_handoff_to_surface_cache_depth_packet() {
        let Some((device, queue)) = test_device() else {
            return;
        };
        let scene_words = [SCENE_DEPTH_HANDOFF_MAGIC, 192, 128, DEPTH_Q24_MAX / 4, 4];
        let scene = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("hybrid-gi-trace-schedule-test-scene"),
            contents: bytemuck::cast_slice(&scene_words),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let trace = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hybrid-gi-trace-schedule-test-trace"),
            size: 10 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hybrid-gi-trace-schedule-test-readback"),
            size: 10 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("hybrid-gi-trace-schedule-test"),
        });
        let bind_group_layout = create_trace_schedule_handoff_bind_group_layout(&device);
        let pipeline = create_trace_schedule_handoff_pipeline(&device, &bind_group_layout);
        let bind_group =
            create_trace_schedule_handoff_bind_group(&device, &bind_group_layout, &scene, &trace);
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("hybrid-gi-trace-schedule-test-compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        encoder.copy_buffer_to_buffer(
            &trace,
            0,
            &readback,
            0,
            10 * std::mem::size_of::<u32>() as u64,
        );
        queue.submit([encoder.finish()]);

        let words = read_u32_words(&device, &readback, 10);
        assert_eq!(words[0], TRACE_SCHEDULE_MAGIC);
        assert_eq!(words[1], 1);
        assert_eq!(words[2], 192);
        assert_eq!(words[3], 128);
        assert_eq!(words[4], DEPTH_Q24_MAX / 4);
        assert_eq!(words[5], 4);
        assert_eq!(words[6], 0xff40_4040);
        assert_eq!(words[7], 1);
        assert_eq!(words[8], 0);
        assert_eq!(words[9], SCENE_DEPTH_HANDOFF_MAGIC);
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
            label: Some("zircon-hybrid-gi-trace-schedule-test-device"),
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
