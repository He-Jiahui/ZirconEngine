//! Neutral window contracts shared by runtime modules and host backends.

mod constants;
mod descriptor;
mod mode;
mod position;
mod present_mode;
mod primary_window_handle;
mod resize_constraints;
mod resolution;
mod validation;

pub use constants::DEFAULT_WINDOW_TITLE;
pub use descriptor::WindowDescriptor;
pub use mode::WindowMode;
pub use position::WindowPosition;
pub use present_mode::WindowPresentMode;
pub use primary_window_handle::PrimaryWindowHandle;
pub use resize_constraints::WindowResizeConstraints;
pub use resolution::WindowResolution;

#[cfg(test)]
mod tests;
