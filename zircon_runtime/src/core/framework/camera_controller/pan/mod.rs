//! 平面相机的帧输入、设置和持续缩放状态契约。
//!
//! 值的解释与更新顺序由 [`crate::input::camera_controller::PanCameraController`] 承担；
//! 本模块不应用场景变换、不持有宿主资源，也不决定宿主何时投递输入。
mod input;
mod settings;
mod state;

pub use input::PanCameraInput;
pub use settings::PanCameraSettings;
pub use state::PanCameraState;
