//! secondary 专用链的紧凑 chip 绘制边界；状态控件和普通按钮拥有各自更早或更晚的归属判断。

mod commands;
mod geometry;
mod identity;
mod layers;
mod metrics;
mod style;
mod surface;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_chip_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    chip_chevron_reserve, chip_chevron_right, chip_chevron_size,
};

#[cfg(test)]
#[path = "template_chips_tests/tests/mod.rs"]
mod tests;
