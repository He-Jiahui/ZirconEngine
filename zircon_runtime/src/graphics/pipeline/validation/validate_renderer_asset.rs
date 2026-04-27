use std::collections::BTreeSet;

use crate::graphics::pipeline::declarations::RendererAsset;

use super::stage_name::stage_name;

pub(in crate::graphics::pipeline) fn validate_renderer_asset(
    renderer: &RendererAsset,
) -> Result<(), String> {
    if renderer.stages.is_empty() {
        return Err(format!(
            "renderer `{}` must declare at least one render stage",
            renderer.name
        ));
    }

    let mut seen_stages = BTreeSet::new();
    for stage in &renderer.stages {
        if !seen_stages.insert(*stage) {
            return Err(format!(
                "renderer `{}` contains duplicate stage `{}`",
                renderer.name,
                stage_name(*stage)
            ));
        }
    }

    let mut seen_features = BTreeSet::new();
    for feature in &renderer.features {
        let feature_name = feature.feature_name();
        if !seen_features.insert(feature_name.clone()) {
            return Err(format!(
                "renderer `{}` contains duplicate feature `{}`",
                renderer.name, feature_name
            ));
        }
    }

    Ok(())
}
