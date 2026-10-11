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
