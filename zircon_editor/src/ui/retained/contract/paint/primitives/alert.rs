// Alert 材质绘制入口由分发器选择；根节点统一拥有表面、正文和操作标记，避免通用模板重复绘制。
#[path = "alert/action.rs"]
mod action;
#[path = "alert/commands.rs"]
mod commands;
#[path = "alert/geometry.rs"]
mod geometry;
#[path = "alert/icon.rs"]
mod icon;
#[path = "alert/identity.rs"]
mod identity;
#[path = "alert/message.rs"]
mod message;
#[path = "alert/style.rs"]
mod style;
#[path = "alert/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_alert_primitive_commands;
