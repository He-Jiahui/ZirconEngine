//! Neutral window contracts shared by runtime modules and host backends.

mod constants;
mod descriptor;
mod lifecycle_policy;
mod mode;
mod monitor_selection;
mod position;
mod present_mode;
mod primary_window_handle;
mod resize_constraints;
mod resolution;
mod validation;
mod video_mode_selection;

pub use constants::{DEFAULT_WINDOW_TITLE, PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY};
pub use descriptor::WindowDescriptor;
pub use lifecycle_policy::{WindowExitCondition, WindowLifecyclePolicy};
pub use mode::WindowMode;
pub use monitor_selection::WindowMonitorSelection;
pub use position::WindowPosition;
pub use present_mode::WindowPresentMode;
pub use primary_window_handle::PrimaryWindowHandle;
pub use resize_constraints::WindowResizeConstraints;
pub use resolution::WindowResolution;
pub use video_mode_selection::{WindowVideoMode, WindowVideoModeSelection};

#[cfg(test)]
mod tests;
