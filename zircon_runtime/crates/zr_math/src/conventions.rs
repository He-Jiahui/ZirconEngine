//! 定义跨模块交换坐标、单位和精度时使用的词汇；这些标签本身不执行坐标转换或数值校验。

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Axis3 {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AxisDirection {
    Positive(Axis3),
    Negative(Axis3),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CoordinateHandedness {
    Right,
    Left,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MatrixConvention {
    ColumnVectorColumnMajor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ClipDepthRange {
    ZeroToOne,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DepthDirection {
    NearToFar,
    FarToNear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FrontFaceWinding {
    CounterClockwise,
    Clockwise,
}

/// Semantic location of coordinates at a cross-module boundary.
/// 供模块边界标注局部、父级、世界及渲染阶段坐标；标签只分类，不执行空间变换。
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpaceKind {
    Local,
    Parent,
    World,
    ViewRelative,
    View,
    Clip,
    Screen,
}

impl SpaceKind {
    /// 判断标签是否属于视图相对、视图、裁剪或屏幕空间，不转换坐标值。
    pub const fn is_render_space(self) -> bool {
        matches!(
            self,
            Self::ViewRelative | Self::View | Self::Clip | Self::Screen
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LengthUnit {
    Meter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AngleUnit {
    Radian,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TimeUnit {
    Second,
}

/// 供边界契约记录标量存储精度；选择 F64 不会切换当前的运行时数学别名。
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScalarPrecision {
    F32,
    F64,
}

impl ScalarPrecision {
    pub const fn bytes(self) -> usize {
        match self {
            Self::F32 => core::mem::size_of::<f32>(),
            Self::F64 => core::mem::size_of::<f64>(),
        }
    }
}
