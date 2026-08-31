use crate::core::framework::render::{
    AdvancedProfileRuntimePlan, RenderFrameExtract, RenderFrameworkError, RenderPipelineHandle,
    RenderProductFeature, RenderProductProfile, RenderProfileBundle, RenderQualityProfile,
    RenderViewportHandle, SolariProviderAvailability, SolariRuntimeReport,
};

use crate::graphics::RenderPipelineAsset;

use super::super::super::capability_validation::validate_quality_profile_capabilities;
use super::super::super::compile_options_for_profile::compile_options_for_profile;
use super::super::super::wgpu_render_framework::WgpuRenderFrameworkAccess;
use super::camera_history_key::camera_history_key_for_extract;
use super::viewport_record_state::ViewportRecordState;

pub(super) fn resolve_viewport_record_state(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    extract: &RenderFrameExtract,
) -> Result<ViewportRecordState, RenderFrameworkError> {
    let state = framework.lock_state();
    let camera_history_key = camera_history_key_for_extract(extract);
    let (
        size,
        pipeline_handle,
        viewport_generation,
        temporal_frame_index,
        quality_profile,
        quality_profile_texture_mip_bias,
        quality_profile_texture_max_anisotropy,
        shader_quality,
        quality_profile_taa_quality,
        previous_visibility,
        previous_static_index,
        previous_dynamic_index,
        previous_motion_vector_camera,
        previous_particle_sprites,
        compile_options,
        advanced_runtime_plan,
        solari_runtime_report,
        capabilities,
        predicted_generation,
    ) = {
        let record =
            state
                .viewports
                .get(&viewport)
                .ok_or(RenderFrameworkError::UnknownViewport {
                    viewport: viewport.raw(),
                })?;
        let pipeline_handle = record.effective_pipeline(default_pipeline_for_extract(extract));
        if let Some(profile) = record.quality_profile() {
            validate_quality_profile_capabilities(
                Some(pipeline_handle),
                profile,
                &state.stats.capabilities,
            )?;
        }
        let advanced_provider_availability = state.stats.advanced_provider_availability.clone();
        let runtime_profile_bundle =
            runtime_profile_bundle_for_quality_profile(record.quality_profile());
        let advanced_runtime_plan = AdvancedProfileRuntimePlan::from_profile_bundle(
            &runtime_profile_bundle,
            &state.stats.capabilities,
            &advanced_provider_availability,
        );
        let solari_provider_availability = state
            .solari_runtime_provider
            .as_ref()
            .map(|provider| provider.provider().availability(provider.provider_id()))
            .unwrap_or_else(SolariProviderAvailability::missing);
        let solari_runtime_report = solari_runtime_report_for_quality_profile(
            record.quality_profile(),
            &runtime_profile_bundle,
            &state.stats.capabilities,
            &solari_provider_availability,
        );
        let previous_history = record.history(&camera_history_key);
        (
            record.size(),
            pipeline_handle,
            record.generation(),
            record.temporal_frame_index(),
            record.quality_profile().map(|profile| profile.name.clone()),
            record
                .quality_profile()
                .map(|profile| profile.texture_mip_bias)
                .unwrap_or_default(),
            record
                .quality_profile()
                .map(|profile| profile.texture_max_anisotropy)
                .unwrap_or(16),
            record
                .quality_profile()
                .map(|profile| profile.shader_quality)
                .unwrap_or_default(),
            record.quality_profile().map(|profile| profile.taa_quality),
            previous_history.map(|history| history.visibility().clone()),
            previous_history.map(|history| history.static_index().clone()),
            previous_history.map(|history| history.dynamic_index().clone()),
            record.motion_vector_camera(&camera_history_key).cloned(),
            record
                .particle_previous_sprites(&camera_history_key)
                .to_vec(),
            compile_options_for_profile(
                record.quality_profile(),
                &state.stats.capabilities,
                &advanced_provider_availability,
            ),
            advanced_runtime_plan,
            solari_runtime_report,
            state.stats.capabilities.clone(),
            state.stats.last_generation.unwrap_or(0) + 1,
        )
    };
    let pipeline_asset = state.pipelines.get(&pipeline_handle).cloned().ok_or(
        RenderFrameworkError::UnknownPipeline {
            pipeline: pipeline_handle.raw(),
        },
    )?;

    Ok(ViewportRecordState::new(
        size,
        pipeline_handle,
        viewport_generation,
        temporal_frame_index,
        quality_profile,
        quality_profile_texture_mip_bias,
        quality_profile_texture_max_anisotropy,
        shader_quality,
        quality_profile_taa_quality,
        previous_visibility,
        previous_static_index,
        previous_dynamic_index,
        previous_motion_vector_camera,
        previous_particle_sprites,
        pipeline_asset,
        compile_options,
        advanced_runtime_plan,
        solari_runtime_report,
        capabilities,
        predicted_generation,
    ))
}

fn default_pipeline_for_extract(extract: &RenderFrameExtract) -> RenderPipelineHandle {
    RenderPipelineAsset::default_handle_for_core_pipeline(extract.view.core_pipeline)
}

fn runtime_profile_bundle_for_quality_profile(
    profile: Option<&RenderQualityProfile>,
) -> RenderProfileBundle {
    let Some(profile) = profile else {
        return RenderProfileBundle::default_render();
    };
    if profile.features.solari {
        let mut features = RenderProfileBundle::solari_experimental()
            .features()
            .to_vec();
        if !profile.features.virtual_geometry {
            features.retain(|feature| *feature != RenderProductFeature::VirtualGeometry);
        }
        if !profile.features.hybrid_global_illumination {
            features.retain(|feature| *feature != RenderProductFeature::HybridGlobalIllumination);
        }
        return RenderProfileBundle::new(RenderProductProfile::SolariExperimental)
            .with_includes([RenderProductProfile::AdvancedRender])
            .with_features(features);
    }
    if !profile.features.virtual_geometry && !profile.features.hybrid_global_illumination {
        return RenderProfileBundle::default_render();
    }

    let mut features = RenderProfileBundle::advanced_render().features().to_vec();
    if !profile.features.virtual_geometry {
        features.retain(|feature| *feature != RenderProductFeature::VirtualGeometry);
    }
    if !profile.features.hybrid_global_illumination {
        features.retain(|feature| *feature != RenderProductFeature::HybridGlobalIllumination);
    }

    RenderProfileBundle::new(RenderProductProfile::AdvancedRender)
        .with_includes([RenderProductProfile::DefaultRender])
        .with_features(features)
}

fn solari_runtime_report_for_quality_profile(
    profile: Option<&RenderQualityProfile>,
    bundle: &RenderProfileBundle,
    capabilities: &crate::core::framework::render::RenderCapabilitySummary,
    availability: &SolariProviderAvailability,
) -> SolariRuntimeReport {
    let requested = bundle.has_feature(RenderProductFeature::Solari);
    let settings = profile
        .map(|profile| profile.solari.clone())
        .unwrap_or_default();
    SolariRuntimeReport::from_inputs(requested, settings, capabilities, availability)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::render::{
        RenderProductFeature, RenderProductProfile, RenderQualityProfile,
    };

    use super::runtime_profile_bundle_for_quality_profile;

    #[test]
    fn default_pipeline_resolution_does_not_construct_builtin_assets() {
        let source = include_str!("resolve_viewport_record_state.rs");

        assert!(!source.contains(concat!("default_core2d()", ".handle")));
        assert!(!source.contains(concat!("default_forward_plus()", ".handle")));
        assert!(source.contains("default_handle_for_core_pipeline"));
    }

    #[test]
    fn runtime_profile_bundle_for_quality_profile_requests_only_enabled_advanced_features() {
        let profile = RenderQualityProfile::new("vg-only").with_virtual_geometry(true);
        let bundle = runtime_profile_bundle_for_quality_profile(Some(&profile));

        assert_eq!(bundle.profile(), RenderProductProfile::AdvancedRender);
        assert!(bundle.has_feature(RenderProductFeature::VirtualGeometry));
        assert!(!bundle.has_feature(RenderProductFeature::HybridGlobalIllumination));
    }

    #[test]
    fn runtime_profile_bundle_for_quality_profile_defaults_without_advanced_flags() {
        let profile = RenderQualityProfile::new("default");
        let bundle = runtime_profile_bundle_for_quality_profile(Some(&profile));

        assert_eq!(bundle.profile(), RenderProductProfile::DefaultRender);
        assert!(!bundle.has_feature(RenderProductFeature::VirtualGeometry));
        assert!(!bundle.has_feature(RenderProductFeature::HybridGlobalIllumination));
    }

    #[test]
    fn runtime_profile_bundle_for_quality_profile_requests_solari_only_when_enabled() {
        let profile = RenderQualityProfile::new("solari")
            .with_solari(true)
            .with_virtual_geometry(false)
            .with_hybrid_global_illumination(false);
        let bundle = runtime_profile_bundle_for_quality_profile(Some(&profile));

        assert_eq!(bundle.profile(), RenderProductProfile::SolariExperimental);
        assert!(bundle.has_feature(RenderProductFeature::Solari));
        assert!(!bundle.has_feature(RenderProductFeature::VirtualGeometry));
        assert!(!bundle.has_feature(RenderProductFeature::HybridGlobalIllumination));
    }

    #[test]
    fn optimization_batch_dj_viewport_state_uses_one_history_lookup_source() {
        let source = include_str!("resolve_viewport_record_state.rs");
        let function = source
            .split("pub(super) fn resolve_viewport_record_state")
            .nth(1)
            .expect("viewport state resolver")
            .split("fn default_pipeline_for_extract")
            .next()
            .expect("resolver body");

        assert!(function.contains("let previous_history = record.history(&camera_history_key);"));
        assert_eq!(
            function
                .matches("record.history(&camera_history_key)")
                .count(),
            1
        );
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dj_single_viewport_history_lookup_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const LOOKUPS_PER_SAMPLE: usize = 65_536;
        const HISTORY_COUNT: usize = 4_096;

        let histories = (0..HISTORY_COUNT as u64)
            .map(|key| (key, [key, key + 1, key + 2]))
            .collect::<HashMap<_, _>>();
        let keys = (0..LOOKUPS_PER_SAMPLE)
            .map(|index| (index % HISTORY_COUNT) as u64)
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_history_projection(&histories, &keys, true));
                optimized_samples.push(measure_history_projection(&histories, &keys, false));
            } else {
                optimized_samples.push(measure_history_projection(&histories, &keys, false));
                legacy_samples.push(measure_history_projection(&histories, &keys, true));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME418_SINGLE_VIEWPORT_HISTORY_LOOKUP_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "single viewport history lookup p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_history_projection(
        histories: &HashMap<u64, [u64; 3]>,
        keys: &[u64],
        legacy: bool,
    ) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for key in keys {
            if legacy {
                checksum = checksum
                    .wrapping_add(black_box(histories).get(black_box(key)).unwrap()[0])
                    .wrapping_add(black_box(histories).get(black_box(key)).unwrap()[1])
                    .wrapping_add(black_box(histories).get(black_box(key)).unwrap()[2]);
            } else {
                let history = black_box(histories).get(black_box(key)).unwrap();
                checksum = checksum
                    .wrapping_add(history[0])
                    .wrapping_add(history[1])
                    .wrapping_add(history[2]);
            }
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = samples
            .len()
            .saturating_mul(95)
            .div_ceil(100)
            .saturating_sub(1);
        samples[index]
    }
}
