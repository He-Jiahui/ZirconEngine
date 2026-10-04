//! 扩展登记从批次验证到宿主投影及撤销按契约拆分，避免只测试单层 Registry。
mod operation_and_view_registration;
mod overlay_lifecycle;
mod plugin_contributions;
mod ticketed_command_revoke;
