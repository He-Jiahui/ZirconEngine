//! secondary 专用链的工作台滑块入口；轨道、滑块、刻度、标签与数值浮层由子模块组合。

mod commands;
mod identity;
mod layers;
mod text;
mod thumb;
mod track;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_slider_commands;

#[cfg(test)]
use identity::{is_workbench_slider, slider_style};

#[cfg(test)]
#[path = "template_sliders_tests/tests/mod.rs"]
mod tests;
