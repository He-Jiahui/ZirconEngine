use zircon_runtime::core::framework::render::{
    RenderHybridGiCompositePolicy, RenderHybridGiExtract, RenderHybridGiFallbackReason,
    RenderHybridGiMode, RenderHybridGiProfile, HYBRID_GI_SOURCE_BAKED_BASELINE,
    HYBRID_GI_SOURCE_DYNAMIC_DELTA, HYBRID_GI_SOURCE_FULL_DYNAMIC,
};

const POST_PROCESS_SHADER: &str = concat!(
    include_str!("../src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl"),
    "\n",
    include_str!(
        "../src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl"
    )
);

#[test]
fn hybrid_gi_m4_source_policy_excludes_full_dynamic_from_baked_delta_mode() {
    let policy = RenderHybridGiCompositePolicy::baked_baseline_with_dynamic_delta(17, 4);

    assert!(policy.accepts_hybrid_gi_output());
    assert_eq!(policy.baked_light_set_generation(), Some(17));
    assert_eq!(policy.participation_epoch(), 4);
    assert_eq!(
        policy.source_mask(),
        HYBRID_GI_SOURCE_BAKED_BASELINE | HYBRID_GI_SOURCE_DYNAMIC_DELTA
    );
    assert_eq!(policy.source_mask() & HYBRID_GI_SOURCE_FULL_DYNAMIC, 0);
}

#[test]
fn hybrid_gi_m4_post_process_source_ledger_shader_parses_and_validates() {
    let module = naga::front::wgsl::parse_str(POST_PROCESS_SHADER)
        .expect("HybridGI M4 post-process shader must parse");
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );

    validator
        .validate(&module)
        .expect("HybridGI M4 post-process shader must validate");
}

#[test]
fn hybrid_gi_m4_profiles_resolve_distinct_workloads_and_structured_fallback() {
    let profile_settings = [
        (RenderHybridGiProfile::FullyDynamic, (96, 192, 96)),
        (RenderHybridGiProfile::IndoorStatic, (64, 256, 64)),
        (RenderHybridGiProfile::OpenWorld, (64, 192, 128)),
        (RenderHybridGiProfile::Cinematic, (192, 512, 192)),
    ];

    for (profile, expected_budgets) in profile_settings {
        let resolved = RenderHybridGiExtract {
            enabled: true,
            profile,
            ..RenderHybridGiExtract::default()
        }
        .resolved_settings(true);
        assert_eq!(
            (
                resolved.trace_budget,
                resolved.card_budget,
                resolved.voxel_budget
            ),
            expected_budgets
        );
    }

    let fallback = RenderHybridGiExtract {
        enabled: true,
        profile: RenderHybridGiProfile::IndoorStatic,
        ..RenderHybridGiExtract::default()
    }
    .resolved_settings(false);
    assert_eq!(fallback.mode, RenderHybridGiMode::DynamicOnly);
    assert_eq!(
        fallback.fallback_reason,
        Some(RenderHybridGiFallbackReason::BakedLightingUnavailable)
    );
}
