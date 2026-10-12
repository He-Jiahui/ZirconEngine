#[path = "../../charts/raster/arc.rs"]
mod arc;
#[path = "../../charts/raster/line.rs"]
mod line;
#[path = "../../charts/raster/math.rs"]
mod math;
#[path = "../../charts/raster/model.rs"]
mod model;
#[path = "../../charts/raster/pie.rs"]
mod pie;
#[path = "../../charts/raster/shape.rs"]
mod shape;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::ChartRaster;
