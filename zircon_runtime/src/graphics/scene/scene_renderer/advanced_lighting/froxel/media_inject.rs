use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::core::framework::render::{FogVolumeData, FroxelGridParams, VolumetricFogSettings};
use crate::core::math::Vec3;

use super::{FroxelViewReconstruction, GpuFroxelViewParams};

pub(crate) const VOLUMETRIC_MEDIA_INJECT_PIPELINE_LABEL: &str = "zircon-volumetric-media-inject";
pub(crate) const VOLUMETRIC_MEDIA_INJECT_WORKGROUP_SIZE: [u32; 3] = [4, 4, 4];

const MEDIA_INJECT_SHADER: &str = concat!(
    include_str!("shaders/zr_froxel_reconstruct.wgsl"),
    "\n",
    include_str!("media_inject/shaders/media_inject.wgsl"),
);

pub(crate) struct FroxelMediaInjectRequest<'a> {
    pub settings: VolumetricFogSettings,
    pub grid: FroxelGridParams,
    pub view: FroxelViewReconstruction,
    pub local_volumes: &'a [FogVolumeData],
    pub include_local_volumes: bool,
}

pub(crate) struct FroxelMediaInjectPipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl FroxelMediaInjectPipeline {
    pub(crate) fn uploaded_bytes(local_volume_count: usize, include_local_volumes: bool) -> u64 {
        let uploaded_volume_count = if include_local_volumes {
            local_volume_count
        } else {
            0
        }
        .max(1);
        let bytes = std::mem::size_of::<GpuMediaInjectParams>().saturating_add(
            std::mem::size_of::<GpuFogVolume>().saturating_mul(uploaded_volume_count),
        );
        u64::try_from(bytes).unwrap_or(u64::MAX)
    }

    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-volumetric-media-inject-bind-group-layout"),
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
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-volumetric-media-inject-shader"),
            source: wgpu::ShaderSource::Wgsl(MEDIA_INJECT_SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-volumetric-media-inject-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(VOLUMETRIC_MEDIA_INJECT_PIPELINE_LABEL),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        Self {
            bind_group_layout,
            pipeline,
        }
    }

    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        output_view: &wgpu::TextureView,
        request: FroxelMediaInjectRequest<'_>,
    ) -> Result<[u32; 3], String> {
        let request = ValidatedMediaInjectRequest::new(request)?;
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-volumetric-media-inject-params"),
            contents: bytemuck::bytes_of(&request.params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let volume_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-volumetric-media-inject-volumes"),
            contents: bytemuck::cast_slice(&request.volumes),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-volumetric-media-inject-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: volume_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(output_view),
                },
            ],
        });
        let dispatch = dispatch_size([
            request.params.grid_and_volume_count[0],
            request.params.grid_and_volume_count[1],
            request.params.grid_and_volume_count[2],
        ]);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("VolumetricMediaInjectPass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(dispatch[0], dispatch[1], dispatch[2]);
        Ok(dispatch)
    }
}

struct ValidatedMediaInjectRequest {
    params: GpuMediaInjectParams,
    volumes: Vec<GpuFogVolume>,
}

impl ValidatedMediaInjectRequest {
    fn new(request: FroxelMediaInjectRequest<'_>) -> Result<Self, String> {
        let grid = request.grid.sanitized();
        let settings = request.settings.sanitized();
        let mut volumes = if request.include_local_volumes {
            request
                .local_volumes
                .iter()
                .map(GpuFogVolume::from)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let volume_count = u32::try_from(volumes.len())
            .map_err(|_| "volumetric media inject local volume count exceeds u32".to_string())?;
        if volumes.is_empty() {
            volumes.push(GpuFogVolume::zeroed());
        }
        Ok(Self {
            params: GpuMediaInjectParams {
                grid_and_volume_count: [
                    grid.dimensions[0],
                    grid.dimensions[1],
                    grid.dimensions[2],
                    volume_count,
                ],
                density_height_scattering: [
                    settings.density,
                    settings.height_falloff,
                    settings.scattering_intensity,
                    0.0,
                ],
                albedo_phase: [
                    settings.albedo.x,
                    settings.albedo.y,
                    settings.albedo.z,
                    settings.phase_g,
                ],
                view: GpuFroxelViewParams::new(request.view, grid)?,
            },
            volumes,
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuMediaInjectParams {
    grid_and_volume_count: [u32; 4],
    density_height_scattering: [f32; 4],
    albedo_phase: [f32; 4],
    view: GpuFroxelViewParams,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuFogVolume {
    bounds_min_density: [f32; 4],
    bounds_max: [f32; 4],
    albedo: [f32; 4],
}

impl From<&FogVolumeData> for GpuFogVolume {
    fn from(volume: &FogVolumeData) -> Self {
        let bounds_min = volume.bounds_min.min(volume.bounds_max);
        let bounds_max = volume.bounds_min.max(volume.bounds_max);
        let albedo = volume.sanitized_albedo();
        Self {
            bounds_min_density: [
                bounds_min.x,
                bounds_min.y,
                bounds_min.z,
                volume.sanitized_density(),
            ],
            bounds_max: [bounds_max.x, bounds_max.y, bounds_max.z, 0.0],
            albedo: [albedo.x, albedo.y, albedo.z, 0.0],
        }
    }
}

fn dispatch_size(dimensions: [u32; 3]) -> [u32; 3] {
    [
        dimensions[0].div_ceil(VOLUMETRIC_MEDIA_INJECT_WORKGROUP_SIZE[0]),
        dimensions[1].div_ceil(VOLUMETRIC_MEDIA_INJECT_WORKGROUP_SIZE[1]),
        dimensions[2].div_ceil(VOLUMETRIC_MEDIA_INJECT_WORKGROUP_SIZE[2]),
    ]
}

#[cfg(test)]
mod tests;
