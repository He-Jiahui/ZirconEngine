//! 此处固定 GPU Scene 的 Rust/WGSL 绑定槽位；改动需同时检查场景 bind group、各绘制管线和着色器读取。

use bytemuck::{Pod, Zeroable};

pub(crate) const GPU_SCENE_PRIMITIVE_DATA_BINDING: u32 = 0;
pub(crate) const GPU_SCENE_INSTANCE_DATA_BINDING: u32 = 1;
pub(crate) const GPU_SCENE_LIGHT_DATA_BINDING: u32 = 2;
pub(crate) const GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING: u32 = 3;
pub(crate) const GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING: u32 = 4;
pub(crate) const GPU_SCENE_VISIBLE_INSTANCE_REMAP_BINDING: u32 = 5;
pub(crate) const GPU_SCENE_VISIBLE_INSTANCE_REMAP_PARAMS_BINDING: u32 = 6;
pub(crate) const GPU_SCENE_MORPH_DELTAS_BINDING: u32 = 7;
pub(crate) const GPU_SCENE_MORPH_WEIGHTS_BINDING: u32 = 8;
pub(crate) const GPU_SCENE_VIRTUAL_GEOMETRY_PAGES_BINDING: u32 = 9;
pub(crate) const GPU_SCENE_VIRTUAL_GEOMETRY_CLUSTERS_BINDING: u32 = 10;
pub(crate) const GPU_SCENE_MORPH_PAYLOADS_BINDING: u32 = 11;

const GPU_SCENE_STORAGE_VISIBILITY: wgpu::ShaderStages =
    wgpu::ShaderStages::VERTEX_FRAGMENT.union(wgpu::ShaderStages::COMPUTE);
const GPU_SCENE_REMAP_PARAMS_VISIBILITY: wgpu::ShaderStages =
    wgpu::ShaderStages::VERTEX_FRAGMENT.union(wgpu::ShaderStages::COMPUTE);

/// 绘制与计算着色器共用的场景计数和可见实例重映射开关。
/// 资源上传准备阶段必须把本帧灯光和虚拟几何数量写入两个模式的参数缓冲，再让绘制包读取。
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub(crate) struct GpuSceneVisibleInstanceRemapParams {
    // [remap_enabled, light_count, active_vg_page_count, active_vg_cluster_word_count]
    values: [u32; 4],
}

impl GpuSceneVisibleInstanceRemapParams {
    pub(crate) const fn direct() -> Self {
        Self::direct_with_scene_counts(0, 0, 0)
    }

    pub(crate) const fn remapped() -> Self {
        Self::remapped_with_scene_counts(0, 0, 0)
    }

    pub(crate) const fn direct_with_scene_counts(
        light_count: u32,
        virtual_geometry_page_count: u32,
        virtual_geometry_cluster_word_count: u32,
    ) -> Self {
        Self::with_values(
            0,
            light_count,
            virtual_geometry_page_count,
            virtual_geometry_cluster_word_count,
        )
    }

    pub(crate) const fn remapped_with_scene_counts(
        light_count: u32,
        virtual_geometry_page_count: u32,
        virtual_geometry_cluster_word_count: u32,
    ) -> Self {
        Self::with_values(
            1,
            light_count,
            virtual_geometry_page_count,
            virtual_geometry_cluster_word_count,
        )
    }

    const fn with_values(
        remap_enabled: u32,
        light_count: u32,
        virtual_geometry_page_count: u32,
        virtual_geometry_cluster_word_count: u32,
    ) -> Self {
        Self {
            values: [
                remap_enabled,
                light_count,
                virtual_geometry_page_count,
                virtual_geometry_cluster_word_count,
            ],
        }
    }
}

pub(crate) fn create_gpu_scene_bind_group_layout(
    device: &wgpu::Device,
    skinned_joint_palette_min_binding_size: wgpu::BufferSize,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-gpu-scene-storage-layout"),
        entries: &gpu_scene_bind_group_layout_entries(skinned_joint_palette_min_binding_size),
    })
}

pub(crate) fn create_gpu_scene_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    primitive_buffer: &wgpu::Buffer,
    instance_buffer: &wgpu::Buffer,
    light_buffer: &wgpu::Buffer,
    skinned_joint_palette_buffer: &wgpu::Buffer,
    previous_skinned_joint_palette_buffer: &wgpu::Buffer,
    visible_instance_remap_buffer: &wgpu::Buffer,
    visible_instance_remap_params_buffer: &wgpu::Buffer,
    morph_deltas_buffer: &wgpu::Buffer,
    morph_weights_buffer: &wgpu::Buffer,
    virtual_geometry_pages_buffer: &wgpu::Buffer,
    virtual_geometry_clusters_buffer: &wgpu::Buffer,
    morph_payloads_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-gpu-scene-storage-bind-group"),
        layout,
        entries: &[
            storage_binding(GPU_SCENE_PRIMITIVE_DATA_BINDING, primitive_buffer),
            storage_binding(GPU_SCENE_INSTANCE_DATA_BINDING, instance_buffer),
            storage_binding(GPU_SCENE_LIGHT_DATA_BINDING, light_buffer),
            storage_binding(
                GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
                skinned_joint_palette_buffer,
            ),
            storage_binding(
                GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING,
                previous_skinned_joint_palette_buffer,
            ),
            storage_binding(
                GPU_SCENE_VISIBLE_INSTANCE_REMAP_BINDING,
                visible_instance_remap_buffer,
            ),
            uniform_binding(
                GPU_SCENE_VISIBLE_INSTANCE_REMAP_PARAMS_BINDING,
                visible_instance_remap_params_buffer,
            ),
            storage_binding(GPU_SCENE_MORPH_DELTAS_BINDING, morph_deltas_buffer),
            storage_binding(GPU_SCENE_MORPH_WEIGHTS_BINDING, morph_weights_buffer),
            storage_binding(
                GPU_SCENE_VIRTUAL_GEOMETRY_PAGES_BINDING,
                virtual_geometry_pages_buffer,
            ),
            storage_binding(
                GPU_SCENE_VIRTUAL_GEOMETRY_CLUSTERS_BINDING,
                virtual_geometry_clusters_buffer,
            ),
            storage_binding(GPU_SCENE_MORPH_PAYLOADS_BINDING, morph_payloads_buffer),
        ],
    })
}

pub(crate) fn gpu_scene_bind_group_layout_entries(
    skinned_joint_palette_min_binding_size: wgpu::BufferSize,
) -> [wgpu::BindGroupLayoutEntry; 12] {
    [
        storage_layout_entry(GPU_SCENE_PRIMITIVE_DATA_BINDING),
        storage_layout_entry(GPU_SCENE_INSTANCE_DATA_BINDING),
        storage_layout_entry(GPU_SCENE_LIGHT_DATA_BINDING),
        skinned_joint_palette_layout_entry(
            GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
            skinned_joint_palette_min_binding_size,
        ),
        skinned_joint_palette_layout_entry(
            GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING,
            skinned_joint_palette_min_binding_size,
        ),
        storage_layout_entry(GPU_SCENE_VISIBLE_INSTANCE_REMAP_BINDING),
        remap_params_layout_entry(GPU_SCENE_VISIBLE_INSTANCE_REMAP_PARAMS_BINDING),
        storage_layout_entry(GPU_SCENE_MORPH_DELTAS_BINDING),
        storage_layout_entry(GPU_SCENE_MORPH_WEIGHTS_BINDING),
        storage_layout_entry(GPU_SCENE_VIRTUAL_GEOMETRY_PAGES_BINDING),
        storage_layout_entry(GPU_SCENE_VIRTUAL_GEOMETRY_CLUSTERS_BINDING),
        storage_layout_entry(GPU_SCENE_MORPH_PAYLOADS_BINDING),
    ]
}

fn storage_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: GPU_SCENE_STORAGE_VISIBILITY,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn skinned_joint_palette_layout_entry(
    binding: u32,
    min_binding_size: wgpu::BufferSize,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: Some(min_binding_size),
        },
        count: None,
    }
}

fn remap_params_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: GPU_SCENE_REMAP_PARAMS_VISIBILITY,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                GpuSceneVisibleInstanceRemapParams,
            >() as u64),
        },
        count: None,
    }
}

fn storage_binding(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset: 0,
            size: None,
        }),
    }
}

fn uniform_binding(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset: 0,
            size: None,
        }),
    }
}

#[cfg(test)]
#[path = "tests/binding.rs"]
mod tests;
