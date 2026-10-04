//! 渲染主包的编辑器能力投影；包身份复用运行时定义，编辑器能力不授予 GPU 执行能力。
pub const PLUGIN_ID: &str = zircon_plugin_rendering_runtime::PLUGIN_ID;
pub const CAPABILITY: &str = "editor.extension.rendering_authoring";

pub const EDITOR_CAPABILITIES: &[&str] = &[CAPABILITY];
