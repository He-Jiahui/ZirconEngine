#[path = "bridge.rs"]
mod bridge;
#[path = "error.rs"]
mod error;

pub(crate) use bridge::BuiltinPaneSurfaceTemplateBridge;
