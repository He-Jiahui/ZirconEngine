#[path = "raster_commands/cache.rs"]
mod cache;
#[path = "raster_commands/commands.rs"]
mod commands;
#[path = "../../../../contract/paint/charts/raster_commands/dimensions.rs"]
mod dimensions;
#[path = "raster_commands/gauge.rs"]
mod gauge;
#[path = "raster_commands/identity.rs"]
mod identity;
#[path = "raster_commands/line.rs"]
mod line;
#[path = "raster_commands/pie.rs"]
mod pie;
#[path = "raster_commands/sparkline.rs"]
mod sparkline;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_chart_raster;
