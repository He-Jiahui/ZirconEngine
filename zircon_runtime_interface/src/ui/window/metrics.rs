use serde::{Deserialize, Serialize};

use crate::ui::layout::UiSize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiWindowPixelSize {
    pub width: u32,
    pub height: u32,
}

impl UiWindowPixelSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiWindowPixelPosition {
    pub x: i32,
    pub y: i32,
}

impl UiWindowPixelPosition {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// 同时携带布局使用的逻辑尺寸和呈现使用的物理尺寸，供窗口事件更新布局与栅格尺度。
/// DPI 切换时缩放因子与尺寸可能由不同事件先后送达；消费者不能要求三者立即一致。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiWindowMetrics {
    #[serde(default)]
    pub logical_size: UiSize,
    #[serde(default)]
    pub physical_size: UiWindowPixelSize,
    #[serde(default = "default_scale_factor")]
    pub scale_factor: f64,
}

impl UiWindowMetrics {
    /// 调用方负责保证 scale_factor 有限且大于零；此构造器保留原值。
    pub const fn new(
        logical_size: UiSize,
        physical_size: UiWindowPixelSize,
        scale_factor: f64,
    ) -> Self {
        Self {
            logical_size,
            physical_size,
            scale_factor,
        }
    }
}

impl Default for UiWindowMetrics {
    fn default() -> Self {
        Self {
            logical_size: UiSize::default(),
            physical_size: UiWindowPixelSize::default(),
            scale_factor: default_scale_factor(),
        }
    }
}

const fn default_scale_factor() -> f64 {
    1.0
}
