//! secondary 专用链的紧凑 chip 绘制边界；状态控件和普通按钮拥有各自更早或更晚的归属判断。

#[path = "chips/commands.rs"]
mod commands;
#[path = "chips/geometry.rs"]
mod geometry;
#[path = "chips/identity.rs"]
mod identity;
#[path = "chips/layers.rs"]
mod layers;
#[path = "chips/metrics.rs"]
mod metrics;
#[path = "chips/style.rs"]
mod style;
#[path = "chips/surface.rs"]
mod surface;
#[path = "chips/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_chip_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    chip_chevron_reserve, chip_chevron_right, chip_chevron_size,
};

#[cfg(test)]
#[path = "chips_tests/tests/mod.rs"]
mod tests;
