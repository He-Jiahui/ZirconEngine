// Alert 材质绘制入口由分发器选择；根节点统一拥有表面、正文和操作标记，避免通用模板重复绘制。
mod action;
mod commands;
mod geometry;
mod icon;
mod identity;
mod message;
mod style;
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_alert_primitive_commands;
