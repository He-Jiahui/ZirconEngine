//! 树形行的专用绘制边界；secondary 分派命中后接管底面、缩进、对象图标、标题与操作。
//! 返回 handled 后通用 fallback 不再补画这些内容，隐藏或退化几何也算已处理。

#[path = "tree_rows/actions.rs"]
mod actions;
#[path = "tree_rows/commands.rs"]
mod commands;
#[path = "tree_rows/geometry.rs"]
mod geometry;
#[path = "tree_rows/identity.rs"]
mod identity;
#[path = "tree_rows/labels.rs"]
mod labels;
#[path = "tree_rows/layers.rs"]
mod layers;
#[path = "tree_rows/style.rs"]
mod style;
#[path = "tree_rows/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_tree_row_commands;

#[cfg(test)]
#[path = "tree_rows_tests/tests/mod.rs"]
mod tests;
