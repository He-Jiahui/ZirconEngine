use super::*;
use std::mem::{offset_of, size_of};

#[test]
fn render_gpu_scene_layout_matches_wgsl_offsets() {
    assert_eq!(size_of::<GpuPrimitiveData>(), GPU_PRIMITIVE_DATA_STRIDE);
    assert_eq!(
        offset_of!(GpuPrimitiveData, local_bounds_center),
        GPU_PRIMITIVE_DATA_LOCAL_BOUNDS_CENTER_OFFSET
    );
    assert_eq!(
        offset_of!(GpuPrimitiveData, local_bounds_radius),
        GPU_PRIMITIVE_DATA_LOCAL_BOUNDS_RADIUS_OFFSET
    );
    assert_eq!(
        offset_of!(GpuPrimitiveData, tint),
        GPU_PRIMITIVE_DATA_TINT_OFFSET
    );
    assert_eq!(
        offset_of!(GpuPrimitiveData, shadow_params),
        GPU_PRIMITIVE_DATA_SHADOW_PARAMS_OFFSET
    );
    assert_eq!(
        offset_of!(GpuPrimitiveData, motion_params),
        GPU_PRIMITIVE_DATA_MOTION_PARAMS_OFFSET
    );
    assert_eq!(
        offset_of!(GpuPrimitiveData, flags),
        GPU_PRIMITIVE_DATA_FLAGS_OFFSET
    );
    assert_eq!(
        offset_of!(GpuPrimitiveData, first_instance_index),
        GPU_PRIMITIVE_DATA_FIRST_INSTANCE_INDEX_OFFSET
    );
    assert_eq!(
        offset_of!(GpuPrimitiveData, instance_count),
        GPU_PRIMITIVE_DATA_INSTANCE_COUNT_OFFSET
    );
    assert_eq!(
        offset_of!(GpuPrimitiveData, payload_slot),
        GPU_PRIMITIVE_DATA_PAYLOAD_SLOT_OFFSET
    );
    assert_eq!(
        offset_of!(GpuPrimitiveData, material_payload_slot),
        GPU_PRIMITIVE_DATA_MATERIAL_PAYLOAD_SLOT_OFFSET
    );
    assert_eq!(
        offset_of!(GpuPrimitiveData, hit_proxy_token),
        GPU_PRIMITIVE_DATA_HIT_PROXY_TOKEN_OFFSET
    );

    assert_eq!(size_of::<GpuInstanceData>(), GPU_INSTANCE_DATA_STRIDE);
    assert_eq!(
        offset_of!(GpuInstanceData, world_from_local),
        GPU_INSTANCE_DATA_WORLD_FROM_LOCAL_OFFSET
    );
    assert_eq!(
        offset_of!(GpuInstanceData, prev_world_from_local),
        GPU_INSTANCE_DATA_PREV_WORLD_FROM_LOCAL_OFFSET
    );
    assert_eq!(
        offset_of!(GpuInstanceData, primitive_index),
        GPU_INSTANCE_DATA_PRIMITIVE_INDEX_OFFSET
    );
    assert_eq!(
        offset_of!(GpuInstanceData, flags),
        GPU_INSTANCE_DATA_FLAGS_OFFSET
    );
    assert_eq!(
        offset_of!(GpuInstanceData, payload_slot),
        GPU_INSTANCE_DATA_PAYLOAD_SLOT_OFFSET
    );
    assert_eq!(
        offset_of!(GpuInstanceData, morph_payload_slot),
        GPU_INSTANCE_DATA_MORPH_PAYLOAD_SLOT_OFFSET
    );
    assert_eq!(
        offset_of!(GpuInstanceData, lightmap_uv_rect),
        GPU_INSTANCE_DATA_LIGHTMAP_UV_RECT_OFFSET
    );
    assert_eq!(
        offset_of!(GpuInstanceData, lightmap_params),
        GPU_INSTANCE_DATA_LIGHTMAP_PARAMS_OFFSET
    );
    assert_eq!(
        offset_of!(GpuInstanceData, skinning_palette_params),
        GPU_INSTANCE_DATA_SKINNING_PALETTE_PARAMS_OFFSET
    );

    assert_eq!(size_of::<GpuMorphPayload>(), GPU_MORPH_PAYLOAD_STRIDE);
    assert_eq!(size_of::<GpuMorphDelta>(), GPU_MORPH_DELTA_STRIDE);
    assert_eq!(size_of::<GpuMorphWeight>(), GPU_MORPH_WEIGHT_STRIDE);

    assert_eq!(
        size_of::<GpuVirtualGeometryPage>(),
        GPU_VIRTUAL_GEOMETRY_PAGE_STRIDE
    );
    assert_eq!(
        offset_of!(GpuVirtualGeometryPage, cluster_base_word),
        GPU_VIRTUAL_GEOMETRY_PAGE_CLUSTER_BASE_WORD_OFFSET
    );
    assert_eq!(
        offset_of!(GpuVirtualGeometryPage, vertex_count),
        GPU_VIRTUAL_GEOMETRY_PAGE_VERTEX_COUNT_OFFSET
    );
    assert_eq!(
        offset_of!(GpuVirtualGeometryPage, page_id),
        GPU_VIRTUAL_GEOMETRY_PAGE_PAGE_ID_OFFSET
    );
    assert_eq!(
        offset_of!(GpuVirtualGeometryPage, flags),
        GPU_VIRTUAL_GEOMETRY_PAGE_FLAGS_OFFSET
    );
    assert_eq!(
        size_of::<GpuVirtualGeometryClusterWord>(),
        GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE
    );
}

#[test]
fn render_gpu_scene_wgsl_primitive_tail_preserves_the_rust_storage_stride() {
    let source = include_str!("../../scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");

    assert!(source.contains("local_bounds_center: vec3<f32>,"));
    assert!(source.contains("local_bounds_radius: f32,"));
    assert!(!source.contains("\n    bounds_center: vec3<f32>,"));
    assert!(!source.contains("\n    bounds_radius: f32,"));
    assert!(source.contains("hit_proxy_token: u32,"));
    assert!(source.contains("material_payload_padding_1: u32,"));
    assert!(source.contains("material_payload_padding_2: u32,"));
    assert!(!source.contains("material_payload_padding_0: u32,"));
    assert!(!source.contains("material_payload_padding: vec3<u32>"));
}

#[test]
fn render_gpu_scene_wgsl_flags_match_the_rust_abi() {
    let source = include_str!("../../scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");
    let expected = [
        (
            "ZR_GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM",
            GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM,
        ),
        (
            "ZR_GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT",
            GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT,
        ),
        (
            "ZR_GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM",
            GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM,
        ),
        (
            "ZR_GPU_INSTANCE_FLAG_NON_ORTHOGONAL_TRANSFORM",
            GPU_INSTANCE_FLAG_NON_ORTHOGONAL_TRANSFORM,
        ),
        ("ZR_GPU_PRIMITIVE_FLAG_VISIBLE", GPU_PRIMITIVE_FLAG_VISIBLE),
        (
            "ZR_GPU_PRIMITIVE_FLAG_CAST_SHADOWS",
            GPU_PRIMITIVE_FLAG_CAST_SHADOWS,
        ),
        (
            "ZR_GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM",
            GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM,
        ),
        (
            "ZR_GPU_PRIMITIVE_FLAG_FORCE_HZB_VISIBLE",
            GPU_PRIMITIVE_FLAG_FORCE_HZB_VISIBLE,
        ),
    ];

    for (name, value) in expected {
        assert!(source.contains(&format!("const {name}: u32 = {value}u;")));
    }
}

#[test]
fn render_gpu_scene_lightmap_slot_preserves_uv_page_and_generation() {
    let mut instance = GpuInstanceData::default();
    let slot = crate::core::framework::render::LightmapInstanceSlot {
        atlas_page: 7,
        uv_rect: glam::Vec4::new(0.25, 0.5, 0.125, 0.25),
    };

    instance.set_lightmap(slot, 0x0123_4567_89ab_cdef);

    assert_eq!(instance.lightmap_uv_rect, [0.25, 0.5, 0.125, 0.25]);
    assert_eq!(instance.lightmap_params, [7, 1, 0x89ab_cdef, 0x0123_4567]);
}
