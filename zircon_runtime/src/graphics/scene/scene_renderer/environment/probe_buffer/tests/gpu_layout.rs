use std::mem::{offset_of, size_of};

use crate::core::framework::render::{ProbeInfluenceShape, ReflectionProbeData, RenderLayerSet};
use crate::core::math::{Quat, Vec3};

use super::super::gpu_layout::{
    reflection_probe_bind_group_layout_entries, GpuPlanarReflection, GpuReflectionProbe,
    PLANAR_REFLECTION_PARAMS_BINDING, PLANAR_REFLECTION_TEXTURE_BINDING,
    REFLECTION_PROBE_CUBEMAP_BINDING, REFLECTION_PROBE_HEADER_BINDING,
    REFLECTION_PROBE_STORAGE_BINDING,
};

#[test]
fn render_probe_gpu_layout_is_96_bytes_with_documented_offsets() {
    assert_eq!(size_of::<GpuReflectionProbe>(), 96);
    assert_eq!(offset_of!(GpuReflectionProbe, position_blend), 0);
    assert_eq!(offset_of!(GpuReflectionProbe, box_min), 16);
    assert_eq!(offset_of!(GpuReflectionProbe, box_max), 32);
    assert_eq!(offset_of!(GpuReflectionProbe, proj_params), 48);
    assert_eq!(offset_of!(GpuReflectionProbe, rotation), 64);
    assert_eq!(offset_of!(GpuReflectionProbe, misc), 80);
}

#[test]
fn planar_reflection_gpu_layout_is_176_bytes_with_documented_offsets() {
    assert_eq!(size_of::<GpuPlanarReflection>(), 176);
    assert_eq!(offset_of!(GpuPlanarReflection, clip_from_world), 0);
    assert_eq!(offset_of!(GpuPlanarReflection, local_from_world), 64);
    assert_eq!(offset_of!(GpuPlanarReflection, bounds_min), 128);
    assert_eq!(offset_of!(GpuPlanarReflection, bounds_max), 144);
    assert_eq!(offset_of!(GpuPlanarReflection, sample_params), 160);
}

#[test]
fn render_probe_gpu_projection_encodes_shape_priority_slot_and_layer_mask() {
    let probe = ReflectionProbeData::try_new(
        17,
        Vec3::new(1.0, 2.0, 3.0),
        Quat::IDENTITY,
        ProbeInfluenceShape::box_shape(Vec3::new(4.0, 5.0, 6.0), 0.75).expect("box influence"),
        Vec3::new(7.0, 8.0, 9.0),
    )
    .expect("reflection probe")
    .with_box_projection(true)
    .with_priority(13)
    .with_layer_mask(RenderLayerSet::from_scene_schema_v1_mask(0x8000_0005));

    let gpu = GpuReflectionProbe::from_probe(&probe, 11, 8);

    assert_eq!(gpu.position_blend, [1.0, 2.0, 3.0, 0.75]);
    assert_eq!(gpu.box_min, [-4.0, -5.0, -6.0, 13.0]);
    assert_eq!(gpu.box_max, [4.0, 5.0, 6.0, 0.0]);
    assert_eq!(gpu.proj_params, [7.0, 8.0, 9.0, 1.0]);
    assert_eq!(gpu.rotation, Quat::IDENTITY.to_array());
    assert_eq!(gpu.misc[1], 8.0);
    assert_eq!(gpu.misc[2], 11.0);
    assert_eq!(gpu.misc[3].to_bits(), 0x8000_0005);
}

#[test]
fn render_probe_pass_layout_uses_reserved_bindings_and_cube_array_dimension() {
    let entries = reflection_probe_bind_group_layout_entries();

    assert_eq!(
        entries.map(|entry| entry.binding),
        [
            REFLECTION_PROBE_STORAGE_BINDING,
            REFLECTION_PROBE_HEADER_BINDING,
            REFLECTION_PROBE_CUBEMAP_BINDING,
            PLANAR_REFLECTION_TEXTURE_BINDING,
            PLANAR_REFLECTION_PARAMS_BINDING,
        ]
    );
    assert!(matches!(
        entries[0].ty,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            ..
        }
    ));
    assert!(matches!(
        entries[1].ty,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            ..
        }
    ));
    assert!(matches!(
        entries[2].ty,
        wgpu::BindingType::Texture {
            view_dimension: wgpu::TextureViewDimension::CubeArray,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            ..
        }
    ));
    assert!(matches!(
        entries[3].ty,
        wgpu::BindingType::Texture {
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            ..
        }
    ));
    assert!(matches!(
        entries[4].ty,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            ..
        }
    ));
}
