//! 自由相机的帧输入、设置和持续状态契约。
//!
//! 这里只定义并导出值；积分和光标意图生成由 [`crate::input::camera_controller::FreeCameraController`] 承担，
//! 窗口执行以及把变换应用到场景由调用方负责。
mod input;
mod settings;
mod state;

pub use input::FreeCameraInput;
pub use settings::FreeCameraSettings;
pub use state::FreeCameraState;
