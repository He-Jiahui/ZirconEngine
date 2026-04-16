use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::types::{GraphicsError, VirtualGeometryPrepareFrame};

use super::gpu_readback::VirtualGeometryGpuPendingReadback;

const VG_UPLOADER_WORKGROUP_SIZE: u32 = 64;
const U32_SIZE: u64 = std::mem::size_of::<u32>() as u64;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct VirtualGeometryUploaderParams {
    resident_count: u32,
    pending_count: u32,
    page_budget: u32,
    evictable_count: u32,
}

pub(crate) struct VirtualGeometryGpuResources {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
    params_buffer: wgpu::Buffer,
}

impl VirtualGeometryGpuResources {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-vg-uploader-bind-group-layout"),
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
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-vg-uploader-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/uploader.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-vg-uploader-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("zircon-vg-uploader-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-uploader-params"),
            size: std::mem::size_of::<VirtualGeometryUploaderParams>() as u64,
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
        prepare: Option<&VirtualGeometryPrepareFrame>,
        page_budget: Option<u32>,
    ) -> Result<Option<VirtualGeometryGpuPendingReadback>, GraphicsError> {
        let Some(prepare) = prepare else {
            return Ok(None);
        };

        let page_budget = page_budget.unwrap_or_default();
        let resident_entries = prepare
            .resident_pages
            .iter()
            .map(|page| [page.page_id, page.slot])
            .collect::<Vec<_>>();
        let pending_requests = prepare
            .pending_page_requests
            .iter()
            .map(|request| request.page_id)
            .collect::<Vec<_>>();
        let page_table_word_count = resident_entries.len() * 2;
        let completed_word_count = pending_requests.len() + 1;

        let page_table_buffer = create_u32_storage_buffer(
            device,
            "zircon-vg-page-table-buffer",
            bytemuck::cast_slice(&resident_entries),
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );
        let page_table_readback = create_readback_buffer(
            device,
            "zircon-vg-page-table-readback",
            page_table_word_count,
        );
        encoder.copy_buffer_to_buffer(
            &page_table_buffer,
            0,
            &page_table_readback,
            0,
            buffer_size_for_words(page_table_word_count),
        );

        let request_buffer = create_u32_storage_buffer(
            device,
            "zircon-vg-request-buffer",
            &pending_requests,
            wgpu::BufferUsages::STORAGE,
        );
        let completed_buffer = create_u32_storage_buffer(
            device,
            "zircon-vg-completed-buffer",
            &vec![0_u32; completed_word_count.max(1)],
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );
        let completed_readback = create_readback_buffer(
            device,
            "zircon-vg-completed-readback",
            completed_word_count.max(1),
        );

        let params = VirtualGeometryUploaderParams {
            resident_count: prepare.resident_pages.len() as u32,
            pending_count: pending_requests.len() as u32,
            page_budget,
            evictable_count: prepare.evictable_pages.len() as u32,
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-vg-uploader-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: request_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: completed_buffer.as_entire_binding(),
                },
            ],
        });

        if !pending_requests.is_empty() {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("VirtualGeometryUploaderPass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                (pending_requests.len() as u32).div_ceil(VG_UPLOADER_WORKGROUP_SIZE),
                1,
                1,
            );
        }

        encoder.copy_buffer_to_buffer(
            &completed_buffer,
            0,
            &completed_readback,
            0,
            buffer_size_for_words(completed_word_count.max(1)),
        );

        Ok(Some(VirtualGeometryGpuPendingReadback::new(
            page_table_word_count,
            page_table_readback,
            completed_word_count.max(1),
            completed_readback,
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
