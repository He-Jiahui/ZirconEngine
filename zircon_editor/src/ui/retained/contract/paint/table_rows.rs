//! 表格有两条入口：工作台专用行接管底面和操作，其他表格只在通用 fallback 中接管单元格文字。

#[path = "table_rows/actions.rs"]
mod actions;
#[path = "table_rows/cells.rs"]
mod cells;
#[path = "table_rows/commands.rs"]
mod commands;
#[path = "table_rows/geometry.rs"]
mod geometry;
#[path = "table_rows/identity.rs"]
mod identity;
#[path = "table_rows/layers.rs"]
mod layers;
#[path = "table_rows/metrics.rs"]
mod metrics;
#[path = "table_rows/style.rs"]
mod style;
#[path = "table_rows/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::{
    push_table_row_commands, push_table_row_text_commands,
};

#[cfg(test)]
#[path = "table_rows_tests/tests/mod.rs"]
mod tests;
