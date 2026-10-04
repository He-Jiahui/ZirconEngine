//! 编辑器作者能力与运行时插件身份共享包 ID；编辑器目录以该能力声明判断扩展可见性。
pub const PLUGIN_ID: &str = zircon_plugin_animation_runtime::PLUGIN_ID;
pub const ANIMATION_AUTHORING_CAPABILITY: &str = "editor.extension.animation_authoring";

pub const EDITOR_CAPABILITIES: &[&str] = &[ANIMATION_AUTHORING_CAPABILITY];
