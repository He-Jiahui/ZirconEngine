use super::*;

#[test]
fn canonical_recipe_selects_the_first_mip_at_or_below_its_diffuse_face_size() {
    assert_eq!(
        CANONICAL_IBL_BAKE_RECIPE.diffuse_source_mip_level(256, 9),
        3
    );
    assert_eq!(CANONICAL_IBL_BAKE_RECIPE.diffuse_source_mip_level(32, 6), 0);
    assert_eq!(CANONICAL_IBL_BAKE_RECIPE.diffuse_source_mip_level(16, 5), 0);
}

#[test]
fn canonical_recipe_never_selects_a_mip_outside_the_declared_chain() {
    assert_eq!(
        CANONICAL_IBL_BAKE_RECIPE.diffuse_source_mip_level(256, 0),
        0
    );
    assert_eq!(CANONICAL_IBL_BAKE_RECIPE.diffuse_source_mip_level(1, 8), 0);
}

#[test]
fn canonical_recipe_owns_cpu_and_gpu_sampling_policy() {
    let recipe = CANONICAL_IBL_BAKE_RECIPE;

    assert_eq!(recipe.pmrem_sample_count(0.0, 0), 32);
    assert_eq!(recipe.pmrem_sample_count(0.5, 4), 64);
    assert_eq!(recipe.pmrem_sample_count(0.75, 6), 128);
    assert_eq!(recipe.runtime_diffuse_sample_count(), 64);
    assert_eq!(recipe.full_roughness_cosine_threshold(), 0.99);
    assert_eq!(recipe.fis_solid_angle_texel_scale(), 2.0);
}

#[test]
fn canonical_recipe_keeps_cpu_and_runtime_integrator_identities_distinct() {
    let asset = CANONICAL_IBL_BAKE_RECIPE.asset_recipe_identity();
    let runtime = CANONICAL_IBL_BAKE_RECIPE.runtime_recipe_identity();

    assert_ne!(asset, runtime);
    assert_eq!(asset.algorithm_version(), runtime.algorithm_version());
    assert_eq!(asset.pmrem_integrator(), runtime.pmrem_integrator());
    assert_eq!(asset.output_format(), IblBakeOutputFormat::Rgba16Float);
    assert_eq!(
        CANONICAL_IBL_BAKE_RECIPE.diffuse_representation(),
        IblBakeDiffuseRepresentation::Sh9WithOptionalIrradianceCube
    );
}

#[test]
fn canonical_recipe_matches_unreal_reflection_capture_roughness_mapping() {
    let recipe = CANONICAL_IBL_BAKE_RECIPE;
    let mip_count = 8;
    for mip_level in 1..(mip_count - 2) {
        let roughness = recipe.roughness_from_pmrem_mip(mip_level, mip_count);
        let resolved_mip = recipe.pmrem_mip_from_roughness(roughness, mip_count);
        assert!((resolved_mip - mip_level as Real).abs() <= 0.0001);
    }
    assert_eq!(
        recipe.pmrem_mip_from_roughness(1.0, mip_count),
        (mip_count - 3) as Real
    );
    assert_eq!(
        recipe.roughness_from_pmrem_mip(mip_count - 3, mip_count),
        1.0
    );
}
