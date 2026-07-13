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

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub(crate) struct GpuSceneVisibleInstanceRemapParams {
    values: [u32; 4],
}

impl GpuSceneVisibleInstanceRemapParams {
    pub(crate) const fn direct() -> Self {
        Self::direct_with_light_count(0)
    }

    pub(crate) const fn remapped() -> Self {
        Self::remapped_with_light_count(0)
    }

    pub(crate) const fn direct_with_light_count(light_count: u32) -> Self {
        Self::with_values(0, light_count)
    }

    pub(crate) const fn remapped_with_light_count(light_count: u32) -> Self {
        Self::with_values(1, light_count)
    }

    const fn with_values(remap_enabled: u32, light_count: u32) -> Self {
        Self {
            values: [remap_enabled, light_count, 0, 0],
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
mod tests {
    use super::*;
    use crate::graphics::resource_limits::GPU_SCENE_COMPUTE_STORAGE_BUFFERS_PER_SHADER_STAGE;

    fn test_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(16).expect("test joint palette binding size is non-zero")
    }

    #[test]
    fn render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings() {
        let entries = gpu_scene_bind_group_layout_entries(test_joint_palette_min_binding_size());
        assert_eq!(entries.len(), 12);
        assert_eq!(entries[0].binding, GPU_SCENE_PRIMITIVE_DATA_BINDING);
        assert_eq!(entries[1].binding, GPU_SCENE_INSTANCE_DATA_BINDING);
        assert_eq!(entries[2].binding, GPU_SCENE_LIGHT_DATA_BINDING);
        assert_eq!(entries[3].binding, GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING);
        assert_eq!(
            entries[4].binding,
            GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING
        );
        assert_eq!(entries[5].binding, GPU_SCENE_VISIBLE_INSTANCE_REMAP_BINDING);
        assert_eq!(
            entries[6].binding,
            GPU_SCENE_VISIBLE_INSTANCE_REMAP_PARAMS_BINDING
        );
        assert_eq!(entries[7].binding, GPU_SCENE_MORPH_DELTAS_BINDING);
        assert_eq!(entries[8].binding, GPU_SCENE_MORPH_WEIGHTS_BINDING);
        assert_eq!(entries[9].binding, GPU_SCENE_VIRTUAL_GEOMETRY_PAGES_BINDING);
        assert_eq!(
            entries[10].binding,
            GPU_SCENE_VIRTUAL_GEOMETRY_CLUSTERS_BINDING
        );
        assert_eq!(entries[11].binding, GPU_SCENE_MORPH_PAYLOADS_BINDING);

        for entry in entries.iter().filter(|entry| {
            matches!(
                entry.binding,
                GPU_SCENE_PRIMITIVE_DATA_BINDING
                    | GPU_SCENE_INSTANCE_DATA_BINDING
                    | GPU_SCENE_LIGHT_DATA_BINDING
                    | GPU_SCENE_VISIBLE_INSTANCE_REMAP_BINDING
                    | GPU_SCENE_MORPH_DELTAS_BINDING
                    | GPU_SCENE_MORPH_WEIGHTS_BINDING
                    | GPU_SCENE_VIRTUAL_GEOMETRY_PAGES_BINDING
                    | GPU_SCENE_VIRTUAL_GEOMETRY_CLUSTERS_BINDING
                    | GPU_SCENE_MORPH_PAYLOADS_BINDING
            )
        }) {
            assert!(entry.visibility.contains(wgpu::ShaderStages::VERTEX));
            assert!(entry.visibility.contains(wgpu::ShaderStages::COMPUTE));
            match &entry.ty {
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset,
                    ..
                } => {
                    assert!(*read_only);
                    assert!(!*has_dynamic_offset);
                }
                other => panic!("expected read-only storage buffer binding, got {other:?}"),
            }
        }

        for entry in entries
            .iter()
            .filter(|entry| entry.binding != GPU_SCENE_VISIBLE_INSTANCE_REMAP_BINDING)
            .skip(3)
        {
            if entry.binding == GPU_SCENE_VISIBLE_INSTANCE_REMAP_PARAMS_BINDING {
                assert!(entry.visibility.contains(wgpu::ShaderStages::VERTEX));
                assert!(entry.visibility.contains(wgpu::ShaderStages::COMPUTE));
                continue;
            }
            if matches!(
                entry.binding,
                GPU_SCENE_MORPH_DELTAS_BINDING
                    | GPU_SCENE_MORPH_WEIGHTS_BINDING
                    | GPU_SCENE_VIRTUAL_GEOMETRY_PAGES_BINDING
                    | GPU_SCENE_VIRTUAL_GEOMETRY_CLUSTERS_BINDING
                    | GPU_SCENE_MORPH_PAYLOADS_BINDING
            ) {
                continue;
            }
            assert_eq!(entry.visibility, wgpu::ShaderStages::VERTEX);
            match &entry.ty {
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset,
                    min_binding_size,
                } => {
                    assert!(*read_only);
                    assert!(!*has_dynamic_offset);
                    assert_eq!(
                        min_binding_size
                            .as_ref()
                            .expect("palette binding should declare a minimum size")
                            .get(),
                        test_joint_palette_min_binding_size().get()
                    );
                }
                other => panic!("expected skinned palette storage binding, got {other:?}"),
            }
        }

        let compute_storage_binding_count = entries
            .iter()
            .filter(|entry| {
                entry.visibility.contains(wgpu::ShaderStages::COMPUTE)
                    && matches!(
                        entry.ty,
                        wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { .. },
                            ..
                        }
                    )
            })
            .count();
        assert_eq!(
            compute_storage_binding_count as u32,
            GPU_SCENE_COMPUTE_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
    }
}
