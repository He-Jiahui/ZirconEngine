use std::sync::Arc;

use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{ProbeInfluenceShape, ReflectionProbeData};

pub(super) const REFLECTION_PROBE_STORAGE_BINDING: u32 = 16;
pub(super) const REFLECTION_PROBE_HEADER_BINDING: u32 = 17;
pub(super) const REFLECTION_PROBE_CUBEMAP_BINDING: u32 = 18;
pub(super) const PLANAR_REFLECTION_TEXTURE_BINDING: u32 = 29;
pub(super) const PLANAR_REFLECTION_PARAMS_BINDING: u32 = 30;

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(super) struct GpuReflectionProbe {
    pub(super) position_blend: [f32; 4],
    pub(super) box_min: [f32; 4],
    pub(super) box_max: [f32; 4],
    pub(super) proj_params: [f32; 4],
    pub(super) rotation: [f32; 4],
    pub(super) misc: [f32; 4],
}

impl GpuReflectionProbe {
    pub(super) fn from_probe(
        probe: &ReflectionProbeData,
        array_slice: u32,
        mip_count: u32,
    ) -> Self {
        let (half_extents, shape) = match probe.shape() {
            ProbeInfluenceShape::Box { half_extents, .. } => (half_extents, 0.0),
            ProbeInfluenceShape::Sphere { radius, .. } => {
                (crate::core::math::Vec3::splat(radius), 1.0)
            }
        };
        let position = probe.position();
        let projection = probe.projection_half_extents();
        Self {
            position_blend: [
                position.x,
                position.y,
                position.z,
                probe.shape().blend_distance(),
            ],
            box_min: [
                -half_extents.x,
                -half_extents.y,
                -half_extents.z,
                probe.priority() as f32,
            ],
            box_max: [half_extents.x, half_extents.y, half_extents.z, shape],
            proj_params: [
                projection.x,
                projection.y,
                projection.z,
                u32::from(probe.box_projection()) as f32,
            ],
            rotation: probe.rotation().to_array(),
            misc: [
                probe.intensity(),
                mip_count.max(1) as f32,
                array_slice as f32,
                f32::from_bits(probe.layer_mask().to_scene_schema_v1_mask_lossy()),
            ],
        }
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub(super) struct GpuReflectionProbeHeader {
    pub(super) probe_count: u32,
    _padding: [u32; 3],
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct GpuPlanarReflection {
    pub(super) clip_from_world: [[f32; 4]; 4],
    pub(super) local_from_world: [[f32; 4]; 4],
    pub(super) bounds_min: [f32; 4],
    pub(super) bounds_max: [f32; 4],
    /// xy = populated fraction of the fixed texture, z = mip count, w = enabled.
    pub(super) sample_params: [f32; 4],
}

impl Default for GpuPlanarReflection {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl GpuReflectionProbeHeader {
    pub(super) const fn with_probe_count(probe_count: u32) -> Self {
        Self {
            probe_count,
            _padding: [0; 3],
        }
    }
}

#[derive(Clone)]
pub(in crate::graphics::scene::scene_renderer) struct ReflectionProbeGpuBindings {
    probe_buffer: Arc<wgpu::Buffer>,
    header_buffer: Arc<wgpu::Buffer>,
    cubemap_array_view: Arc<wgpu::TextureView>,
    planar_params_buffer: Arc<wgpu::Buffer>,
    planar_texture_view: Arc<wgpu::TextureView>,
}

impl ReflectionProbeGpuBindings {
    pub(super) fn new(
        probe_buffer: Arc<wgpu::Buffer>,
        header_buffer: Arc<wgpu::Buffer>,
        cubemap_array_view: Arc<wgpu::TextureView>,
        planar_params_buffer: Arc<wgpu::Buffer>,
        planar_texture_view: Arc<wgpu::TextureView>,
    ) -> Self {
        Self {
            probe_buffer,
            header_buffer,
            cubemap_array_view,
            planar_params_buffer,
            planar_texture_view,
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn bind_group_entries(
        &self,
    ) -> [wgpu::BindGroupEntry<'_>; 5] {
        [
            wgpu::BindGroupEntry {
                binding: REFLECTION_PROBE_STORAGE_BINDING,
                resource: self.probe_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: REFLECTION_PROBE_HEADER_BINDING,
                resource: self.header_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: REFLECTION_PROBE_CUBEMAP_BINDING,
                resource: wgpu::BindingResource::TextureView(&self.cubemap_array_view),
            },
            wgpu::BindGroupEntry {
                binding: PLANAR_REFLECTION_TEXTURE_BINDING,
                resource: wgpu::BindingResource::TextureView(&self.planar_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: PLANAR_REFLECTION_PARAMS_BINDING,
                resource: self.planar_params_buffer.as_entire_binding(),
            },
        ]
    }
}

pub(in crate::graphics::scene::scene_renderer) fn reflection_probe_bind_group_layout_entries()
-> [wgpu::BindGroupLayoutEntry; 5] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: REFLECTION_PROBE_STORAGE_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: REFLECTION_PROBE_HEADER_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    std::mem::size_of::<GpuReflectionProbeHeader>() as u64,
                ),
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: REFLECTION_PROBE_CUBEMAP_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::CubeArray,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: PLANAR_REFLECTION_TEXTURE_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: PLANAR_REFLECTION_PARAMS_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    std::mem::size_of::<GpuPlanarReflection>() as u64
                ),
            },
            count: None,
        },
    ]
}
