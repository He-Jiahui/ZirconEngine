//! 扩展登记从批次验证到宿主投影及撤销按契约拆分，避免只测试单层 Registry。
#[path = "../../extensions_registration/operation_and_view_register.rs"]
mod operation_and_view_registration;
#[path = "../../runtime/extensions_registration/overlay_lifecycle.rs"]
mod overlay_lifecycle;
#[path = "../../runtime/extensions_registration/plugin_contributions.rs"]
mod plugin_contributions;
#[path = "../../extensions_registration/ticketed_command_revoke.rs"]
mod ticketed_command_revoke;
