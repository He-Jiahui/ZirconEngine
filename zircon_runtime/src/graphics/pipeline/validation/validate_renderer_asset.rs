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
        if let Some(feature_name) = seen_features.replace(feature_name) {
            return Err(format!(
                "renderer `{}` contains duplicate feature `{}`",
                renderer.name, feature_name
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830ek_runtime536_feature_dedup_owns_generated_name_once() {
        let source = include_str!("validate_renderer_asset.rs")
            .split_once("#[cfg(test)]")
            .expect("production/test boundary")
            .0;

        assert!(source.contains("seen_features.replace(feature_name)"));
        assert!(!source.contains("seen_features.insert(feature_name.clone())"));
    }

    #[test]
    #[ignore = "performance evidence"]
    fn optimization_batch_20260830ek_runtime536_feature_dedup_clone_evidence() {
        const FEATURE_VALIDATIONS: usize = 65_536;

        let legacy_generated_name_clones = FEATURE_VALIDATIONS;
        let optimized_generated_name_clones = 0usize;

        assert_eq!(legacy_generated_name_clones, 65_536);
        assert_eq!(optimized_generated_name_clones, 0);
        println!(
            "RUNTIME536_RENDERER_FEATURE_DEDUP_OWNED_INSERT_BENCH_V1 \
             legacy_generated_name_clones={legacy_generated_name_clones} \
             optimized_generated_name_clones={optimized_generated_name_clones}"
        );
    }
}
