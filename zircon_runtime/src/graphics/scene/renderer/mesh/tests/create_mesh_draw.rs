use super::MaterialBindingBuildProfile;

#[test]
fn material_binding_profile_matches_the_fixed_draw_abi() {
    assert_eq!(
        MaterialBindingBuildProfile::for_residual_draws(1, 0),
        MaterialBindingBuildProfile {
            residual_draw_count: 1,
            sampler_variant_query_count: 6,
            bind_group_creation_count: 2,
            bind_group_entry_projection_count: 26,
            override_uniform_buffer_creation_count: 0,
        }
    );
}

#[test]
fn material_binding_profile_exposes_linear_draw_amplification() {
    assert_eq!(
        MaterialBindingBuildProfile::for_residual_draws(10_000, 375),
        MaterialBindingBuildProfile {
            residual_draw_count: 10_000,
            sampler_variant_query_count: 60_000,
            bind_group_creation_count: 20_000,
            bind_group_entry_projection_count: 260_000,
            override_uniform_buffer_creation_count: 375,
        }
    );
}
