//! 工作台 section-title 先认领语义，再组合背景、可选图标和标题文字；未识别节点返回通用模板链。

mod commands;
mod geometry;
mod identity;
mod style;
mod surface;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_section_title_commands;

#[cfg(test)]
#[path = "template_section_titles_tests/tests/mod.rs"]
mod tests;
