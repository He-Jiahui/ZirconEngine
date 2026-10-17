//! 为位置、向量和法向携带空间标签；受检算术要求标签相同，并检查运算后的有限性。
//! 标签不会执行坐标或单位转换，跨空间操作须由调用者先提供相应变换。

use super::{is_finite_vec3, NumericError, NumericPolicy, SpaceKind, UnitDirection3, Vec3};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpatialValue {
    Position,
    Vector,
    Normal,
}

#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum SpatialError {
    #[error("{value:?} must be finite")]
    NonFinite { value: SpatialValue },
    #[error("spatial operation requires matching spaces, got {left:?} and {right:?}")]
    SpaceMismatch { left: SpaceKind, right: SpaceKind },
    #[error("normal must be a finite, nonzero direction: {source}")]
    InvalidNormal {
        #[source]
        source: NumericError,
    },
}

/// 带空间标签的有限位置；相加只接受同空间位移，相减返回同空间位移。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position3 {
    value: Vec3,
    space: SpaceKind,
}

impl Position3 {
    /// 构造有限位置；space 仅标记坐标所属空间，不执行变换。
    pub fn try_new(value: Vec3, space: SpaceKind) -> Result<Self, SpatialError> {
        is_finite_vec3(value)
            .then_some(Self { value, space })
            .ok_or(SpatialError::NonFinite {
                value: SpatialValue::Position,
            })
    }

    pub const fn value(self) -> Vec3 {
        self.value
    }

    pub const fn space(self) -> SpaceKind {
        self.space
    }

    /// 将同空间位移加到此位置，并拒绝空间不匹配或非有限结果。
    pub fn checked_add(self, vector: Vector3) -> Result<Self, SpatialError> {
        require_same_space(self.space, vector.space)?;
        Self::try_new(self.value + vector.value, self.space)
    }

    /// 计算两个同空间位置之差，并以该空间标记返回位移。
    pub fn checked_sub(self, other: Self) -> Result<Vector3, SpatialError> {
        require_same_space(self.space, other.space)?;
        Vector3::try_new(self.value - other.value, self.space)
    }
}

/// 带空间标签的有限位移或方向量；其受检加法要求两个向量处于同一空间。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector3 {
    value: Vec3,
    space: SpaceKind,
}

impl Vector3 {
    /// 构造有限向量；space 只记录分量所属空间。
    pub fn try_new(value: Vec3, space: SpaceKind) -> Result<Self, SpatialError> {
        is_finite_vec3(value)
            .then_some(Self { value, space })
            .ok_or(SpatialError::NonFinite {
                value: SpatialValue::Vector,
            })
    }

    pub const fn value(self) -> Vec3 {
        self.value
    }

    pub const fn space(self) -> SpaceKind {
        self.space
    }

    /// 将同空间向量相加，并拒绝空间不匹配或非有限结果。
    pub fn checked_add(self, other: Self) -> Result<Self, SpatialError> {
        require_same_space(self.space, other.space)?;
        Self::try_new(self.value + other.value, self.space)
    }
}

/// A finite unit normal whose coordinate space is explicit at every boundary.
/// 法向构造和反序列化都会按 NumericPolicy::STRICT 归一化，空间标签不参与转换。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Normal3 {
    direction: UnitDirection3,
    space: SpaceKind,
}

impl Normal3 {
    /// 保存法向时使用 STRICT 下限归一化；输入及计算出的范数须有效，space 只记录所属空间。
    pub fn try_new(value: Vec3, space: SpaceKind) -> Result<Self, SpatialError> {
        let direction = UnitDirection3::try_new(value, NumericPolicy::STRICT)
            .map_err(|source| SpatialError::InvalidNormal { source })?;
        Ok(Self { direction, space })
    }

    pub const fn value(self) -> Vec3 {
        self.direction.as_vec3()
    }

    pub const fn space(self) -> SpaceKind {
        self.space
    }
}

// 位置、向量和法向共用此线格式；各自反序列化仍须经构造器恢复有限性或单位长度约束。
#[derive(Serialize, Deserialize)]
struct SpatialVec3Wire {
    value: Vec3,
    space: SpaceKind,
}

impl Serialize for Position3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SpatialVec3Wire {
            value: self.value,
            space: self.space,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Position3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SpatialVec3Wire::deserialize(deserializer)?;
        Self::try_new(wire.value, wire.space).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Vector3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SpatialVec3Wire {
            value: self.value,
            space: self.space,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Vector3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SpatialVec3Wire::deserialize(deserializer)?;
        Self::try_new(wire.value, wire.space).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Normal3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SpatialVec3Wire {
            value: self.value(),
            space: self.space,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Normal3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SpatialVec3Wire::deserialize(deserializer)?;
        Self::try_new(wire.value, wire.space).map_err(serde::de::Error::custom)
    }
}

fn require_same_space(left: SpaceKind, right: SpaceKind) -> Result<(), SpatialError> {
    (left == right)
        .then_some(())
        .ok_or(SpatialError::SpaceMismatch { left, right })
}
