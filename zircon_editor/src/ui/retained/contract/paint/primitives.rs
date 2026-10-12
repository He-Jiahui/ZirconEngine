//! 汇集 Material 组件专用绘制命令；分发器决定完整接管节点或仅替换 TextField 表面。
#[path = "primitives/alert.rs"]
mod alert;
#[path = "primitives/avatar.rs"]
mod avatar;
#[path = "primitives/badge.rs"]
mod badge;
#[path = "primitives/chip.rs"]
mod chip;
#[path = "primitives/dispatch.rs"]
mod dispatch;
#[path = "primitives/divider.rs"]
mod divider;
#[path = "primitives/paper.rs"]
mod paper;
#[path = "primitives/shared.rs"]
mod shared;
#[path = "primitives/skeleton.rs"]
mod skeleton;
#[path = "primitives/text_field.rs"]
mod text_field;
#[path = "primitives/timeline.rs"]
mod timeline;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use super::template_style_color::resolved_style_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use dispatch::{
    push_material_primitive_commands, push_material_text_field_surface_commands,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use shared::{
    bounded_extent, component_variant_contains, first_non_empty,
};
