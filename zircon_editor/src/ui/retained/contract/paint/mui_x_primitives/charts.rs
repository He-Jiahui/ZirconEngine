#[path = "charts/bars.rs"]
mod bars;
#[path = "charts/identity.rs"]
mod identity;
#[path = "charts/raster.rs"]
mod raster;
#[path = "../charts/raster_commands.rs"]
mod raster_commands;
#[path = "charts/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::{
    chart_kind, ChartKind,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use surface::push_chart;
