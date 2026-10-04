use serde::{Deserialize, Serialize};

use crate::core::math::UVec2;

use super::constants::{
    DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_SCALE_FACTOR, DEFAULT_WINDOW_WIDTH, MIN_WINDOW_AXIS,
};
use super::validation::{valid_scale_factor, valid_window_axis};

// BUG: [CR-FRAMEWORK-WINDOW-0002] 派生反序列化绕过 new/set 的尺寸和缩放校验，可构造零尺寸或零缩放并流入物理尺寸、逻辑尺寸计算；证据见本类型构造器与 App 窗口属性转换。
/// 启动窗口的物理像素尺寸与 DPI 缩放意图；App 用物理尺寸创建表面，逻辑尺寸供调用者换算与诊断。
/// 构造和修改入口会夹紧无效值；从配置读取时也必须保住相同不变式。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowResolution {
    physical_width: u32,
    physical_height: u32,
    scale_factor_override: Option<f32>,
    scale_factor: f32,
}

impl WindowResolution {
    pub const fn new(physical_width: u32, physical_height: u32) -> Self {
        Self {
            physical_width: if physical_width == 0 {
                1
            } else {
                physical_width
            },
            physical_height: if physical_height == 0 {
                1
            } else {
                physical_height
            },
            scale_factor_override: None,
            scale_factor: DEFAULT_WINDOW_SCALE_FACTOR,
        }
    }

    pub fn with_scale_factor_override(mut self, scale_factor_override: f32) -> Self {
        self.set_scale_factor_override(Some(scale_factor_override));
        self
    }

    pub fn physical_size(&self) -> UVec2 {
        UVec2::new(self.physical_width, self.physical_height)
    }

    pub fn logical_size(&self) -> [f32; 2] {
        [
            self.physical_width as f32 / self.scale_factor(),
            self.physical_height as f32 / self.scale_factor(),
        ]
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor_override
            .unwrap_or_else(|| self.base_scale_factor())
    }

    pub const fn base_scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub const fn scale_factor_override(&self) -> Option<f32> {
        self.scale_factor_override
    }

    pub fn set_logical_size(&mut self, width: f32, height: f32) {
        self.set_physical_size(
            scaled_axis_to_physical(width, self.scale_factor()),
            scaled_axis_to_physical(height, self.scale_factor()),
        );
    }

    pub fn set_physical_size(&mut self, width: u32, height: u32) {
        self.physical_width = width.max(1);
        self.physical_height = height.max(1);
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = valid_scale_factor(scale_factor);
    }

    /// 设置启动配置的强制缩放，仅在调用方确实要覆盖后端 DPI 观测时使用。
    pub fn set_scale_factor_override(&mut self, scale_factor_override: Option<f32>) {
        self.scale_factor_override = scale_factor_override.map(valid_scale_factor);
    }
}

impl Default for WindowResolution {
    fn default() -> Self {
        Self::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
    }
}

impl From<(u32, u32)> for WindowResolution {
    fn from((width, height): (u32, u32)) -> Self {
        Self::new(width, height)
    }
}

impl From<[u32; 2]> for WindowResolution {
    fn from([width, height]: [u32; 2]) -> Self {
        Self::new(width, height)
    }
}

impl From<UVec2> for WindowResolution {
    fn from(size: UVec2) -> Self {
        Self::new(size.x, size.y)
    }
}

fn scaled_axis_to_physical(logical_axis: f32, scale_factor: f32) -> u32 {
    valid_window_axis(logical_axis)
        .mul_add(scale_factor, 0.0)
        .round()
        .max(MIN_WINDOW_AXIS) as u32
}
