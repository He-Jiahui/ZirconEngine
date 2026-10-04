//! 贴花编辑器标识复用运行时定义，供描述符和特性清单投影保持一致。
pub const FEATURE_ID: &str = zircon_plugin_rendering_decals_runtime::FEATURE_ID;
pub const CAPABILITY: &str = zircon_plugin_rendering_decals_runtime::EDITOR_CAPABILITY;

pub const EDITOR_CAPABILITIES: &[&str] = &[CAPABILITY];
