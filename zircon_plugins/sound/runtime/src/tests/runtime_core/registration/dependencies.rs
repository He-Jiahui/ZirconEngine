// 从插件注册报告观察实际贡献及包清单，确认公开入口发布的字段与功能相符。
use super::super::super::*;

#[test]
fn sound_plugin_registration_contributes_optional_timeline_dependency() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report
        .package_manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.id == "timeline_sequence" && !dependency.required));
}
