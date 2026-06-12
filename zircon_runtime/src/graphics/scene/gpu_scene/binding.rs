pub(crate) const GPU_SCENE_PRIMITIVE_DATA_BINDING: u32 = 0;
pub(crate) const GPU_SCENE_INSTANCE_DATA_BINDING: u32 = 1;
pub(crate) const GPU_SCENE_LIGHT_DATA_BINDING: u32 = 2;
pub(crate) const GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING: u32 = 3;
pub(crate) const GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING: u32 = 4;

const GPU_SCENE_STORAGE_VISIBILITY: wgpu::ShaderStages =
    wgpu::ShaderStages::VERTEX_FRAGMENT.union(wgpu::ShaderStages::COMPUTE);

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
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-gpu-scene-storage-bind-group"),
        layout,
        entries: &[
            storage_binding(GPU_SCENE_PRIMITIVE_DATA_BINDING, primitive_buffer),
            storage_binding(GPU_SCENE_INSTANCE_DATA_BINDING, instance_buffer),
            storage_binding(GPU_SCENE_LIGHT_DATA_BINDING, light_buffer),
            uniform_binding(
                GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
                skinned_joint_palette_buffer,
            ),
            uniform_binding(
                GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING,
                previous_skinned_joint_palette_buffer,
            ),
        ],
    })
}

pub(crate) fn gpu_scene_bind_group_layout_entries(
    skinned_joint_palette_min_binding_size: wgpu::BufferSize,
) -> [wgpu::BindGroupLayoutEntry; 5] {
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
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(min_binding_size),
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

    fn test_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(16).expect("test joint palette binding size is non-zero")
    }

    #[test]
    fn render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings() {
        let entries = gpu_scene_bind_group_layout_entries(test_joint_palette_min_binding_size());
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].binding, GPU_SCENE_PRIMITIVE_DATA_BINDING);
        assert_eq!(entries[1].binding, GPU_SCENE_INSTANCE_DATA_BINDING);
        assert_eq!(entries[2].binding, GPU_SCENE_LIGHT_DATA_BINDING);
        assert_eq!(entries[3].binding, GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING);
        assert_eq!(
            entries[4].binding,
            GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING
        );

        for entry in entries.iter().take(3) {
            assert!(entry.visibility.contains(wgpu::ShaderStages::VERTEX));
            assert!(entry.visibility.contains(wgpu::ShaderStages::FRAGMENT));
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

        for entry in entries.iter().skip(3) {
            assert_eq!(entry.visibility, wgpu::ShaderStages::VERTEX);
            match &entry.ty {
                wgpu::BindingType::Buffer {
                    ty,
                    has_dynamic_offset,
                    min_binding_size,
                } => {
                    assert!(matches!(ty, wgpu::BufferBindingType::Uniform));
                    assert!(!*has_dynamic_offset);
                    assert_eq!(
                        min_binding_size
                            .as_ref()
                            .expect("palette binding should declare a minimum size")
                            .get(),
                        test_joint_palette_min_binding_size().get()
                    );
                }
                other => panic!("expected skinned palette uniform binding, got {other:?}"),
            }
        }
    }
}
