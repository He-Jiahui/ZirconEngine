#[path = "rich_table/axes.rs"]
mod axes;
#[path = "rich_table/cell_layout.rs"]
mod cell_layout;
#[path = "rich_table/geometry.rs"]
mod geometry;
#[path = "rich_table/grid.rs"]
mod grid;
#[path = "rich_table/layout.rs"]
mod layout;
#[path = "rich_table/sizing.rs"]
mod sizing;
#[path = "rich_table/source_slice.rs"]
mod source_slice;

pub(super) use layout::layout_rich_tables_with_provider;
