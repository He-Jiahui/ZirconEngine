#[path = "cells/allocation.rs"]
mod allocation;
#[path = "cells/commands.rs"]
mod commands;
#[path = "cells/geometry.rs"]
mod geometry;
#[path = "cells/metrics.rs"]
mod metrics;
#[path = "cells/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_table_cells;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::table_cell_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::table_content_offset;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::split_archived_table_text;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::table_cells;
