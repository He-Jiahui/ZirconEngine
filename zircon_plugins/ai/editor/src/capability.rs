pub use zircon_plugin_ai_runtime::PLUGIN_ID;
pub const AI_AUTHORING_CAPABILITY: &str = "editor.extension.ai_authoring";
pub const AI_DEBUG_CAPABILITY: &str = "editor.extension.ai_debug";
pub const EDITOR_CAPABILITIES: &[&str] = &[AI_AUTHORING_CAPABILITY, AI_DEBUG_CAPABILITY];
