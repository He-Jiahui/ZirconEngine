//! 轨道相机的动作、输入、设置和持续目标契约。
//!
//! 这里只定义动作值和状态值；[`crate::input::camera_controller::OrbitCameraController`] 消费动作，
//! Editor 与动态 Runtime 调用方负责维护目标和相机变换所在的坐标空间，并应用输出。
mod action;
mod input;
mod settings;
mod state;

pub use action::OrbitCameraAction;
pub use input::OrbitCameraInput;
pub use settings::OrbitCameraSettings;
pub use state::OrbitCameraState;
