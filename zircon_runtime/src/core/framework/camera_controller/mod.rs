//! 相机控制器共用的输入、设置、持续状态与输出契约。
//!
//! 这里拥有可复制的值及其构造边界；[`crate::input::camera_controller`] 执行更新，
//! 调用方负责应用 [`CameraControllerOutput`] 并另行执行光标意图。
//! Runtime-owned reusable camera controller contracts.

mod controller_output;
mod free;
mod orbit;
mod pan;

pub use controller_output::{CameraControllerOutput, CursorGrabIntent, CursorGrabMode};
pub use free::{FreeCameraInput, FreeCameraSettings, FreeCameraState};
pub use orbit::{OrbitCameraAction, OrbitCameraInput, OrbitCameraSettings, OrbitCameraState};
pub use pan::{PanCameraInput, PanCameraSettings, PanCameraState};
