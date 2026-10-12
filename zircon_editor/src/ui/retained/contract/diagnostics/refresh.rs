#[path = "refresh/model.rs"]
mod model;
#[path = "refresh/overlay_text.rs"]
mod overlay_text;
#[path = "refresh/timing.rs"]
mod timing;

pub(crate) use self::model::HostRefreshDiagnostics;
