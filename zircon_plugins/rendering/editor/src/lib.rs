//! 渲染主包的编辑器公共面；能力查询与包清单用于宿主发现，可选特性由各自模块提供。
mod capability;
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID};
pub use plugin::{
    editor_capabilities, editor_plugin, editor_plugin_descriptor, package_manifest,
    RenderingEditorPlugin,
};
