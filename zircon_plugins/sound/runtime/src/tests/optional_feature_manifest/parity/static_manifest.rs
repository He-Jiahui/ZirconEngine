// 静态 TOML 与宿主 package_manifest 的功能签名应一致，避免导出选择与内建运行时声明分叉。
// TODO: [CR-SOUND-TESTS-0001] 本签名没有包含独立提供者包 ID 与 distribution 元数据；需核查另有覆盖后决定是否扩展对照。
use super::super::support::{optional_features_from_plugin_toml, STATIC_SOUND_PLUGIN_MANIFEST};
use super::support::{
    sorted_runtime_optional_feature_signatures, sorted_static_optional_feature_signatures,
};

#[test]
fn static_plugin_manifest_keeps_optional_feature_manifests_in_sync() {
    let static_features = sorted_static_optional_feature_signatures(
        optional_features_from_plugin_toml(STATIC_SOUND_PLUGIN_MANIFEST),
    );
    let runtime_features =
        sorted_runtime_optional_feature_signatures(crate::package_manifest().optional_features);

    assert_eq!(static_features, runtime_features);
}
