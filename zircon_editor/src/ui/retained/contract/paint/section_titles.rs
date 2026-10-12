//! 工作台 section-title 先认领语义，再组合背景、可选图标和标题文字；未识别节点返回通用模板链。

#[path = "section_titles/commands.rs"]
mod commands;
#[path = "section_titles/geometry.rs"]
mod geometry;
#[path = "section_titles/identity.rs"]
mod identity;
#[path = "section_titles/style.rs"]
mod style;
#[path = "section_titles/surface.rs"]
mod surface;
#[path = "section_titles/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_section_title_commands;

#[cfg(test)]
#[path = "section_titles_tests/tests/mod.rs"]
mod tests;
