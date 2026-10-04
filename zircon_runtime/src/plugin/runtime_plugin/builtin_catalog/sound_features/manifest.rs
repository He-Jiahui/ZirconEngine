use crate::core::framework::project::ExportPackagingStrategy;
use crate::plugin::{
    PluginDistributionManifest, PluginFeatureBundleManifest, PluginFeatureDependency,
    PluginModuleManifest,
};

use super::rows::SoundFeatureRow;

const SOUND_FEATURE_DIST_DESCRIPTOR_SYMBOL: &str = "zircon_native_plugin_descriptor_v3";
const SOUND_FEATURE_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const SOUND_FEATURE_DIST_ABI_VERSION: u32 = 3;

fn join_string_parts(parts: &[&str]) -> String {
    let capacity = parts.iter().map(|part| part.len()).sum();
    let mut joined = String::with_capacity(capacity);
    for part in parts {
        joined.push_str(part);
    }
    joined
}

// 同一行同时描述运行时、编辑器和原生分发模块；原生分发另附 ABI 元数据。
pub(super) fn sound_feature(row: &SoundFeatureRow) -> PluginFeatureBundleManifest {
    let feature_id = join_string_parts(&["sound.", row.id_suffix]);
    let mut manifest =
        PluginFeatureBundleManifest::new(feature_id.clone(), row.display_name, "sound")
            .with_provider_package_id(row.provider_package_id)
            .with_distribution(sound_feature_distribution_manifest(row))
            .with_default_packaging([
                ExportPackagingStrategy::SourceTemplate,
                ExportPackagingStrategy::LibraryEmbed,
                ExportPackagingStrategy::NativeDynamic,
            ])
            .with_dependency(PluginFeatureDependency::primary(
                "sound",
                "runtime.plugin.sound",
            ))
            .with_capability(row.runtime_capability)
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    join_string_parts(&[&feature_id, ".runtime"]),
                    row.runtime_crate,
                )
                .with_target_modes(row.runtime_target_modes.iter().copied())
                .with_capabilities([row.runtime_capability.to_string()]),
            )
            .with_editor_module(
                PluginModuleManifest::editor(
                    join_string_parts(&[&feature_id, ".editor"]),
                    row.editor_crate,
                )
                .with_capabilities([row.editor_capability.to_string()]),
            )
            .with_native_module(
                PluginModuleManifest::native(
                    join_string_parts(&[&feature_id, ".dist"]),
                    row.dist_crate,
                )
                .with_target_modes(row.runtime_target_modes.iter().copied())
                .with_capabilities([row.runtime_capability.to_string()]),
            );
    for dependency in row.extra_dependencies {
        manifest = manifest.with_dependency(PluginFeatureDependency::required(
            dependency.provider_plugin_id,
            dependency.capability,
        ));
    }
    manifest
}

fn sound_feature_distribution_manifest(row: &SoundFeatureRow) -> PluginDistributionManifest {
    PluginDistributionManifest {
        forms: vec!["dist".to_string()],
        default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
        abi_version: Some(SOUND_FEATURE_DIST_ABI_VERSION),
        engine_compat: SOUND_FEATURE_DIST_ENGINE_COMPAT.to_string(),
        dist_crate: row.dist_crate.to_string(),
        descriptor_symbol: SOUND_FEATURE_DIST_DESCRIPTOR_SYMBOL.to_string(),
        runtime_entry: row.dist_runtime_entry.to_string(),
        ..PluginDistributionManifest::default()
    }
}

#[cfg(test)]
#[path = "tests/manifest.rs"]
mod tests;
