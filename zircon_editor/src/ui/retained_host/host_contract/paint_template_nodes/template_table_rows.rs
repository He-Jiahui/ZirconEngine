//! 表格有两条入口：工作台专用行接管底面和操作，其他表格只在通用 fallback 中接管单元格文字。

mod actions;
mod cells;
mod commands;
mod geometry;
mod identity;
mod layers;
mod metrics;
mod style;
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::{
    push_table_row_commands, push_table_row_text_commands,
};

#[cfg(test)]
#[path = "template_table_rows_tests/tests/mod.rs"]
mod tests;
