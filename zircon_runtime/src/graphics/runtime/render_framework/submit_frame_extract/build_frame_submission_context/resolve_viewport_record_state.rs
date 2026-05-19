use crate::core::framework::render::{
    AdvancedProfileRuntimePlan, CorePipelineKind, RenderFrameExtract, RenderFrameworkError,
    RenderPipelineHandle, RenderProductFeature, RenderProductProfile, RenderProfileBundle,
    RenderQualityProfile, RenderViewportHandle, SolariProviderAvailability, SolariRuntimeReport,
};

use crate::RenderPipelineAsset;

use super::super::super::capability_validation::validate_quality_profile_capabilities;
use super::super::super::compile_options_for_profile::compile_options_for_profile;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::viewport_record_state::ViewportRecordState;

pub(super) fn resolve_viewport_record_state(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: &RenderFrameExtract,
) -> Result<ViewportRecordState, RenderFrameworkError> {
    let state = server.lock_state();
    let (
        size,
        pipeline_handle,
        viewport_generation,
        quality_profile,
        previous_visibility,
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
        (
            record.size(),
            pipeline_handle,
            record.generation(),
            record.quality_profile().map(|profile| profile.name.clone()),
            record.history().map(|history| history.visibility().clone()),
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
        quality_profile,
        previous_visibility,
        pipeline_asset,
        compile_options,
        advanced_runtime_plan,
        solari_runtime_report,
        capabilities,
        predicted_generation,
    ))
}

fn default_pipeline_for_extract(extract: &RenderFrameExtract) -> RenderPipelineHandle {
    match extract.view.core_pipeline {
        CorePipelineKind::Core2d => RenderPipelineAsset::default_core2d().handle,
        CorePipelineKind::Core3d => RenderPipelineAsset::default_forward_plus().handle,
    }
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
    use crate::core::framework::render::{
        RenderProductFeature, RenderProductProfile, RenderQualityProfile,
    };

    use super::runtime_profile_bundle_for_quality_profile;

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
}
