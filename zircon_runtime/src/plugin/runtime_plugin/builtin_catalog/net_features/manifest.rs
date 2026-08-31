use crate::plugin::{PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest};

use super::rows::NetFeatureRow;

fn join_string_parts(parts: &[&str]) -> String {
    let capacity = parts.iter().map(|part| part.len()).sum();
    let mut joined = String::with_capacity(capacity);
    for part in parts {
        joined.push_str(part);
    }
    joined
}

pub(super) fn net_feature(row: &NetFeatureRow) -> PluginFeatureBundleManifest {
    let feature_id = join_string_parts(&["net.", row.id_suffix]);
    let runtime_module_id = join_string_parts(&[&feature_id, ".runtime"]);
    let mut manifest = PluginFeatureBundleManifest::new(feature_id, row.display_name, "net")
        .with_dependency(PluginFeatureDependency::primary(
            "net",
            "runtime.plugin.net",
        ))
        .with_capability(row.capability)
        .with_runtime_module(
            PluginModuleManifest::runtime(runtime_module_id, row.runtime_crate)
                .with_target_modes(row.target_modes.iter().copied())
                .with_capabilities([row.capability.to_string()]),
        );
    for dependency in row.extra_dependencies {
        manifest = manifest.with_dependency(PluginFeatureDependency::required(
            dependency.provider_plugin_id,
            dependency.capability,
        ));
    }
    manifest
}

#[cfg(test)]
mod tests {
    use super::join_string_parts;

    #[test]
    fn exact_net_identifier_join_preserves_feature_and_runtime_ids() {
        let feature_id = join_string_parts(&["net.", "reliable_udp"]);
        assert_eq!(feature_id, "net.reliable_udp");
        assert_eq!(
            join_string_parts(&[&feature_id, ".runtime"]),
            "net.reliable_udp.runtime"
        );
    }
}
