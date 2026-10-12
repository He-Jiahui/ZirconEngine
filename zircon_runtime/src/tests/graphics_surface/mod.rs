//! 图形公共面与宿主吸收的回归入口；结构检查和实际渲染提交属于不同证据，均受本模块的图形功能开关控制。
#[path = "host_wiring.rs"]
mod host_wiring;
#[path = "internal_visibility.rs"]
mod internal_visibility;
#[path = "module_absorption.rs"]
mod module_absorption;
#[path = "runtime_ui_integration.rs"]
mod runtime_ui_integration;
