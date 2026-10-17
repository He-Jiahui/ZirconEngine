//! 文本核心共享的字节区间、尺寸和矩形值。
//! 此处仅保存数值，坐标空间、文本所有者与验证时机由消费方决定。

use serde::{Deserialize, Serialize};

/// 文本坐标空间中的半开 UTF-8 字节区间；消费端须按字段语义区分源区间与视觉文本区间。
/// 结构本身不校验顺序、长度或字素边界，进入整形和选择投影前仍需验证。
///
/// 两个数值不持有文本；派生 Serde 不绑定文档或验证其边界。
/// 空区间的准入由实际操作决定，字节偏移不自动转换为字符或字素索引。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

/// 文本测量结果的局部几何尺寸，经传输适配转为 UI 尺寸，不携带控件位置。
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TextSize {
    pub width: f32,
    pub height: f32,
}

impl TextSize {
    /// 按调用方给定的宽高组装尺寸。
    ///
    /// # 参数与返回
    /// `width`、`height` 原值写入返回值；不在此检查有限性、非负性或布局预算。
    pub(crate) const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// 文本或装饰线所在坐标空间的矩形；与 UI 传输矩形的转换保留坐标值。
/// 有限值及布局范围由使用该矩形的几何预算负责验证。
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TextFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl TextFrame {
    /// 按调用方坐标空间中的分量组装矩形。
    ///
    /// # 参数与返回
    /// `x`、`y`、`width`、`height` 原值写入返回值；不在此转换坐标空间或验证布局约束。
    pub(crate) const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}
