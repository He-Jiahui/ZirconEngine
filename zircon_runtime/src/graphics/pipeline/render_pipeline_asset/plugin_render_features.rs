use crate::graphics::feature::RenderFeatureDescriptor;
use crate::graphics::pipeline::declarations::{RenderPipelineAsset, RendererFeatureAsset};

impl RenderPipelineAsset {
    pub fn with_plugin_render_features(
        mut self,
        descriptors: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) -> Self {
        self.apply_plugin_render_features(descriptors);
        self
    }

    pub fn apply_plugin_render_features(
        &mut self,
        descriptors: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) {
        for descriptor in descriptors {
            self.remove_features_replaced_by_plugin_descriptor(&descriptor);
            self.renderer
                .features
                .push(RendererFeatureAsset::plugin(descriptor));
        }
    }

    fn remove_features_replaced_by_plugin_descriptor(
        &mut self,
        descriptor: &RenderFeatureDescriptor,
    ) {
        self.renderer
            .features
            .retain(|feature| !feature_is_replaced_by_plugin_descriptor(feature, descriptor));
    }
}

fn feature_is_replaced_by_plugin_descriptor(
    feature: &RendererFeatureAsset,
    descriptor: &RenderFeatureDescriptor,
) -> bool {
    feature.feature_name() == descriptor.name
        || (feature.builtin_feature().is_some()
            && descriptor
                .capability_requirements
                .iter()
                .any(|requirement| feature.requires_capability(*requirement)))
}
