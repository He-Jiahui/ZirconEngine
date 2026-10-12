//! 相机控制器共用的输入、设置、持续状态与输出契约。
//!
//! 这里拥有可复制的值及其构造边界；[`crate::input::camera_controller`] 执行更新，
//! 调用方负责应用 [`CameraControllerOutput`] 并另行执行光标意图。
//! Runtime-owned reusable camera controller contracts.

#[path = "controller_output.rs"]
mod controller_output;
#[path = "free/mod.rs"]
mod free;
#[path = "orbit/mod.rs"]
mod orbit;
#[path = "pan/mod.rs"]
mod pan;

pub use controller_output::{CameraControllerOutput, CursorGrabIntent, CursorGrabMode};
pub use free::{FreeCameraInput, FreeCameraSettings, FreeCameraState};
pub use orbit::{OrbitCameraAction, OrbitCameraInput, OrbitCameraSettings, OrbitCameraState};
pub use pan::{PanCameraInput, PanCameraSettings, PanCameraState};
