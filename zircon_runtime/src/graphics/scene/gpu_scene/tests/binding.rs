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

#[test]
fn render_gpu_scene_remap_params_carry_active_virtual_geometry_counts() {
    assert_eq!(
        GpuSceneVisibleInstanceRemapParams::direct_with_scene_counts(3, 5, 7).values,
        [0, 3, 5, 7]
    );
    assert_eq!(
        GpuSceneVisibleInstanceRemapParams::remapped_with_scene_counts(3, 5, 7).values,
        [1, 3, 5, 7]
    );
}
