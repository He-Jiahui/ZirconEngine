use super::*;
use crate::core::framework::render::{
    build_environment_brdf_lut_with_extent, encode_rg16f_texels, EnvironmentBrdfLutFormat,
    EnvironmentBrdfLutIntegrator, EnvironmentPbrEnergyMode, CANONICAL_ENVIRONMENT_PBR_RECIPE,
};

#[test]
fn black_cube_payload_contains_six_rgba16f_faces() {
    let payload = payloads::black_cube_rgba16float_bytes();

    assert_eq!(payload.len(), 48);
    assert!(payload
        .chunks_exact(8)
        .all(|face| face == [0, 0, 0, 0, 0, 0, 0, 60]));
}

#[test]
fn generation_owner_batches_black_cube_and_brdf_under_one_texture_ticket() {
    let source = include_str!("../system_texture_generation_owner.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or_default();
    let resources = include_str!("../system_texture_generation_owner/resources.rs");

    assert_eq!(resources.matches("push_upload(").count(), 7);
    assert_eq!(resources.matches("push_solid_upload(").count(), 6);
    assert!(resources.contains("SYSTEM_TEXTURE_UPLOAD_COUNT: usize = 10"));
    assert!(resources.contains("SYSTEM_TEXTURE_UPLOAD_BYTES: u64 = 16_768"));
    assert_eq!(
        production
            .matches("enqueue_native_texture_upload_batch(uploads)")
            .count(),
        1
    );
    assert!(resources.contains("with_depth_or_array_layers(BLACK_CUBE_FACE_COUNT)"));
    assert!(!production.contains("queue.write_texture"));
    assert!(!resources.contains("queue.write_texture"));
}

#[test]
fn production_uses_the_versioned_builtin_brdf_lut_without_runtime_integration() {
    let source = include_str!("../system_texture_generation_owner.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or_default();

    assert!(production.contains("builtin_environment_brdf_lut_rg16float_bytes"));
    assert!(!production.contains("build_environment_brdf_lut_with_extent"));
    assert!(!production.contains("encode_rg16f_texels"));
    assert!(!production.contains("ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT"));
}

#[test]
fn builtin_brdf_lut_metadata_and_bytes_match_the_canonical_generator() {
    use sha2::{Digest, Sha256};

    let pbr_recipe = CANONICAL_ENVIRONMENT_PBR_RECIPE;
    let recipe = pbr_recipe.brdf_lut_recipe();
    let builtin = payloads::builtin_environment_brdf_lut_rg16float_bytes();
    let generated = encode_rg16f_texels(&build_environment_brdf_lut_with_extent(
        recipe.width(),
        recipe.height(),
        recipe.sample_count(),
    ));

    assert_eq!(recipe.algorithm_version(), 2026_08_31_0001);
    assert_eq!(recipe.extent(), [128, 32]);
    assert_eq!(recipe.sample_count(), 128);
    assert_eq!(
        recipe.integrator(),
        EnvironmentBrdfLutIntegrator::GgxJointSmithSplitSum
    );
    assert_eq!(
        pbr_recipe.base_lobe_energy_mode(),
        EnvironmentPbrEnergyMode::SingleScatterSplitSum
    );
    assert_eq!(recipe.output_format(), EnvironmentBrdfLutFormat::Rg16Float);
    assert_eq!(
        resources::environment_brdf_lut_wgpu_format(recipe.output_format()),
        wgpu::TextureFormat::Rg16Float
    );
    assert_eq!(builtin.len(), recipe.expected_byte_len());
    assert_eq!(
        builtin.len(),
        payloads::ENVIRONMENT_BRDF_LUT_ARTIFACT_BYTE_LEN
    );
    assert_eq!(builtin.as_ref(), generated);
    assert_eq!(
        Sha256::digest(builtin.as_ref()).as_slice(),
        &payloads::ENVIRONMENT_BRDF_LUT_ARTIFACT_SHA256
    );
}
