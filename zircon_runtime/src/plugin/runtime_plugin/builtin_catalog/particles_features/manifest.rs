use crate::plugin::{PluginFeatureBundleManifest, PluginFeatureDependency};

use super::rows::ParticlesFeatureRow;

fn join_string_parts(parts: &[&str]) -> String {
    let capacity = parts.iter().map(|part| part.len()).sum();
    let mut joined = String::with_capacity(capacity);
    for part in parts {
        joined.push_str(part);
    }
    joined
}

// 生成 particles.<suffix> 清单，先登记基础包主依赖，再追加该行声明的物理、动画或渲染图依赖。
pub(super) fn particles_feature(row: &ParticlesFeatureRow) -> PluginFeatureBundleManifest {
    let feature_id = join_string_parts(&["particles.", row.id_suffix]);
    let mut manifest = PluginFeatureBundleManifest::new(feature_id, row.display_name, "particles")
        .with_dependency(PluginFeatureDependency::primary(
            "particles",
            "runtime.plugin.particles",
        ))
        .with_capability(row.capability);
    for dependency in row.extra_dependencies {
        manifest = manifest.with_dependency(PluginFeatureDependency::required(
            dependency.provider_plugin_id,
            dependency.capability,
        ));
    }
    manifest
}

#[cfg(test)]
#[path = "tests/manifest.rs"]
mod tests;
