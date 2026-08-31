use crate::core::framework::render::{
    AoSourceSettings, IblBakeArtifactRequest, PostProcessEffectKind, PostProcessStackDescriptor,
    RenderFrameExtract, ShaderQualityTier,
};
use crate::render_graph::QueueLane;

use crate::graphics::feature::{
    BuiltinRenderFeature, RenderFeatureCapabilityRequirement,
    descriptor_only_advanced_slot_requires_capability_opt_in,
};
use crate::graphics::pipeline::declarations::{
    AdvancedLightingCompileInputs, RenderPipelineCompileOptions, RendererFeatureAsset,
    RendererFeatureSource,
};

impl RenderPipelineCompileOptions {
    pub fn with_feature_enabled(mut self, feature: BuiltinRenderFeature) -> Self {
        self.disabled_features.remove(&feature);
        self.enabled_features.insert(feature);
        self
    }

    pub fn with_feature_disabled(mut self, feature: BuiltinRenderFeature) -> Self {
        self.enabled_features.remove(&feature);
        self.disabled_features.insert(feature);
        self
    }

    pub fn with_plugin_feature_enabled(mut self, feature_name: impl Into<String>) -> Self {
        let feature_name = feature_name.into();
        self.disabled_plugin_features.remove(&feature_name);
        self
    }

    pub fn with_plugin_feature_disabled(mut self, feature_name: impl Into<String>) -> Self {
        self.disabled_plugin_features.insert(feature_name.into());
        self
    }

    pub fn with_capability_enabled(
        mut self,
        capability: RenderFeatureCapabilityRequirement,
    ) -> Self {
        self.enabled_capabilities.insert(capability);
        self
    }

    pub fn with_capability_disabled(
        mut self,
        capability: RenderFeatureCapabilityRequirement,
    ) -> Self {
        self.enabled_capabilities.remove(&capability);
        self
    }

    pub fn with_async_compute(mut self, enabled: bool) -> Self {
        self.allow_async_compute = enabled;
        self
    }

    pub fn with_hzb_occlusion_culling(mut self, enabled: bool) -> Self {
        self.enable_hzb_occlusion_culling = enabled;
        self
    }

    pub fn with_half_resolution_transparency(mut self, enabled: bool) -> Self {
        self.enable_half_resolution_transparency = enabled;
        self
    }

    pub fn with_half_resolution_transparency_depth_sigma(mut self, sigma: u16) -> Self {
        self.half_resolution_transparency_depth_sigma = sigma.max(1);
        self
    }

    pub fn with_graph_msaa_sample_count(mut self, sample_count: u32) -> Self {
        self.graph_msaa_sample_count = Some(sample_count.max(1));
        self
    }

    pub fn without_graph_msaa_sample_count(mut self) -> Self {
        self.graph_msaa_sample_count = None;
        self
    }

    pub fn with_shader_quality(mut self, quality: ShaderQualityTier) -> Self {
        self.shader_quality = quality;
        self
    }

    pub fn with_post_process_stack(mut self, stack: PostProcessStackDescriptor) -> Self {
        self.post_process_stack = Some(stack);
        self
    }

    pub fn with_ambient_occlusion_source(mut self, settings: AoSourceSettings) -> Self {
        self.ambient_occlusion_source = Some(settings.into());
        self
    }

    pub(crate) fn resolved_ambient_occlusion_source(
        &self,
        extract: &RenderFrameExtract,
    ) -> AoSourceSettings {
        self.ambient_occlusion_source
            .map(AoSourceSettings::from)
            .unwrap_or(extract.post_process.ambient_occlusion)
    }

    pub fn without_post_process_stack(mut self) -> Self {
        self.post_process_stack = None;
        self
    }

    pub fn with_post_process_effect_disabled(mut self, effect: PostProcessEffectKind) -> Self {
        self.post_process_stack = self
            .post_process_stack
            .take()
            .map(|stack| stack.with_effect_disabled(effect));
        self
    }

    pub fn with_environment_ibl_bake_request(mut self, request: IblBakeArtifactRequest) -> Self {
        self.environment_ibl_bake_request = Some(request);
        self
    }

    pub fn without_environment_ibl_bake_request(mut self) -> Self {
        self.environment_ibl_bake_request = None;
        self
    }

    pub fn environment_ibl_bake_request(&self) -> Option<&IblBakeArtifactRequest> {
        self.environment_ibl_bake_request.as_ref()
    }

    pub(crate) fn with_advanced_lighting_inputs(
        mut self,
        inputs: AdvancedLightingCompileInputs,
    ) -> Self {
        self.advanced_lighting_inputs = Some(inputs);
        self
    }

    pub(crate) fn resolved_advanced_lighting_inputs(
        &self,
        extract: &RenderFrameExtract,
    ) -> AdvancedLightingCompileInputs {
        self.advanced_lighting_inputs
            .clone()
            .unwrap_or_else(|| AdvancedLightingCompileInputs::from_extract(extract))
    }

    pub fn graph_msaa_sample_count(&self, camera_msaa_samples: u32) -> u32 {
        self.graph_msaa_sample_count
            .unwrap_or(camera_msaa_samples)
            .max(1)
    }

    pub(in crate::graphics::pipeline) fn permits_feature(
        &self,
        feature: BuiltinRenderFeature,
    ) -> bool {
        !self.disabled_features.contains(&feature)
            && (!feature.requires_explicit_opt_in() || self.enabled_features.contains(&feature))
    }

    pub(in crate::graphics::pipeline) fn permits_feature_asset(
        &self,
        feature: &RendererFeatureAsset,
    ) -> bool {
        if let Some(builtin) = feature.builtin_feature() {
            if !self.permits_feature(builtin) {
                return false;
            }

            return feature
                .descriptor()
                .capability_requirements
                .iter()
                .all(|requirement| {
                    !builtin_descriptor_capability_requires_explicit_opt_in(builtin, *requirement)
                        || self.enabled_capabilities.contains(requirement)
                });
        }

        let RendererFeatureSource::Plugin(feature_name) = &feature.feature else {
            unreachable!("builtin render features return before plugin admission")
        };
        let feature_name = feature
            .descriptor_override
            .as_ref()
            .map(|descriptor| descriptor.name.as_str())
            .unwrap_or(feature_name.as_str());
        if self.disabled_plugin_features.contains(feature_name) {
            return false;
        }

        let descriptor_requirements = feature
            .descriptor_override
            .as_ref()
            .map(|descriptor| descriptor.capability_requirements.as_slice())
            .unwrap_or_default();
        feature
            .capability_requirements
            .iter()
            .chain(descriptor_requirements)
            .all(|requirement| self.permits_capability_requirement(*requirement))
    }

    pub(in crate::graphics::pipeline) fn resolve_queue(&self, queue: QueueLane) -> QueueLane {
        match queue {
            QueueLane::AsyncCompute if !self.allow_async_compute => QueueLane::Graphics,
            _ => queue,
        }
    }

    fn permits_capability_requirement(
        &self,
        requirement: RenderFeatureCapabilityRequirement,
    ) -> bool {
        !capability_requires_explicit_opt_in(requirement)
            || self.enabled_capabilities.contains(&requirement)
    }
}

fn capability_requires_explicit_opt_in(requirement: RenderFeatureCapabilityRequirement) -> bool {
    matches!(
        requirement,
        RenderFeatureCapabilityRequirement::VirtualGeometry
            | RenderFeatureCapabilityRequirement::HybridGlobalIllumination
            | RenderFeatureCapabilityRequirement::AccelerationStructures
            | RenderFeatureCapabilityRequirement::InlineRayQuery
            | RenderFeatureCapabilityRequirement::RayTracingPipeline
            | RenderFeatureCapabilityRequirement::NeuralCompute
            | RenderFeatureCapabilityRequirement::SparseTexture
            | RenderFeatureCapabilityRequirement::SubgroupOps
            | RenderFeatureCapabilityRequirement::PipelineStatisticsQuery
    )
}

fn builtin_descriptor_capability_requires_explicit_opt_in(
    feature: BuiltinRenderFeature,
    requirement: RenderFeatureCapabilityRequirement,
) -> bool {
    descriptor_only_advanced_slot_requires_capability_opt_in(feature, requirement)
}

#[cfg(test)]
mod optimization_batch_df_runtime413_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::graphics::feature::{RenderFeatureCapabilityRequirement, RenderFeatureDescriptor};
    use crate::graphics::pipeline::declarations::{
        RenderPipelineCompileOptions, RendererFeatureAsset,
    };

    const SAMPLE_PAIRS: usize = 17;
    const CHECKS_PER_SAMPLE: usize = 512;
    const DESCRIPTOR_SECTION_COUNT: usize = 128;

    #[test]
    fn optimization_batch_df_runtime413_borrowed_plugin_admission_matches_legacy_semantics() {
        let feature = plugin_feature();
        let enabled = RenderPipelineCompileOptions::default()
            .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry);
        let disabled = enabled
            .clone()
            .with_plugin_feature_disabled("plugin.deep-feature");

        assert_eq!(
            enabled.permits_feature_asset(&feature),
            legacy_permits_feature_asset(&enabled, &feature)
        );
        assert_eq!(
            disabled.permits_feature_asset(&feature),
            legacy_permits_feature_asset(&disabled, &feature)
        );
        assert!(enabled.permits_feature_asset(&feature));
        assert!(!disabled.permits_feature_asset(&feature));
        assert!(!RenderPipelineCompileOptions::default().permits_feature_asset(&feature));

        let renamed = RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
            "plugin.source-name",
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ))
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "plugin.override-name",
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ));
        let override_disabled = RenderPipelineCompileOptions::default()
            .with_plugin_feature_disabled("plugin.override-name");
        assert_eq!(
            override_disabled.permits_feature_asset(&renamed),
            legacy_permits_feature_asset(&override_disabled, &renamed)
        );
        assert!(!override_disabled.permits_feature_asset(&renamed));
    }

    #[test]
    fn optimization_batch_df_runtime413_plugin_admission_borrows_name_and_descriptor() {
        let source = include_str!("methods.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let admission = production
            .split("pub(in crate::graphics::pipeline) fn permits_feature_asset")
            .nth(1)
            .unwrap()
            .split("pub(in crate::graphics::pipeline) fn resolve_queue")
            .next()
            .unwrap();
        let plugin_admission = admission
            .split("let RendererFeatureSource::Plugin(feature_name)")
            .nth(1)
            .unwrap();

        assert!(admission.contains("RendererFeatureSource::Plugin(feature_name)"));
        assert!(plugin_admission.contains("descriptor_override"));
        assert!(plugin_admission.contains(".as_ref()"));
        assert!(!plugin_admission.contains("feature.feature_name()"));
        assert!(!plugin_admission.contains("feature.descriptor()"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_df_runtime413_borrowed_plugin_admission_p95() {
        let feature = plugin_feature();
        let options = RenderPipelineCompileOptions::default()
            .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry);
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&options, &feature, false));
                optimized.push(measure(&options, &feature, true));
            } else {
                optimized.push(measure(&options, &feature, true));
                legacy.push(measure(&options, &feature, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME413_PLUGIN_FEATURE_BORROWED_ADMISSION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} descriptor_sections={DESCRIPTOR_SECTION_COUNT} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "borrowed plugin admission must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn plugin_feature() -> RendererFeatureAsset {
        let required_extract_sections = (0..DESCRIPTOR_SECTION_COUNT)
            .map(|index| format!("plugin.deep-feature.extract.section.{index:03}"))
            .collect();
        let descriptor = RenderFeatureDescriptor::new(
            "plugin.deep-feature",
            required_extract_sections,
            Vec::new(),
            Vec::new(),
        )
        .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry);
        RendererFeatureAsset::plugin(descriptor)
            .with_capability_requirement(RenderFeatureCapabilityRequirement::BufferReadback)
    }

    fn legacy_permits_feature_asset(
        options: &RenderPipelineCompileOptions,
        feature: &RendererFeatureAsset,
    ) -> bool {
        if options
            .disabled_plugin_features
            .contains(&feature.feature_name())
        {
            return false;
        }
        let descriptor = feature.descriptor();
        black_box(&descriptor.required_extract_sections);
        feature
            .capability_requirements
            .iter()
            .chain(descriptor.capability_requirements.iter())
            .all(|requirement| options.permits_capability_requirement(*requirement))
    }

    fn measure(
        options: &RenderPipelineCompileOptions,
        feature: &RendererFeatureAsset,
        optimized: bool,
    ) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let permitted = if optimized {
                options.permits_feature_asset(black_box(feature))
            } else {
                legacy_permits_feature_asset(options, black_box(feature))
            };
            black_box(permitted);
        }
        started.elapsed().as_nanos()
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        ordered[(ordered.len() - 1) * percentile / 100]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
