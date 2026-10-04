//! 变换轴输入外观的共享样式入口；surface/text 分别消费背景、轮廓、文字选择结果，保证同一状态优先级。

mod background;
mod border;
mod colors;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use background::axis_field_background;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use border::{
    axis_field_border, axis_field_border_width,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::axis_field_text_color;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use background::axis_field_background_from_host;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use border::axis_field_border_from_host;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::axis_field_text_color_from_host;
