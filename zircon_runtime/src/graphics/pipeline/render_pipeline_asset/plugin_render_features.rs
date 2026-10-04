use crate::graphics::feature::RenderFeatureDescriptor;
use crate::graphics::pipeline::declarations::{RenderPipelineAsset, RendererFeatureAsset};
use crate::graphics::scene::anti_alias::smaa::{SMAA_EXECUTOR_ID, SMAA_PASS_NAME};

impl RenderPipelineAsset {
    /// 以值接收插件 descriptor，按内建 feature 锚点插入并返回修订后的资产。
    pub fn with_plugin_render_features(
        mut self,
        descriptors: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) -> Self {
        self.apply_plugin_render_features(descriptors);
        self
    }

    /// 追加插件 feature；同名或能力冲突的内建 feature 会先被过滤，成功变更推进 revision。
    pub fn apply_plugin_render_features(
        &mut self,
        descriptors: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) {
        let mut changed = false;
        for descriptor in descriptors {
            self.remove_features_replaced_by_plugin_descriptor(&descriptor);
            let feature = RendererFeatureAsset::plugin(descriptor);
            if let Some(index) = plugin_feature_insert_index(&self.renderer.features, &feature) {
                self.renderer.features.insert(index, feature);
            } else {
                self.renderer.features.push(feature);
            }
            changed = true;
        }
        if changed {
            self.bump_revision();
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
    // 锚点只表达图形依赖的最小顺序；找不到锚点时保留调用者给出的尾部追加语义。
    match feature.feature_name().as_str() {
        "screen_space_ambient_occlusion" | "ssao" => {
            index_before_feature_name(features, "clustered_lighting")
                .or_else(|| index_after_feature_name(features, "shadows"))
        }
        "contact_shadow" => index_before_feature_name(features, "clustered_lighting")
            .or_else(|| index_after_feature_name(features, "hzb")),
        "volumetric_fog" => index_after_feature_name(features, "clustered_lighting")
            .or_else(|| index_after_feature_name(features, "shadows")),
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
    if names.is_empty() {
        return None;
    }
    let mut matched_names = vec![false; names.len()];
    let mut remaining_names = names.len();
    let mut last_index = None;
    for (feature_index, feature) in features.iter().enumerate() {
        let feature_name = feature.feature_name();
        let Some(name_index) = names.iter().position(|name| *name == feature_name.as_str()) else {
            continue;
        };
        if matched_names[name_index] {
            continue;
        }
        matched_names[name_index] = true;
        remaining_names = remaining_names.saturating_sub(1);
        last_index = Some(feature_index + 1);
        if remaining_names == 0 {
            break;
        }
    }
    last_index
}

fn feature_is_replaced_by_plugin_descriptor(
    feature: &RendererFeatureAsset,
    descriptor: &RenderFeatureDescriptor,
) -> bool {
    // SMAA 插件仍需保留内建 AntiAlias 壳，以便其 terminal slot 被编译器识别。
    if feature.is_builtin(crate::graphics::BuiltinRenderFeature::AntiAlias)
        && descriptor_declares_smaa_terminal_slot(descriptor)
    {
        return false;
    }

    feature.feature_name() == descriptor.name
        || (feature.builtin_feature().is_some()
            && descriptor
                .capability_requirements
                .iter()
                .any(|requirement| feature.requires_capability(*requirement)))
}

fn descriptor_declares_smaa_terminal_slot(descriptor: &RenderFeatureDescriptor) -> bool {
    descriptor.stage_passes.iter().any(|pass| {
        pass.pass_name == SMAA_PASS_NAME || pass.executor_id.as_str() == SMAA_EXECUTOR_ID
    })
}

#[cfg(test)]
#[path = "tests/plugin_render_features_optimization_batch_20260830cm_runtime389_tests.rs"]
mod optimization_batch_20260830cm_runtime389_tests;
