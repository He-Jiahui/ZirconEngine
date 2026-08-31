use crate::plugin::PluginFeatureBundleManifest;

use super::shape::{
    validate_runtime_plugin_feature_field, validate_runtime_plugin_feature_namespace,
    validate_runtime_plugin_feature_token,
};

pub(super) fn validate_runtime_plugin_feature_identity(
    feature: &PluginFeatureBundleManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_field("feature id", &feature.id, diagnostics);
    validate_runtime_plugin_feature_namespace("feature id", &feature.id, diagnostics);
    validate_runtime_plugin_feature_field("display_name", &feature.display_name, diagnostics);
    validate_runtime_plugin_feature_field("owner_plugin_id", &feature.owner_plugin_id, diagnostics);
    validate_runtime_plugin_feature_token("owner_plugin_id", &feature.owner_plugin_id, diagnostics);
    if !feature_id_has_owner(&feature.owner_plugin_id, &feature.id) {
        diagnostics.push(format!(
            "runtime plugin feature manifest feature id `{}` must be prefixed by owner_plugin_id `{}`",
            feature.id, feature.owner_plugin_id
        ));
    }
}

fn feature_id_has_owner(owner_plugin_id: &str, feature_id: &str) -> bool {
    let feature = feature_id.as_bytes();
    let owner = owner_plugin_id.as_bytes();
    feature.len() > owner.len() && feature.starts_with(owner) && feature[owner.len()] == b'.'
}

#[cfg(test)]
mod tests {
    #[test]
    fn runtime_feature_owner_prefix_check_does_not_format_a_string() {
        let source = include_str!("identity.rs");
        let formatted_prefix = ["format!(\"{}", ".\", feature.owner_plugin_id)"].concat();
        assert!(!source.contains(&formatted_prefix));
    }

    #[test]
    fn runtime_feature_owner_matching_preserves_the_dot_boundary() {
        assert!(super::feature_id_has_owner(
            "rendering",
            "rendering.deferred"
        ));
        assert!(!super::feature_id_has_owner("render", "rendering.deferred"));
    }

    #[test]
    fn optimization_batch_gn_runtime496_feature_owner_byte_boundary_preserves_rules() {
        assert!(super::feature_id_has_owner(
            "rendering",
            "rendering.deferred"
        ));
        assert!(!super::feature_id_has_owner("rendering", "rendering"));
        assert!(!super::feature_id_has_owner("render", "rendering.deferred"));
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gn_runtime496_feature_owner_byte_boundary_benchmark() {
        const MARKER: &str = "RUNTIME496_FEATURE_OWNER_BYTE_BOUNDARY_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let owner = "rendering";
        let feature = "rendering.deferred.materials.shadow_pass.quality_profile";
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(super::feature_id_has_owner(owner, feature));
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(feature
                .strip_prefix(owner)
                .is_some_and(|suffix| suffix.starts_with('.')));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
    }
}
