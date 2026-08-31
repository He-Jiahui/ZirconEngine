use crate::serialization::SchemaId;
use zr_math::{
    AngleUnit, Axis3, AxisDirection, ClipDepthRange, CoordinateHandedness, DepthDirection,
    FrontFaceWinding, LengthUnit, MatrixConvention, ScalarPrecision, TimeUnit,
};

/// Versioned conventions shared by authoring, runtime, and render boundaries.
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

    pub const fn cpu_scalar_bytes(&self) -> usize {
        self.runtime_scalar.bytes()
    }

    pub const fn render_scalar_bytes(&self) -> usize {
        self.render_scalar.bytes()
    }
}
