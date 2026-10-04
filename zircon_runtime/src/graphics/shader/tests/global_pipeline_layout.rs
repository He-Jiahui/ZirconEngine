use crate::core::framework::render::ShaderResourceKind;
use crate::graphics::shader::builtin_global_shader_contracts::{
    hzb_build_dispatch_plan, motion_vector_tile_max_pass_plan, HZB_SCENE_DEPTH_RESOURCE,
    HZB_SOURCE_RESOURCE, HZB_TARGET_RESOURCE, MOTION_VECTOR_SOURCE_RESOURCE,
};

use super::*;

#[test]
fn hzb_compute_layout_is_projected_from_named_plan_bindings() {
    let entries = compute_shader_bind_group_layout_entries(
        hzb_build_dispatch_plan(),
        &[
            ShaderWgpuResourceDescriptor::texture(
                HZB_SCENE_DEPTH_RESOURCE,
                wgpu::TextureSampleType::Depth,
                wgpu::TextureViewDimension::D2,
                false,
            ),
            ShaderWgpuResourceDescriptor::texture(
                HZB_SOURCE_RESOURCE,
                wgpu::TextureSampleType::Float { filterable: false },
                wgpu::TextureViewDimension::D2,
                false,
            ),
            ShaderWgpuResourceDescriptor::storage_texture(
                HZB_TARGET_RESOURCE,
                wgpu::TextureFormat::Rgba16Float,
                wgpu::TextureViewDimension::D2,
            ),
        ],
    )
    .expect("HZB plan should project to WGPU entries");

    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.binding)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(matches!(
        &entries[0].ty,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            ..
        }
    ));
    assert!(matches!(
        &entries[1].ty,
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Depth,
            ..
        }
    ));
    assert!(matches!(
        &entries[3].ty,
        wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::Rgba16Float,
            ..
        }
    ));
}

#[test]
fn hzb_compute_layout_reports_missing_named_resource_before_wgpu_creation() {
    let error = compute_shader_bind_group_layout_entries(
        hzb_build_dispatch_plan(),
        &[
            ShaderWgpuResourceDescriptor::texture(
                HZB_SCENE_DEPTH_RESOURCE,
                wgpu::TextureSampleType::Depth,
                wgpu::TextureViewDimension::D2,
                false,
            ),
            ShaderWgpuResourceDescriptor::texture(
                HZB_SOURCE_RESOURCE,
                wgpu::TextureSampleType::Float { filterable: false },
                wgpu::TextureViewDimension::D2,
                false,
            ),
        ],
    )
    .expect_err("missing HZB target type must fail before WGPU layout creation");

    assert_eq!(
        error,
        GlobalShaderPipelineLayoutError::MissingResourceType {
            name: HZB_TARGET_RESOURCE.to_string(),
        }
    );
}

#[test]
fn fullscreen_layout_reports_named_type_mismatch_before_wgpu_creation() {
    let error = fullscreen_pass_input_layout_entries(
        motion_vector_tile_max_pass_plan(),
        &[ShaderWgpuResourceDescriptor::storage_texture(
            MOTION_VECTOR_SOURCE_RESOURCE,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureViewDimension::D2,
        )],
    )
    .expect_err("texture contract must reject a storage-texture projection");

    assert_eq!(
        error,
        GlobalShaderPipelineLayoutError::ResourceKindMismatch {
            name: MOTION_VECTOR_SOURCE_RESOURCE.to_string(),
            expected: ShaderResourceKind::Texture,
            actual: ShaderResourceKind::StorageTexture,
        }
    );
}
