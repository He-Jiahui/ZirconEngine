mod dispatch_result;
mod request;

pub(crate) use dispatch_result::NativePointerDispatchResult;
pub(crate) use request::HostRedrawRequest;

#[cfg(test)]
#[path = "redraw_tests.rs"]
mod tests;
