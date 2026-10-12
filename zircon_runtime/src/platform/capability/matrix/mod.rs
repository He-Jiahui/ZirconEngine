#[path = "build_report.rs"]
mod build_report;
#[path = "gamepad.rs"]
mod gamepad;
#[path = "input.rs"]
mod input;
#[path = "linux.rs"]
mod linux;
#[path = "platform_capability_matrix.rs"]
mod platform_capability_matrix;
#[path = "policy.rs"]
mod policy;
#[path = "window.rs"]
mod window;

pub use platform_capability_matrix::PlatformCapabilityMatrix;
