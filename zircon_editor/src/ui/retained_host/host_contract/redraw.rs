mod damage_region;
mod dispatch_result;
mod request;

pub(crate) use damage_region::{HostDamageRegion, HostDamageRegionMetrics};
pub(crate) use dispatch_result::NativePointerDispatchResult;
pub(crate) use request::HostRedrawRequest;

#[cfg(test)]
#[path = "redraw_tests.rs"]
mod tests;
