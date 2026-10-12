#[path = "redraw/damage_region.rs"]
mod damage_region;
#[path = "redraw/dispatch_result.rs"]
mod dispatch_result;
#[path = "redraw/request.rs"]
mod request;

pub(crate) use damage_region::{HostDamageRegion, HostDamageRegionMetrics};
pub(crate) use dispatch_result::NativePointerDispatchResult;
pub(crate) use request::HostRedrawRequest;

#[cfg(test)]
#[path = "tests/redraw.rs"]
mod tests;
