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
        if !seen_features.insert(feature.feature) {
            return Err(format!(
                "renderer `{}` contains duplicate feature `{}`",
                renderer.name,
                feature.feature.descriptor().name
            ));
        }
    }

    Ok(())
}
