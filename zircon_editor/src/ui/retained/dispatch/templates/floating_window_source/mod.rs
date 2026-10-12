#[path = "../../floating_window_source/bridge.rs"]
mod bridge;
#[path = "error.rs"]
mod error;
#[path = "../../floating_window_source/source_frames.rs"]
mod source_frames;
#[path = "../../floating_window_source/surface.rs"]
mod surface;

pub(crate) use bridge::BuiltinFloatingWindowSourceTemplateBridge;
pub(crate) use source_frames::BuiltinFloatingWindowSourceFrames;
