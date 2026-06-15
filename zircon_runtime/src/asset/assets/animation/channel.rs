use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationInterpolationAsset {
    Step,
    Hermite,
    Linear,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationChannelValueAsset {
    Bool(bool),
    Integer(i32),
    Scalar(Real),
    Vec2([Real; 2]),
    Vec3([Real; 3]),
    Vec4([Real; 4]),
    Quaternion([Real; 4]),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationChannelValueBinary {
    tag: u8,
    bool_value: bool,
    integer_value: i32,
    scalar_values: [Real; 4],
    arity: u8,
}

impl From<&AnimationChannelValueAsset> for AnimationChannelValueBinary {
    fn from(value: &AnimationChannelValueAsset) -> Self {
        match value {
            AnimationChannelValueAsset::Bool(bool_value) => Self {
                tag: 0,
                bool_value: *bool_value,
                integer_value: 0,
                scalar_values: [0.0; 4],
                arity: 0,
            },
            AnimationChannelValueAsset::Integer(integer_value) => Self {
                tag: 1,
                bool_value: false,
                integer_value: *integer_value,
                scalar_values: [0.0; 4],
                arity: 0,
            },
            AnimationChannelValueAsset::Scalar(value) => Self {
                tag: 2,
                bool_value: false,
                integer_value: 0,
                scalar_values: [*value, 0.0, 0.0, 0.0],
                arity: 1,
            },
            AnimationChannelValueAsset::Vec2(value) => Self {
                tag: 3,
                bool_value: false,
                integer_value: 0,
                scalar_values: [value[0], value[1], 0.0, 0.0],
                arity: 2,
            },
            AnimationChannelValueAsset::Vec3(value) => Self {
                tag: 4,
                bool_value: false,
                integer_value: 0,
                scalar_values: [value[0], value[1], value[2], 0.0],
                arity: 3,
            },
            AnimationChannelValueAsset::Vec4(value) => Self {
                tag: 5,
                bool_value: false,
                integer_value: 0,
                scalar_values: *value,
                arity: 4,
            },
            AnimationChannelValueAsset::Quaternion(value) => Self {
                tag: 6,
                bool_value: false,
                integer_value: 0,
                scalar_values: *value,
                arity: 4,
            },
        }
    }
}

impl TryFrom<AnimationChannelValueBinary> for AnimationChannelValueAsset {
    type Error = String;

    fn try_from(value: AnimationChannelValueBinary) -> Result<Self, Self::Error> {
        match value.tag {
            0 => Ok(Self::Bool(value.bool_value)),
            1 => Ok(Self::Integer(value.integer_value)),
            2 => Ok(Self::Scalar(value.scalar_values[0])),
            3 => Ok(Self::Vec2([value.scalar_values[0], value.scalar_values[1]])),
            4 => Ok(Self::Vec3([
                value.scalar_values[0],
                value.scalar_values[1],
                value.scalar_values[2],
            ])),
            5 => Ok(Self::Vec4(value.scalar_values)),
            6 => Ok(Self::Quaternion(value.scalar_values)),
            other => Err(format!("unknown animation channel value tag {other}")),
        }
    }
}

impl Serialize for AnimationChannelValueAsset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        AnimationChannelValueBinary::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AnimationChannelValueAsset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        AnimationChannelValueBinary::deserialize(deserializer)?
            .try_into()
            .map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationChannelKeyAsset {
    pub time_seconds: Real,
    pub value: AnimationChannelValueAsset,
    pub in_tangent: Option<AnimationChannelValueAsset>,
    pub out_tangent: Option<AnimationChannelValueAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationChannelAsset {
    pub interpolation: AnimationInterpolationAsset,
    pub keys: Vec<AnimationChannelKeyAsset>,
}
