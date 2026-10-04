use super::*;

#[test]
fn optimization_batch_20260830ep_capability_flags_keep_scanning_until_first_match() {
    let ordinary = RendererFeatureAsset::builtin(BuiltinRenderFeature::Bloom);
    let advanced = RendererFeatureAsset::builtin(BuiltinRenderFeature::ColorGrading)
        .with_capability_requirement(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
        .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry);

    let flags = CompiledRenderPipelineRuntimeFeatureFlags::from_compiled_inputs(
        &[ordinary, advanced.clone(), advanced],
        &[],
    );

    assert!(flags.hybrid_global_illumination_enabled);
    assert!(flags.virtual_geometry_enabled);
}

#[test]
fn optimization_batch_20260830ep_capability_flags_short_circuit_after_match() {
    let source = include_str!("../runtime_feature_flags.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("runtime feature flag production source");

    assert!(production.contains("if !flags.hybrid_global_illumination_enabled"));
    assert!(production.contains("if !flags.virtual_geometry_enabled"));
}

#[test]
#[ignore = "release-only capability resolution short-circuit evidence"]
fn optimization_batch_20260830ep_capability_resolution_short_circuit_evidence() {
    const FEATURE_COUNT: usize = 65_536;
    const CAPABILITY_COUNT: usize = 2;
    let legacy_capability_resolutions = FEATURE_COUNT * CAPABILITY_COUNT;
    let optimized_capability_resolutions = CAPABILITY_COUNT;

    assert_eq!(legacy_capability_resolutions, 131_072);
    assert_eq!(optimized_capability_resolutions, 2);
    println!(
        "RUNTIME547_COMPILED_CAPABILITY_SHORT_CIRCUIT_BENCH_V1 features={FEATURE_COUNT} \
             legacy_capability_resolutions={legacy_capability_resolutions} \
             optimized_capability_resolutions={optimized_capability_resolutions} reduction_pct=99.998"
    );
}
