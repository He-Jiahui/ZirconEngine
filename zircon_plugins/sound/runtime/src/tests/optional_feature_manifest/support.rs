//! 测试专用静态解析与运行时投影入口；仅针对 sound/plugin.toml 当前写法及可选功能对照使用。
mod parser;
mod runtime;
mod types;
mod values;

pub(in crate::tests::optional_feature_manifest) use types::StaticOptionalFeatureManifest;

pub(super) const STATIC_SOUND_PLUGIN_MANIFEST: &str = include_str!("../../../../plugin.toml");

pub(super) fn optional_features_from_plugin_toml(
    manifest: &str,
) -> Vec<StaticOptionalFeatureManifest> {
    parser::optional_features_from_plugin_toml(manifest)
}

pub(super) fn optional_feature_signature(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> StaticOptionalFeatureManifest {
    runtime::optional_feature_signature(feature)
}
