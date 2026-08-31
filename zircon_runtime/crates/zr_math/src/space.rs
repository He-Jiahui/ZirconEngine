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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position3 {
    value: Vec3,
    space: SpaceKind,
}

impl Position3 {
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

    pub fn checked_add(self, vector: Vector3) -> Result<Self, SpatialError> {
        require_same_space(self.space, vector.space)?;
        Self::try_new(self.value + vector.value, self.space)
    }

    pub fn checked_sub(self, other: Self) -> Result<Vector3, SpatialError> {
        require_same_space(self.space, other.space)?;
        Vector3::try_new(self.value - other.value, self.space)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector3 {
    value: Vec3,
    space: SpaceKind,
}

impl Vector3 {
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

    pub fn checked_add(self, other: Self) -> Result<Self, SpatialError> {
        require_same_space(self.space, other.space)?;
        Self::try_new(self.value + other.value, self.space)
    }
}

/// A finite unit normal whose coordinate space is explicit at every boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Normal3 {
    direction: UnitDirection3,
    space: SpaceKind,
}

impl Normal3 {
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
