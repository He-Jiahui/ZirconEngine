// 从插件注册报告观察实际贡献及包清单，确认公开入口发布的字段与功能相符。
use super::super::super::*;

#[test]
fn sound_plugin_registration_contributes_runtime_components() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    for component in [
        AUDIO_SOURCE_COMPONENT_TYPE,
        AUDIO_LISTENER_COMPONENT_TYPE,
        AUDIO_VOLUME_COMPONENT_TYPE,
    ] {
        assert!(report
            .extensions
            .components()
            .iter()
            .any(|descriptor| descriptor.type_id == component));
        assert!(report
            .package_manifest
            .components
            .iter()
            .any(|descriptor| descriptor.type_id == component));
    }
}
