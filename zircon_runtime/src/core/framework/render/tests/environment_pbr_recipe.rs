use super::*;

#[test]
fn canonical_environment_pbr_recipe_composes_artifact_and_device_global_identities() {
    let recipe = CANONICAL_ENVIRONMENT_PBR_RECIPE;
    let asset = recipe.asset_recipe_identity();
    let runtime = recipe.runtime_recipe_identity();

    assert_ne!(asset.ibl_bake_recipe(), runtime.ibl_bake_recipe());
    assert_eq!(
        asset.brdf_lut_recipe(),
        CANONICAL_ENVIRONMENT_BRDF_LUT_RECIPE.identity()
    );
    assert_eq!(asset.brdf_lut_recipe(), runtime.brdf_lut_recipe());
    assert_eq!(
        asset.base_lobe_energy_mode(),
        EnvironmentPbrEnergyMode::SingleScatterSplitSum
    );
    assert_eq!(
        asset.base_lobe_energy_mode(),
        runtime.base_lobe_energy_mode()
    );
}

#[test]
fn canonical_environment_brdf_lut_recipe_owns_the_unreal_baseline_domain() {
    let recipe = CANONICAL_ENVIRONMENT_BRDF_LUT_RECIPE;

    assert_eq!(recipe.algorithm_version(), 2026_08_31_0001);
    assert_eq!(recipe.extent(), [128, 32]);
    assert_eq!(recipe.sample_count(), 128);
    assert_eq!(
        recipe.integrator(),
        EnvironmentBrdfLutIntegrator::GgxJointSmithSplitSum
    );
    assert_eq!(recipe.output_format(), EnvironmentBrdfLutFormat::Rg16Float);
    assert_eq!(recipe.expected_byte_len(), 16_384);
    assert_eq!(ENVIRONMENT_BRDF_LUT_WIDTH, recipe.width());
    assert_eq!(ENVIRONMENT_BRDF_LUT_HEIGHT, recipe.height());
    assert_eq!(ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT, recipe.sample_count());
}
