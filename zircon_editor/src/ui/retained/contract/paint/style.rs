//! 供通用表面与文字 fallback 消费的样式投影；专用控件可覆写自己的语义和几何。
//! 状态解析复用共享选择器，颜色、边界尺寸与交互生命周期分开维护。

#[path = "style/colors.rs"]
mod colors;
#[path = "style/dimensions.rs"]
mod dimensions;
#[path = "style/elevation.rs"]
mod elevation;
#[path = "style/overlay.rs"]
mod overlay;
#[path = "style/state.rs"]
mod state;
#[path = "style/surface_roles.rs"]
mod surface_roles;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use colors::{
    border_color, surface_color, text_color,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use dimensions::{
    template_border_width, template_corner_radius,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use elevation::{
    draws_elevation_shadow, elevation_shadow_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use overlay::is_mui_overlay_surface_node;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use state::is_button_disabled;

#[cfg(test)]
#[path = "style_tests/tests/mod.rs"]
mod tests;
