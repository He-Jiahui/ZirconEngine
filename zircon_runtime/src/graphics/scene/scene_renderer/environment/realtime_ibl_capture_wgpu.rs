use wgpu::util::DeviceExt;

use crate::core::framework::render::ProceduralSkyParams;

use super::realtime_ibl_time_slice::CubeFaceRange;

const CAPTURE_WGSL: &str = include_str!("shaders/realtime_ibl_capture.wgsl");
const DOWNSAMPLE_WGSL: &str = include_str!("shaders/realtime_ibl_downsample.wgsl");
const WORKGROUP_SIZE: u32 = 8;

pub(in crate::graphics) struct RealtimeIblCaptureWgpuPipelines {
    capture_layout: wgpu::BindGroupLayout,
    capture_pipeline: wgpu::ComputePipeline,
    downsample_layout: wgpu::BindGroupLayout,
    downsample_pipeline: wgpu::ComputePipeline,
    source_sampler: wgpu::Sampler,
}

impl RealtimeIblCaptureWgpuPipelines {
    pub(in crate::graphics) fn new(device: &wgpu::Device) -> Self {
        let capture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-realtime-ibl-capture-layout"),
            entries: &[uniform_layout_entry(0), storage_texture_layout_entry(1)],
        });
        let downsample_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-realtime-ibl-downsample-layout"),
            entries: &[
                uniform_layout_entry(0),
                sampled_cube_layout_entry(1),
                sampler_layout_entry(2),
                storage_texture_layout_entry(3),
            ],
        });
        let capture_pipeline = create_pipeline(
            device,
            "zircon-realtime-ibl-capture",
            CAPTURE_WGSL,
            &capture_layout,
        );
        let downsample_pipeline = create_pipeline(
            device,
            "zircon-realtime-ibl-downsample",
            DOWNSAMPLE_WGSL,
            &downsample_layout,
        );
        let source_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-realtime-ibl-downsample-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..wgpu::SamplerDescriptor::default()
        });
        Self {
            capture_layout,
            capture_pipeline,
            downsample_layout,
            downsample_pipeline,
            source_sampler,
        }
    }

    pub(in crate::graphics) fn record_capture(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        params: &ProceduralSkyParams,
        face_size: u32,
        faces: CubeFaceRange,
        output: &wgpu::TextureView,
    ) {
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-realtime-ibl-capture-params"),
            contents: &capture_params_bytes(params, face_size, faces.first),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-realtime-ibl-capture-bind-group"),
            layout: &self.capture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(output),
                },
            ],
        });
        record_dispatch(
            encoder,
            "zircon-realtime-ibl-capture",
            &self.capture_pipeline,
            &bind_group,
            [
                div_ceil(face_size, WORKGROUP_SIZE),
                div_ceil(face_size, WORKGROUP_SIZE),
                u32::from(faces.count),
            ],
        );
    }

    pub(in crate::graphics) fn record_downsample_mip(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_face_size: u32,
        destination_face_size: u32,
        source: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        let words = [source_face_size, destination_face_size, 0, 0];
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-realtime-ibl-downsample-params"),
            contents: bytemuck::cast_slice(&words),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-realtime-ibl-downsample-bind-group"),
            layout: &self.downsample_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.source_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(output),
                },
            ],
        });
        record_dispatch(
            encoder,
            "zircon-realtime-ibl-downsample",
            &self.downsample_pipeline,
            &bind_group,
            [
                div_ceil(destination_face_size, WORKGROUP_SIZE),
                div_ceil(destination_face_size, WORKGROUP_SIZE),
                6,
            ],
        );
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    label: &str,
    source: &str,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn record_dispatch(
    encoder: &mut wgpu::CommandEncoder,
    label: &str,
    pipeline: &wgpu::ComputePipeline,
    bind_group: &wgpu::BindGroup,
    groups: [u32; 3],
) {
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some(label),
        timestamp_writes: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.dispatch_workgroups(groups[0], groups[1], groups[2]);
}

fn capture_params_bytes(params: &ProceduralSkyParams, face_size: u32, first_face: u8) -> [u8; 112] {
    let mut bytes = [0_u8; 112];
    let mut offset = 0;
    let sun = params.resolved_sun();
    for value in [
        params.horizon_color.x,
        params.horizon_color.y,
        params.horizon_color.z,
        params.horizon_color.w,
        params.zenith_color.x,
        params.zenith_color.y,
        params.zenith_color.z,
        params.zenith_color.w,
        params.ground_color.x,
        params.ground_color.y,
        params.ground_color.z,
        params.ground_color.w,
        sun.direction.x,
        sun.direction.y,
        sun.direction.z,
        sun.direction.w,
        params.sun_color.x,
        params.sun_color.y,
        params.sun_color.z,
        params.sun_color.w,
        sun.intensity_and_cosines.x,
        sun.intensity_and_cosines.y,
        sun.intensity_and_cosines.z,
        sun.intensity_and_cosines.w,
    ] {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        offset += 4;
    }
    for value in [face_size, u32::from(first_face), 0, 0] {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        offset += 4;
    }
    debug_assert_eq!(offset, bytes.len());
    bytes
}

fn uniform_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn sampled_cube_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::Cube,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn storage_texture_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::Rgba16Float,
            view_dimension: wgpu::TextureViewDimension::D2Array,
        },
        count: None,
    }
}

const fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor
}

#[cfg(test)]
mod tests;
