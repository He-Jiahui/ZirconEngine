use crate::core::framework::render::{
    RenderMaterialTextureTransform, StandardPbrMaterialFeatures, STANDARD_MATERIAL_MIN_ROUGHNESS,
};

use super::{
    standard_material_uniform_contents_from_values,
    standard_material_uniform_contents_from_values_with_normal_details,
    standard_material_uniform_contents_from_values_with_normal_scale,
    GPU_MATERIAL_UNIFORM_MIN_SIZE, STANDARD_TEXTURE_TRANSFORM_COUNT,
};

#[test]
fn standard_material_uniform_packs_pbr_scalars_without_property_schema_offsets() {
    let bytes: [u8; GPU_MATERIAL_UNIFORM_MIN_SIZE] = standard_material_uniform_contents_from_values(
        1.4,
        0.0,
        0.25,
        [0.25, -1.0, 2.0],
        true,
        0,
        1.4,
        0,
        Some(1.4),
        [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
        [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
        &StandardPbrMaterialFeatures::default(),
    );

    assert_eq!(bytes.len(), GPU_MATERIAL_UNIFORM_MIN_SIZE);
    assert_eq!(f32_at(&bytes, 0), 1.0);
    assert_eq!(f32_at(&bytes, 4), STANDARD_MATERIAL_MIN_ROUGHNESS);
    assert_eq!(f32_at(&bytes, 8), 0.25);
    assert_eq!(f32_at(&bytes, 12), 1.0);
    assert_eq!(f32_at(&bytes, 16), 0.25);
    assert_eq!(f32_at(&bytes, 20), 0.0);
    assert_eq!(f32_at(&bytes, 24), 2.0);
    assert_eq!(f32_at(&bytes, 128), 1.0);
    assert_eq!(f32_at(&bytes, 132), 0.0);
    assert_eq!(f32_at(&bytes, 136), 1.0);
}

#[test]
fn standard_material_uniform_clamps_occlusion_strength_at_data0_z() {
    for (occlusion_strength, expected) in [(f32::NAN, 1.0), (-1.0, 0.0), (0.25, 0.25), (1.4, 1.0)] {
        let bytes = standard_material_uniform_contents_from_values(
            0.5,
            0.5,
            occlusion_strength,
            [0.0; 3],
            false,
            2,
            0.0,
            0,
            None,
            [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
            [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
            &StandardPbrMaterialFeatures::default(),
        );

        assert_eq!(f32_at(&bytes, 8), expected);
    }
}

#[test]
fn standard_material_uniform_packs_finite_normal_scale_at_data12_x() {
    let bytes = standard_material_uniform_contents_from_values_with_normal_scale(
        0.5,
        0.5,
        1.0,
        [0.0; 3],
        false,
        2,
        0.0,
        0,
        None,
        [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
        [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
        &StandardPbrMaterialFeatures::default(),
        0.35,
    );

    assert_eq!(bytes.len(), GPU_MATERIAL_UNIFORM_MIN_SIZE);
    assert_eq!(f32_at(&bytes, 192), 0.35);

    let non_finite = standard_material_uniform_contents_from_values_with_normal_scale(
        0.5,
        0.5,
        1.0,
        [0.0; 3],
        false,
        2,
        0.0,
        0,
        None,
        [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
        [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
        &StandardPbrMaterialFeatures::default(),
        f32::NAN,
    );
    assert_eq!(f32_at(&non_finite, 192), 1.0);
}

#[test]
fn standard_material_uniform_packs_cpu_derived_dielectric_f0_at_data12_y() {
    let bytes = standard_material_uniform_contents_from_values_with_normal_scale(
        0.5,
        0.5,
        1.0,
        [0.0; 3],
        false,
        2,
        0.0,
        0,
        None,
        [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
        [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
        &StandardPbrMaterialFeatures {
            ior: 2.5,
            ..Default::default()
        },
        1.0,
    );

    assert!((f32_at(&bytes, 196) - 0.18367347).abs() < 0.000001);
    assert_eq!(f32_at(&bytes, 200), 1.0);
    assert_eq!(f32_at(&bytes, 204), 0.0);
}

#[test]
fn standard_material_uniform_packs_per_slot_texture_transforms() {
    let bytes = standard_material_uniform_contents_from_values(
        0.5,
        0.5,
        1.0,
        [0.0, 0.0, 0.0],
        false,
        2,
        0.25,
        0,
        Some(0.625),
        [
            transform([2.0, 3.0], [0.25, 0.5]),
            transform_with_rotation([4.0, 5.0], [0.125, 0.25], std::f32::consts::FRAC_PI_2),
            transform([6.0, 7.0], [0.75, 0.875]),
            transform([8.0, 9.0], [1.25, 1.5]),
            transform_with_rotation([f32::NAN, 11.0], [f32::INFINITY, -0.25], f32::NAN),
        ],
        [1, 0, 2, u32::MAX, 1],
        &StandardPbrMaterialFeatures::default(),
    );

    assert_eq!(vec4_at(&bytes, 32), [2.0, 3.0, 0.25, 0.5]);
    assert_eq!(vec4_at(&bytes, 48), [4.0, 5.0, 0.125, 0.25]);
    assert_eq!(vec4_at(&bytes, 64), [6.0, 7.0, 0.75, 0.875]);
    assert_eq!(vec4_at(&bytes, 80), [8.0, 9.0, 1.25, 1.5]);
    assert_eq!(vec4_at(&bytes, 96), [1.0, 11.0, 0.0, -0.25]);
    assert_eq!(f32_at(&bytes, 28), 0.0);
    assert_eq!(vec4_at(&bytes, 112), [17.0, 1.0, 1.0, 0.0]);
    assert_eq!(vec4_at(&bytes, 128), [0.25, 2.0 / 255.0, 0.625, 0.0]);
    assert_eq!(f32_at(&bytes, 208), 1.0);
    assert_eq!(f32_at(&bytes, 212), 0.0);
    assert_eq!(vec4_at(&bytes, 224), [1.0, 0.0, 1.0, 0.0]);
    assert_eq!(vec4_at(&bytes, 240), [1.0, 0.0, 1.0, 0.0]);
    assert_f32_near(f32_at(&bytes, 216), 0.0);
    assert_f32_near(f32_at(&bytes, 220), 1.0);
}

#[test]
fn standard_material_uniform_packs_clearcoat_normal_without_growing_the_256_byte_abi() {
    let bytes = standard_material_uniform_contents_from_values_with_normal_details(
        0.5,
        0.5,
        1.0,
        [0.0; 3],
        false,
        2,
        0.0,
        0,
        None,
        [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
        [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
        &StandardPbrMaterialFeatures::default(),
        0.8,
        transform_with_rotation([0.5, 0.75], [0.1, 0.2], 0.4),
        1,
        0.35,
    );

    assert_eq!(bytes.len(), GPU_MATERIAL_UNIFORM_MIN_SIZE);
    assert_eq!(f32_at(&bytes, 28), 0.2);
    assert_eq!(vec4_at(&bytes, 112), [32.0, 0.5, 0.75, 0.1]);
    assert_eq!(f32_at(&bytes, 192), 0.8);
    assert_eq!(f32_at(&bytes, 200), 0.35);
    // BUG: [CR-R02-runtime_wave12_graphics_resource_residency-0001] 此 0.4 弧度夹具把 cos/sin 期望交换，首个近似断言必失败；helper、生产打包与 WGSL 均约定 204 字节为 cos、248 字节为 sin。
    assert_f32_near(f32_at(&bytes, 204), 0.4_f32.sin());
    assert_f32_near(f32_at(&bytes, 248), 0.4_f32.cos());
}

#[test]
fn standard_material_uniform_packs_shading_model_id_for_gbuffer_encoding() {
    let bytes = standard_material_uniform_contents_from_values(
        0.5,
        0.5,
        1.0,
        [0.0, 0.0, 0.0],
        false,
        16,
        0.0,
        0,
        None,
        [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
        [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
        &StandardPbrMaterialFeatures::default(),
    );

    assert_eq!(f32_at(&bytes, 132), 16.0 / 255.0);
}

#[test]
fn standard_material_uniform_packs_alpha_cutoff_for_template_clip() {
    for (alpha_cutoff, expected) in [
        (None, 0.0),
        (Some(-1.0), 0.0),
        (Some(0.42), 0.42),
        (Some(1.4), 1.0),
        (Some(f32::NAN), 0.0),
    ] {
        let bytes = standard_material_uniform_contents_from_values(
            0.5,
            0.5,
            1.0,
            [0.0, 0.0, 0.0],
            false,
            2,
            0.0,
            0,
            alpha_cutoff,
            [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
            [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
            &StandardPbrMaterialFeatures::default(),
        );

        assert_eq!(f32_at(&bytes, 136), expected);
    }
}

#[test]
fn standard_material_uniform_packs_subsurface_profile_index() {
    let bytes = standard_material_uniform_contents_from_values(
        0.0,
        0.5,
        1.0,
        [0.0; 3],
        false,
        16,
        0.0,
        11,
        None,
        [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
        [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
        &StandardPbrMaterialFeatures::default(),
    );

    assert_eq!(f32_at(&bytes, 140), 11.0 / 255.0);
}

#[test]
fn render_advanced_material_uniform_packs_clearcoat_anisotropy_and_transmission() {
    let features = StandardPbrMaterialFeatures {
        clearcoat: 0.8,
        clearcoat_perceptual_roughness: 0.2,
        anisotropy_strength: 0.65,
        anisotropy_rotation: 1.25,
        specular_transmission: 0.7,
        diffuse_transmission: 0.15,
        thickness: 0.4,
        ior: 1.52,
        attenuation_color: [0.8, 0.9, 1.0],
        attenuation_distance: 12.0,
        ..Default::default()
    };
    let bytes = standard_material_uniform_contents_from_values(
        0.0,
        0.5,
        1.0,
        [0.0; 3],
        false,
        2,
        0.0,
        0,
        None,
        [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
        [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
        &features,
    );

    assert_eq!(bytes.len(), GPU_MATERIAL_UNIFORM_MIN_SIZE);
    assert_eq!(vec4_at(&bytes, 144), [0.8, 0.2, 0.65, 1.25]);
    assert_eq!(vec4_at(&bytes, 160), [0.7, 0.15, 0.4, 1.52]);
    assert_eq!(vec4_at(&bytes, 176), [0.8, 0.9, 1.0, 12.0]);
}

fn transform(scale: [f32; 2], offset: [f32; 2]) -> RenderMaterialTextureTransform {
    transform_with_rotation(scale, offset, 0.0)
}

fn transform_with_rotation(
    scale: [f32; 2],
    offset: [f32; 2],
    rotation: f32,
) -> RenderMaterialTextureTransform {
    RenderMaterialTextureTransform {
        scale,
        offset,
        rotation,
    }
}

fn vec4_at(bytes: &[u8], offset: usize) -> [f32; 4] {
    [
        f32_at(bytes, offset),
        f32_at(bytes, offset + 4),
        f32_at(bytes, offset + 8),
        f32_at(bytes, offset + 12),
    ]
}

fn f32_at(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn assert_f32_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.000_001,
        "expected {expected:?}, got {actual:?}"
    );
}
