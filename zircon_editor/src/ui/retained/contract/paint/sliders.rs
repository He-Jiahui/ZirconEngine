//! secondary 专用链的工作台滑块入口；轨道、滑块、刻度、标签与数值浮层由子模块组合。

#[path = "sliders/commands.rs"]
mod commands;
#[path = "sliders/identity.rs"]
mod identity;
#[path = "sliders/layers.rs"]
mod layers;
#[path = "sliders/text.rs"]
mod text;
#[path = "sliders/thumb.rs"]
mod thumb;
#[path = "sliders/track.rs"]
mod track;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_slider_commands;

#[cfg(test)]
use identity::{is_workbench_slider, slider_style};

#[cfg(test)]
#[path = "sliders_tests/tests/mod.rs"]
mod tests;
