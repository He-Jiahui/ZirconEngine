use serde::{Deserialize, Serialize};

use crate::ui::layout::UiSize;

/// 非负的物理像素尺寸；允许零值，默认宽高均为零。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiWindowPixelSize {
    pub width: u32,
    pub height: u32,
}

impl UiWindowPixelSize {
    /// 原样保存宽、高像素数，不生成逻辑尺寸或缩放因子。
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// 有符号物理像素位置；坐标原点由宿主约定，允许负值。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiWindowPixelPosition {
    pub x: i32,
    pub y: i32,
}

impl UiWindowPixelPosition {
    /// 原样保存物理像素坐标，不执行移动或边界裁剪。
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Logical and physical window metrics share one DTO so DPI changes can mark
/// layout metrics dirty without implying input-state mutation.
/// 同时携带布局使用的逻辑尺寸和呈现使用的物理尺寸，供窗口事件更新布局与栅格尺度。
/// DPI 切换时缩放因子与尺寸可能由不同事件先后送达；消费者不能要求三者立即一致。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiWindowMetrics {
    /// 布局使用的逻辑尺寸；缺省为零，不从物理尺寸自动计算。
    #[serde(default)]
    pub logical_size: UiSize,
    /// 呈现使用的物理像素尺寸；缺省为零。
    #[serde(default)]
    pub physical_size: UiWindowPixelSize,
    /// 逻辑/物理尺度之间的来源值；缺省为 1，反序列化不验证有限性或正值。
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

/// 默认值为零逻辑尺寸、零物理尺寸和单位缩放，不表示窗口已经创建或完成布局。
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
