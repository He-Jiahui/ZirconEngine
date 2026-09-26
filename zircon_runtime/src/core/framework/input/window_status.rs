use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowTheme {
    Unknown,
    Light,
    Dark,
}

/// 宿主窗口状态变化的帧级通知；输入管理器在下一帧开始时清空，消费方需及时读取。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WindowStatusEvent {
    Moved { x: i32, y: i32 },
    Occluded(bool),
    ThemeChanged(WindowTheme),
    ScaleFactorChanged { scale_factor: f32 },
    BackendScaleFactorChanged { scale_factor: f32 },
    SurfaceRecreated,
    CloseRequested,
    Destroyed,
}
