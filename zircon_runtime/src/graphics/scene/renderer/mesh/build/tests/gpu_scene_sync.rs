use crate::core::framework::render::{
    CastShadowsMode, LightmapAtlasDescriptor, LightmapAtlasFormat, LightmapConsumeContract,
    LightmapInstanceSlot, RendererCommon,
};
use crate::core::math::Vec4;
use crate::core::resource::ResourceId;
use crate::graphics::scene::gpu_scene::{
    GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM, GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM,
    GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT, GPU_INSTANCE_FLAG_NON_ORTHOGONAL_TRANSFORM,
    GPU_PRIMITIVE_FLAG_CAST_SHADOWS, GPU_PRIMITIVE_FLAG_FORCE_HZB_VISIBLE,
    GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM, GPU_PRIMITIVE_FLAG_VISIBLE,
};

use super::{
    cross3, dot3, normal_transform_flags_for_model_matrix,
    normal_transform_flags_reverse_raster_winding, primitive_flags_for_renderer,
    resolve_hit_proxy_token, velocity_history_is_available, MeshHitProxyTokenSource,
};

struct FixedHitProxyToken(Option<u32>);

impl MeshHitProxyTokenSource for FixedHitProxyToken {
    fn token_for_instance(&self, _stable_instance_key: u64) -> Option<u32> {
        self.0
    }
}

#[test]
fn render_scene_journal_owns_product_gpu_scene_membership_when_available() {
    let source = include_str!("../gpu_scene_sync.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("GPUScene sync must retain a test boundary");

    assert!(production.contains("if !gpu_scene.journal_membership_authoritative()"));
    assert!(production.contains("gpu_scene.retain_registered_keys(&live_keys)"));
    assert!(!production.contains("let mut live_keys = HashSet::with_capacity"));
}

#[test]
fn render_hit_proxy_tokens_are_opt_in_and_reserve_zero_for_no_hit() {
    assert_eq!(resolve_hit_proxy_token(17, None), 0);
    assert_eq!(
        resolve_hit_proxy_token(17, Some(&FixedHitProxyToken(None))),
        0
    );
    assert_eq!(
        resolve_hit_proxy_token(17, Some(&FixedHitProxyToken(Some(0)))),
        0
    );
    assert_eq!(
        resolve_hit_proxy_token(17, Some(&FixedHitProxyToken(Some(9)))),
        9
    );
}

#[test]
fn render_renderer_common_shadow_modes_project_to_gpu_primitive_flags() {
    let off = renderer_common(CastShadowsMode::Off);
    let two_sided = renderer_common(CastShadowsMode::TwoSided);
    let shadows_only = renderer_common(CastShadowsMode::ShadowsOnly);

    assert_eq!(
        primitive_flags_for_renderer(&off, false, false),
        GPU_PRIMITIVE_FLAG_VISIBLE
    );
    assert_ne!(
        primitive_flags_for_renderer(&two_sided, false, false) & GPU_PRIMITIVE_FLAG_CAST_SHADOWS,
        0
    );
    assert_ne!(
        primitive_flags_for_renderer(&shadows_only, true, false) & GPU_PRIMITIVE_FLAG_CAST_SHADOWS,
        0
    );
    assert_ne!(
        primitive_flags_for_renderer(&shadows_only, true, false)
            & GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM,
        0
    );
}

#[test]
fn render_temporally_unsafe_bounds_force_hzb_visibility() {
    let common = renderer_common(CastShadowsMode::On);

    let flags = primitive_flags_for_renderer(&common, true, true);

    assert_ne!(flags & GPU_PRIMITIVE_FLAG_FORCE_HZB_VISIBLE, 0);
    assert_ne!(flags & GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM, 0);
}

#[test]
fn render_non_skinned_first_frame_seeds_zero_velocity_history() {
    assert!(velocity_history_is_available(false, false));
    assert!(!velocity_history_is_available(true, false));
    assert!(velocity_history_is_available(true, true));
}

#[test]
fn render_instance_normal_transform_flags_preserve_fast_path_and_affine_correctness() {
    let identity = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let uniform_scale = [
        [2.0, 0.0, 0.0, 0.0],
        [0.0, 2.0, 0.0, 0.0],
        [0.0, 0.0, 2.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let small_uniform_scale = [
        [0.0001, 0.0, 0.0, 0.0],
        [0.0, 0.0001, 0.0, 0.0],
        [0.0, 0.0, 0.0001, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let non_uniform_scale = [
        [2.0, 0.0, 0.0, 0.0],
        [0.0, 3.0, 0.0, 0.0],
        [0.0, 0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let shear = [
        [1.0, 0.0, 0.0, 0.0],
        [0.5, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let mirrored = [
        [-1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let degenerate = [
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let non_finite = [
        [f32::NAN, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    assert_eq!(normal_transform_flags_for_model_matrix(&identity), 0);
    assert_eq!(normal_transform_flags_for_model_matrix(&uniform_scale), 0);
    assert_eq!(
        normal_transform_flags_for_model_matrix(&small_uniform_scale),
        0
    );
    assert_eq!(
        normal_transform_flags_for_model_matrix(&non_uniform_scale),
        GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM
    );
    assert_eq!(
        normal_transform_flags_for_model_matrix(&shear),
        GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM | GPU_INSTANCE_FLAG_NON_ORTHOGONAL_TRANSFORM
    );
    assert_eq!(
        normal_transform_flags_for_model_matrix(&mirrored),
        GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT
    );
    assert!(!normal_transform_flags_reverse_raster_winding(
        normal_transform_flags_for_model_matrix(&identity)
    ));
    assert!(normal_transform_flags_reverse_raster_winding(
        normal_transform_flags_for_model_matrix(&mirrored)
    ));
    assert_eq!(
        normal_transform_flags_for_model_matrix(&degenerate),
        GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM
    );
    assert_eq!(
        normal_transform_flags_for_model_matrix(&non_finite),
        GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM
    );
}

#[test]
fn render_instance_normal_transform_matches_inverse_transpose_oracle() {
    let normal = normalize3([0.3, 0.8, -0.2]);
    let transforms = [
        [
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 3.0, 0.0, 0.0],
            [0.0, 0.0, 4.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.5, 1.0, 0.0, 0.0],
            [0.2, -0.3, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        [
            [-2.0, 0.0, 0.0, 0.0],
            [0.25, 3.0, 0.0, 0.0],
            [0.0, 0.0, 4.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        [
            [-1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    ];

    for transform in transforms {
        let flags = normal_transform_flags_for_model_matrix(&transform);
        let selected = normalize3(shader_equivalent_normal_direction(transform, normal, flags));
        let oracle = normalize3(inverse_transpose_normal_direction(transform, normal));
        assert_vec3_approx_eq(selected, oracle, 1.0e-5);
    }
}

#[test]
fn render_mirrored_affine_transform_preserves_tbn_handedness() {
    let transform = [
        [-2.0, 0.0, 0.0, 0.0],
        [0.25, 3.0, 0.0, 0.0],
        [0.0, 0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let normal = [0.0, 0.0, 1.0];
    let tangent = [1.0, 0.0, 0.0];
    let bitangent = [0.0, 1.0, 0.0];
    let flags = normal_transform_flags_for_model_matrix(&transform);

    assert_ne!(flags & GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM, 0);
    assert_ne!(flags & GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT, 0);
    let transformed_normal =
        normalize3(shader_equivalent_normal_direction(transform, normal, flags));
    let transformed_tangent = normalize3(linear_transform_direction(transform, tangent));
    let handedness = if flags & GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT != 0 {
        -1.0
    } else {
        1.0
    };
    let reconstructed_bitangent = normalize3(scale3(
        cross3(transformed_normal, transformed_tangent),
        handedness,
    ));
    let expected_bitangent = normalize3(linear_transform_direction(transform, bitangent));

    assert_vec3_approx_eq(reconstructed_bitangent, expected_bitangent, 1.0e-5);
}

#[test]
fn optimization_wave_20260824rs_runtime09f2_gpu_scene_lookup_preserves_first_match() {
    let first = LightmapInstanceSlot {
        atlas_page: 0,
        uv_rect: Vec4::new(0.5, 0.5, 0.0, 0.0),
    };
    let replacement = LightmapInstanceSlot {
        atlas_page: 0,
        uv_rect: Vec4::new(0.25, 0.25, 0.5, 0.5),
    };
    let contract = LightmapConsumeContract::new(
        1,
        ResourceId::from_stable_label("res://tests/lightmap-array"),
        LightmapAtlasDescriptor {
            page_size: 4,
            page_count: 1,
            format: LightmapAtlasFormat::Rgba16Float,
        },
        vec![(7, first), (7, replacement)],
    );

    assert_eq!(contract.slot_for_instance(7), Some(first));
}

fn renderer_common(cast_shadows: CastShadowsMode) -> RendererCommon {
    RendererCommon {
        cast_shadows,
        ..RendererCommon::default()
    }
}

fn shader_equivalent_normal_direction(
    transform: [[f32; 4]; 4],
    normal: [f32; 3],
    flags: u32,
) -> [f32; 3] {
    if flags & GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM != 0 {
        return [0.0; 3];
    }
    let x = [transform[0][0], transform[0][1], transform[0][2]];
    let y = [transform[1][0], transform[1][1], transform[1][2]];
    let z = [transform[2][0], transform[2][1], transform[2][2]];
    if flags & GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM == 0 {
        return mat3_column_multiply(x, y, z, normal);
    }
    let mut direction = add3(
        add3(
            scale3(cross3(y, z), normal[0]),
            scale3(cross3(z, x), normal[1]),
        ),
        scale3(cross3(x, y), normal[2]),
    );
    if flags & GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT != 0 {
        direction = scale3(direction, -1.0);
    }
    direction
}

fn inverse_transpose_normal_direction(transform: [[f32; 4]; 4], normal: [f32; 3]) -> [f32; 3] {
    let x = [transform[0][0], transform[0][1], transform[0][2]];
    let y = [transform[1][0], transform[1][1], transform[1][2]];
    let z = [transform[2][0], transform[2][1], transform[2][2]];
    let adjugate_normal = add3(
        add3(
            scale3(cross3(y, z), normal[0]),
            scale3(cross3(z, x), normal[1]),
        ),
        scale3(cross3(x, y), normal[2]),
    );
    scale3(adjugate_normal, 1.0 / dot3(x, cross3(y, z)))
}

fn mat3_column_multiply(x: [f32; 3], y: [f32; 3], z: [f32; 3], value: [f32; 3]) -> [f32; 3] {
    add3(
        add3(scale3(x, value[0]), scale3(y, value[1])),
        scale3(z, value[2]),
    )
}

fn linear_transform_direction(transform: [[f32; 4]; 4], value: [f32; 3]) -> [f32; 3] {
    mat3_column_multiply(
        [transform[0][0], transform[0][1], transform[0][2]],
        [transform[1][0], transform[1][1], transform[1][2]],
        [transform[2][0], transform[2][1], transform[2][2]],
        value,
    )
}

fn normalize3(value: [f32; 3]) -> [f32; 3] {
    let length = dot3(value, value).sqrt();
    scale3(value, 1.0 / length)
}

fn add3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn scale3(value: [f32; 3], scale: f32) -> [f32; 3] {
    [value[0] * scale, value[1] * scale, value[2] * scale]
}

fn assert_vec3_approx_eq(actual: [f32; 3], expected: [f32; 3], tolerance: f32) {
    for (actual_component, expected_component) in actual.into_iter().zip(expected) {
        assert!(
            (actual_component - expected_component).abs() <= tolerance,
            "actual {actual:?} differs from expected {expected:?}"
        );
    }
}
