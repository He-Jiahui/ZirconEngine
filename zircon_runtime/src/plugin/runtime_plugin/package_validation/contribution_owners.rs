//! 包校验要求组件和 UI 组件显式属于承载包，事件目录则通过命名空间边界表达归属。
//! 这里追加诊断，不修改身份或清单；字段形状与实际注册合法性由其他校验负责。
mod components;
mod event_catalogs;
mod ui_components;

use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_event_catalog_owners(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    event_catalogs::validate_event_catalog_owners(package_manifest, diagnostics);
}

pub(in crate::plugin::runtime_plugin) fn validate_component_owners(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    components::validate_component_owners(package_manifest, diagnostics);
}

pub(in crate::plugin::runtime_plugin) fn validate_ui_component_owners(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    ui_components::validate_ui_component_owners(package_manifest, diagnostics);
}
