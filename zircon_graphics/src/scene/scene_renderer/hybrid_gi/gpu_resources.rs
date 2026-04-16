use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::types::{GraphicsError, HybridGiPrepareFrame};

use super::gpu_readback::HybridGiGpuPendingReadback;

const HYBRID_GI_COMPLETION_WORKGROUP_SIZE: u32 = 64;
const U32_SIZE: u64 = std::mem::size_of::<u32>() as u64;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct HybridGiCompletionParams {
    resident_probe_count: u32,
    pending_probe_count: u32,
    probe_budget: u32,
    trace_region_count: u32,
    tracing_budget: u32,
    evictable_probe_count: u32,
    _padding: [u32; 2],
}

pub(crate) struct HybridGiGpuResources {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
    params_buffer: wgpu::Buffer,
}

impl HybridGiGpuResources {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-hybrid-gi-completion-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-hybrid-gi-completion-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/update_completion.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-hybrid-gi-completion-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("zircon-hybrid-gi-completion-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-hybrid-gi-completion-params"),
            size: std::mem::size_of::<HybridGiCompletionParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            bind_group_layout,
            pipeline,
            params_buffer,
        }
    }

    pub(crate) fn execute_prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        prepare: Option<&HybridGiPrepareFrame>,
        probe_budget: Option<u32>,
        tracing_budget: Option<u32>,
    ) -> Result<Option<HybridGiGpuPendingReadback>, GraphicsError> {
        let Some(prepare) = prepare else {
            return Ok(None);
        };

        let probe_budget = probe_budget.unwrap_or_default();
        let tracing_budget = tracing_budget.unwrap_or_default();
        let cache_entries = prepare
            .resident_probes
            .iter()
            .map(|probe| [probe.probe_id, probe.slot])
            .collect::<Vec<_>>();
        let pending_probe_ids = prepare
            .pending_updates
            .iter()
            .map(|update| update.probe_id)
            .collect::<Vec<_>>();
        let trace_region_ids = prepare.scheduled_trace_region_ids.clone();
        let cache_word_count = cache_entries.len() * 2;
        let completed_probe_word_count = pending_probe_ids.len() + 1;
        let completed_trace_word_count = trace_region_ids.len() + 1;

        let cache_buffer = create_u32_storage_buffer(
            device,
            "zircon-hybrid-gi-cache-buffer",
            bytemuck::cast_slice(&cache_entries),
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );
        let cache_readback =
            create_readback_buffer(device, "zircon-hybrid-gi-cache-readback", cache_word_count);
        encoder.copy_buffer_to_buffer(
            &cache_buffer,
            0,
            &cache_readback,
            0,
            buffer_size_for_words(cache_word_count),
        );

        let pending_probe_buffer = create_u32_storage_buffer(
            device,
            "zircon-hybrid-gi-pending-probes",
            &pending_probe_ids,
            wgpu::BufferUsages::STORAGE,
        );
        let trace_region_buffer = create_u32_storage_buffer(
            device,
            "zircon-hybrid-gi-trace-regions",
            &trace_region_ids,
            wgpu::BufferUsages::STORAGE,
        );
        let completed_probe_buffer = create_u32_storage_buffer(
            device,
            "zircon-hybrid-gi-completed-probes",
            &vec![0_u32; completed_probe_word_count.max(1)],
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );
        let completed_trace_buffer = create_u32_storage_buffer(
            device,
            "zircon-hybrid-gi-completed-traces",
            &vec![0_u32; completed_trace_word_count.max(1)],
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );
        let completed_probe_readback = create_readback_buffer(
            device,
            "zircon-hybrid-gi-completed-probe-readback",
            completed_probe_word_count.max(1),
        );
        let completed_trace_readback = create_readback_buffer(
            device,
            "zircon-hybrid-gi-completed-trace-readback",
            completed_trace_word_count.max(1),
        );

        let params = HybridGiCompletionParams {
            resident_probe_count: prepare.resident_probes.len() as u32,
            pending_probe_count: pending_probe_ids.len() as u32,
            probe_budget,
            trace_region_count: trace_region_ids.len() as u32,
            tracing_budget,
            evictable_probe_count: prepare.evictable_probe_ids.len() as u32,
            _padding: [0; 2],
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-hybrid-gi-completion-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pending_probe_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: trace_region_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: completed_probe_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: completed_trace_buffer.as_entire_binding(),
                },
            ],
        });

        let dispatch_count = pending_probe_ids.len().max(trace_region_ids.len());
        if dispatch_count > 0 {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("HybridGiCompletionPass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                (dispatch_count as u32).div_ceil(HYBRID_GI_COMPLETION_WORKGROUP_SIZE),
                1,
                1,
            );
        }

        encoder.copy_buffer_to_buffer(
            &completed_probe_buffer,
            0,
            &completed_probe_readback,
            0,
            buffer_size_for_words(completed_probe_word_count.max(1)),
        );
        encoder.copy_buffer_to_buffer(
            &completed_trace_buffer,
            0,
            &completed_trace_readback,
            0,
            buffer_size_for_words(completed_trace_word_count.max(1)),
        );

        Ok(Some(HybridGiGpuPendingReadback::new(
            cache_word_count,
            cache_readback,
            completed_probe_word_count.max(1),
            completed_probe_readback,
            completed_trace_word_count.max(1),
            completed_trace_readback,
        )))
    }
}

fn create_u32_storage_buffer(
    device: &wgpu::Device,
    label: &'static str,
    contents: &[u32],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let contents = if contents.is_empty() { &[0] } else { contents };
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(contents),
        usage: usage | wgpu::BufferUsages::COPY_DST,
    })
}

fn create_readback_buffer(
    device: &wgpu::Device,
    label: &'static str,
    word_count: usize,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: buffer_size_for_words(word_count.max(1)),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn buffer_size_for_words(word_count: usize) -> u64 {
    (word_count.max(1) as u64) * U32_SIZE
}
