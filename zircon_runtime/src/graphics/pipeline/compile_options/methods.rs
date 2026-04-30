use crate::render_graph::QueueLane;

use crate::graphics::feature::{BuiltinRenderFeature, RenderFeatureCapabilityRequirement};
use crate::graphics::pipeline::declarations::{RenderPipelineCompileOptions, RendererFeatureAsset};

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
            return self.permits_feature(builtin);
        }

        if self
            .disabled_plugin_features
            .contains(&feature.feature_name())
        {
            return false;
        }

        let descriptor = feature.descriptor();
        feature
            .capability_requirements
            .iter()
            .chain(descriptor.capability_requirements.iter())
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
    )
}
