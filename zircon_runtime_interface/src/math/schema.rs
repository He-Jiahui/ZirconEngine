//! 版本化声明坐标、单位与精度约定，供运行时和编辑器解释同一数值数据。
//! 具体数学运算仍由 `zr_math` 实现；这里固定产品层序列化身份。
use crate::serialization::SchemaId;
use zr_math::{
    AngleUnit, Axis3, AxisDirection, ClipDepthRange, CoordinateHandedness, DepthDirection,
    FrontFaceWinding, LengthUnit, MatrixConvention, ScalarPrecision, TimeUnit,
};

/// Versioned conventions shared by authoring, runtime, and render boundaries.
/// 供创作、运行时和渲染边界共同解释坐标值的版本化约定。
/// 字段进入序列化载荷后，`schema_id` 与 `version` 必须随数据一起保留。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CoordinateSchema {
    pub schema_id: SchemaId,
    pub version: u16,
    pub handedness: CoordinateHandedness,
    pub up: AxisDirection,
    pub forward: AxisDirection,
    pub matrix_convention: MatrixConvention,
    pub clip_depth_range: ClipDepthRange,
    pub depth_direction: DepthDirection,
    pub canonical_front_face: FrontFaceWinding,
}

/// 当前产品约定：右手系、Y 向上、负 Z 向前，深度映射为零到一。
pub const ZIRCON_COORDINATE_SCHEMA: CoordinateSchema = CoordinateSchema {
    schema_id: SchemaId::new("zircon.coordinate"),
    version: 1,
    handedness: CoordinateHandedness::Right,
    up: AxisDirection::Positive(Axis3::Y),
    forward: AxisDirection::Negative(Axis3::Z),
    matrix_convention: MatrixConvention::ColumnVectorColumnMajor,
    clip_depth_range: ClipDepthRange::ZeroToOne,
    depth_direction: DepthDirection::NearToFar,
    canonical_front_face: FrontFaceWinding::CounterClockwise,
};

/// Authoritative base units paired with the coordinate schema they describe.
/// 与所描述坐标系配套的权威基础单位；转换方必须先核对嵌套坐标 schema。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitSchema {
    pub schema_id: SchemaId,
    pub version: u16,
    pub coordinate: CoordinateSchema,
    pub length: LengthUnit,
    pub angle: AngleUnit,
    pub time: TimeUnit,
}

pub const ZIRCON_UNIT_SCHEMA: UnitSchema = UnitSchema {
    schema_id: SchemaId::new("zircon.units"),
    version: 1,
    coordinate: ZIRCON_COORDINATE_SCHEMA,
    length: LengthUnit::Meter,
    angle: AngleUnit::Radian,
    time: TimeUnit::Second,
};

/// Versioned product-level numeric identity for runtime and render boundaries.
/// 运行时与渲染边界使用的版本化数值精度身份，区分 CPU 计算类型和渲染窄化类型。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrecisionProfile {
    pub schema_id: SchemaId,
    pub version: u16,
    pub runtime_scalar: ScalarPrecision,
    pub render_scalar: ScalarPrecision,
}

pub const ZIRCON_PRECISION_PROFILE: PrecisionProfile = PrecisionProfile {
    schema_id: SchemaId::new("zircon.precision"),
    version: 1,
    runtime_scalar: ScalarPrecision::F32,
    render_scalar: ScalarPrecision::F32,
};

impl PrecisionProfile {
    pub const CURRENT: Self = ZIRCON_PRECISION_PROFILE;

    /// 返回运行时标量的字节宽度，不执行数值转换。
    pub const fn cpu_scalar_bytes(&self) -> usize {
        self.runtime_scalar.bytes()
    }

    /// 返回渲染标量的字节宽度；调用方仍须使用显式窄化 API 检查可表示性。
    pub const fn render_scalar_bytes(&self) -> usize {
        self.render_scalar.bytes()
    }
}
