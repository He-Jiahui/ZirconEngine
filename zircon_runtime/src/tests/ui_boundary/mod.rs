//! 界面模块结构与共享界面宿主的回归入口；测试夹具归测试树拥有，生产帧载荷保持独立。
//!
//! # 挂载与边界
//! 仅在启用原生宿主和界面功能的测试构建中由父测试树挂载；本入口不创建宿主或帧资源。
//! 各子模块拥有其读取结果与局部扫描集合，测试函数由测试框架分别调用。
#[path = "assets.rs"]
mod assets;
#[path = "binding_event_namespaces.rs"]
mod binding_event_namespaces;
#[path = "module_absorption.rs"]
mod module_absorption;
#[path = "runtime_host.rs"]
mod runtime_host;
#[path = "surface_dispatch_namespaces.rs"]
mod surface_dispatch_namespaces;
#[path = "template_namespaces.rs"]
mod template_namespaces;
