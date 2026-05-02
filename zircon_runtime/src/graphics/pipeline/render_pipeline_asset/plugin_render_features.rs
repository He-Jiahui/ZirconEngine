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
            let feature = RendererFeatureAsset::plugin(descriptor);
            if let Some(index) = plugin_feature_insert_index(&self.renderer.features, &feature) {
                self.renderer.features.insert(index, feature);
            } else {
                self.renderer.features.push(feature);
            }
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

fn plugin_feature_insert_index(
    features: &[RendererFeatureAsset],
    feature: &RendererFeatureAsset,
) -> Option<usize> {
    match feature.feature_name().as_str() {
        "screen_space_ambient_occlusion" | "ssao" => {
            index_before_feature_name(features, "clustered_lighting")
                .or_else(|| index_after_feature_name(features, "shadows"))
        }
        "reflection_probes" => index_after_feature_name(features, "bloom"),
        "baked_lighting" => {
            index_after_last_feature_name(features, &["reflection_probes", "bloom"])
        }
        "decals" => index_after_last_feature_name(
            features,
            &["baked_lighting", "reflection_probes", "bloom"],
        ),
        "post_process" => index_after_last_feature_name(
            features,
            &["decals", "baked_lighting", "reflection_probes", "bloom"],
        ),
        "shader_graph" => index_after_last_feature_name(features, &["post_process"]),
        _ => None,
    }
}

fn index_before_feature_name(features: &[RendererFeatureAsset], name: &str) -> Option<usize> {
    features
        .iter()
        .position(|feature| feature.feature_name() == name)
}

fn index_after_feature_name(features: &[RendererFeatureAsset], name: &str) -> Option<usize> {
    features
        .iter()
        .position(|feature| feature.feature_name() == name)
        .map(|index| index + 1)
}

fn index_after_last_feature_name(
    features: &[RendererFeatureAsset],
    names: &[&str],
) -> Option<usize> {
    names
        .iter()
        .filter_map(|name| index_after_feature_name(features, name))
        .max()
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
