//! 模块与插件回调上下文；普通回访使用弱句柄，停止清理另用受限的借用能力。
#[path = "module_cleanup_context.rs"]
mod module_cleanup_context;
#[path = "module_context.rs"]
mod module_context;
#[path = "plugin_context.rs"]
mod plugin_context;

pub use module_cleanup_context::ModuleCleanupContext;
pub use module_context::ModuleContext;
pub use plugin_context::PluginContext;
