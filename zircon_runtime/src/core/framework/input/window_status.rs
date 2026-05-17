use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowTheme {
    Unknown,
    Light,
    Dark,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WindowStatusEvent {
    Moved { x: i32, y: i32 },
    Occluded(bool),
    ThemeChanged(WindowTheme),
    ScaleFactorChanged { scale_factor: f32 },
    BackendScaleFactorChanged { scale_factor: f32 },
    CloseRequested,
    Destroyed,
}
