use crate::render_graph::QueueLane;

use crate::graphics::feature::BuiltinRenderFeature;
use crate::graphics::pipeline::declarations::RenderPipelineCompileOptions;

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

    pub fn with_async_compute(mut self, enabled: bool) -> Self {
        self.allow_async_compute = enabled;
        self
    }

    pub(in crate::graphics::pipeline) fn permits_feature(&self, feature: BuiltinRenderFeature) -> bool {
        !self.disabled_features.contains(&feature)
            && (!feature.requires_explicit_opt_in() || self.enabled_features.contains(&feature))
    }

    pub(in crate::graphics::pipeline) fn resolve_queue(&self, queue: QueueLane) -> QueueLane {
        match queue {
            QueueLane::AsyncCompute if !self.allow_async_compute => QueueLane::Graphics,
            _ => queue,
        }
    }
}
