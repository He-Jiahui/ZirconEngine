mod capability;
mod extension_ids;
mod plugin;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub use capability::{EDITOR_CAPABILITIES, HYBRID_GI_AUTHORING_CAPABILITY, PLUGIN_ID};
pub use extension_ids::{HYBRID_GI_AUTHORING_VIEW_ID, HYBRID_GI_DRAWER_ID, HYBRID_GI_TEMPLATE_ID};
// TODO: [CR-HYBRID-GI-EP-0001] 首方编辑器目录尚未映射此注册入口；核对 EditorHost 选择路径并补测试。
pub use plugin::{
    editor_capabilities, editor_host_contract_marker, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration, HybridGiEditorPlugin,
};
