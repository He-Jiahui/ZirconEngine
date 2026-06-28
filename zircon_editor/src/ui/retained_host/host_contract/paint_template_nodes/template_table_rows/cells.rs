mod allocation;
mod commands;
mod geometry;
mod metrics;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_table_cells;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::table_cell_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::table_content_offset;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::TABLE_ACTION_WIDTH;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::split_archived_table_text;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::table_cells;
