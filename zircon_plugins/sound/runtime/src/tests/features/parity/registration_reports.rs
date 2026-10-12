// 注册报告以 owner 回退规则标识提供者；这里核对其清单签名及成功状态，独立包身份需另查发布边界。
use super::support::{
    sorted_registration_report_optional_feature_signatures,
    sorted_runtime_optional_feature_signatures,
};

#[test]
fn linked_feature_registration_reports_match_sound_package_manifest() {
    let package_features =
        sorted_runtime_optional_feature_signatures(crate::package_manifest().optional_features);
    let linked_provider_features = sorted_registration_report_optional_feature_signatures([
        zircon_plugin_sound_timeline_animation_runtime::plugin_feature_registration(),
        zircon_plugin_sound_ray_traced_convolution_runtime::plugin_feature_registration(),
    ]);

    assert_eq!(package_features, linked_provider_features);
}
