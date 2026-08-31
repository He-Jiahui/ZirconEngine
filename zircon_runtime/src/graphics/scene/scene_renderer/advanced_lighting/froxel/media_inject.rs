use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{
    FogVolumeData, FroxelGridParams, RenderLayerSet, VolumetricFogSettings,
};
use crate::core::math::Vec3;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassGpuRecordingContext, RenderPassGpuResourceFactory,
};

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
    fallback_volume_buffer: wgpu::Buffer,
    pipeline: wgpu::ComputePipeline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FroxelMediaInjectOutcome {
    pub(crate) dispatch: [u32; 3],
    pub(crate) uploaded_bytes: u64,
}

impl FroxelMediaInjectPipeline {
    pub(crate) fn uploaded_bytes(local_volume_count: usize, include_local_volumes: bool) -> u64 {
        let uploaded_volume_count = if include_local_volumes {
            local_volume_count
        } else {
            0
        };
        let bytes = std::mem::size_of::<GpuMediaInjectParams>().saturating_add(
            std::mem::size_of::<GpuFogVolume>().saturating_mul(uploaded_volume_count),
        );
        u64::try_from(bytes).unwrap_or(u64::MAX)
    }

    pub(crate) fn new<F: RenderPassGpuResourceFactory + ?Sized>(factory: &F) -> Self {
        let bind_group_layout =
            factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-volumetric-media-inject-shader"),
            source: wgpu::ShaderSource::Wgsl(MEDIA_INJECT_SHADER.into()),
        });
        let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-volumetric-media-inject-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(VOLUMETRIC_MEDIA_INJECT_PIPELINE_LABEL),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let fallback_volume_buffer =
            factory.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-volumetric-media-inject-fallback-volume"),
                contents: bytemuck::bytes_of(&GpuFogVolume::zeroed()),
                usage: wgpu::BufferUsages::STORAGE,
            });
        Self {
            bind_group_layout,
            fallback_volume_buffer,
            pipeline,
        }
    }

    pub(crate) fn encode<C: RenderPassGpuRecordingContext>(
        &self,
        context: &mut C,
        output_view: &wgpu::TextureView,
        request: FroxelMediaInjectRequest<'_>,
    ) -> Result<[u32; 3], String> {
        let request = PreparedMediaInjectRequest::new(request, None)?;
        self.encode_prepared(context, output_view, request)
            .map(|outcome| outcome.dispatch)
    }

    pub(crate) fn prepare_for_layers(
        request: FroxelMediaInjectRequest<'_>,
        render_layers: &RenderLayerSet,
    ) -> Result<PreparedMediaInjectRequest, String> {
        PreparedMediaInjectRequest::new(request, Some(render_layers))
    }

    pub(crate) fn encode_prepared<C: RenderPassGpuRecordingContext>(
        &self,
        context: &mut C,
        output_view: &wgpu::TextureView,
        request: PreparedMediaInjectRequest,
    ) -> Result<FroxelMediaInjectOutcome, String> {
        let uploaded_bytes = Self::uploaded_bytes(request.volumes.len(), true);
        let params_buffer =
            context
                .resource_factory()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("zircon-volumetric-media-inject-params"),
                    contents: bytemuck::bytes_of(&request.params),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
        let uploaded_volume_buffer = (!request.volumes.is_empty()).then(|| {
            context
                .resource_factory()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("zircon-volumetric-media-inject-volumes"),
                    contents: bytemuck::cast_slice(&request.volumes),
                    usage: wgpu::BufferUsages::STORAGE,
                })
        });
        let volume_buffer = uploaded_volume_buffer
            .as_ref()
            .unwrap_or(&self.fallback_volume_buffer);
        let bind_group = context
            .resource_factory()
            .create_bind_group(&wgpu::BindGroupDescriptor {
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
        let mut pass = context
            .command_encoder()
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("VolumetricMediaInjectPass"),
                timestamp_writes: None,
            });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(dispatch[0], dispatch[1], dispatch[2]);
        Ok(FroxelMediaInjectOutcome {
            dispatch,
            uploaded_bytes,
        })
    }
}

pub(crate) struct PreparedMediaInjectRequest {
    params: GpuMediaInjectParams,
    volumes: Vec<GpuFogVolume>,
}

impl PreparedMediaInjectRequest {
    fn new(
        request: FroxelMediaInjectRequest<'_>,
        render_layers: Option<&RenderLayerSet>,
    ) -> Result<Self, String> {
        let grid = request.grid.sanitized();
        let settings = request.settings.sanitized();
        let volumes = collect_gpu_volumes(
            request.local_volumes,
            request.include_local_volumes,
            render_layers,
        );
        let volume_count = u32::try_from(volumes.len())
            .map_err(|_| "volumetric media inject local volume count exceeds u32".to_string())?;
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

fn collect_gpu_volumes(
    local_volumes: &[FogVolumeData],
    include_local_volumes: bool,
    render_layers: Option<&RenderLayerSet>,
) -> Vec<GpuFogVolume> {
    if !include_local_volumes {
        return Vec::new();
    }
    let mut volumes = Vec::with_capacity(local_volumes.len());
    volumes.extend(
        local_volumes
            .iter()
            .filter(|volume| {
                render_layers
                    .map(|render_layers| volume.layer_mask.intersects(render_layers))
                    .unwrap_or(true)
            })
            .map(GpuFogVolume::from),
    );
    volumes
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct GpuMediaInjectParams {
    grid_and_volume_count: [u32; 4],
    density_height_scattering: [f32; 4],
    albedo_phase: [f32; 4],
    view: GpuFroxelViewParams,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
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
mod performance_tests;

#[cfg(test)]
mod tests;
