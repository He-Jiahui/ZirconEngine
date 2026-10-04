//! 应用组合层从 plugin::native::host 获取稳定宿主句柄；具体库生命周期仍由 native_plugin_loader 管理。
//! 强/弱句柄的升降级和 authority 约束见其原定义，此处不复制实现或延长库寿命。
pub use super::super::native_plugin_loader::{NativePluginHostHandle, NativePluginHostWeakHandle};
