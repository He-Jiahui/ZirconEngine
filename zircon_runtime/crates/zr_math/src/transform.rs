use super::{
    is_finite_quat, is_finite_vec3, Axis3, Mat4, NumericError, NumericPolicy, NumericValue, Quat,
    Real, Vec3,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

const LOOK_AT_COLLINEAR_DOT_LIMIT: Real = 1.0 - 4.0 * Real::EPSILON;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum LookAtError {
    #[error("look-at eye must be finite")]
    NonFiniteEye,
    #[error("look-at target must be finite")]
    NonFiniteTarget,
    #[error("look-at forward direction is invalid: {source}")]
    InvalidForward {
        #[source]
        source: NumericError,
    },
    #[error("look-at up direction is invalid: {source}")]
    InvalidUp {
        #[source]
        source: NumericError,
    },
    #[error("look-at view and up axes are collinear within squared threshold {minimum_squared}")]
    CollinearAxes { minimum_squared: Real },
    #[error("look-at basis is invalid: {source}")]
    InvalidBasis {
        #[source]
        source: NumericError,
    },
    #[error("look-at rotation is invalid: {source}")]
    InvalidRotation {
        #[source]
        source: NumericError,
    },
}

impl Transform {
    pub const fn identity() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::identity()
        }
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn validate(self, policy: NumericPolicy) -> Result<ValidatedTransform, NumericError> {
        ValidatedTransform::try_new(self, policy)
    }

    pub fn matrix(self) -> Mat4 {
        transform_to_mat4(self)
    }

    pub fn forward(self) -> Vec3 {
        (self.rotation * -Vec3::Z).normalize_or_zero()
    }

    pub fn right(self) -> Vec3 {
        (self.rotation * Vec3::X).normalize_or_zero()
    }

    pub fn up(self) -> Vec3 {
        (self.rotation * Vec3::Y).normalize_or_zero()
    }

    /// Builds a camera transform and selects a deterministic orthogonal axis when the requested
    /// forward or up direction cannot define a basis. Use [`Self::try_looking_at`] when invalid
    /// input must be reported instead of recovered.
    pub fn looking_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let forward = (target - eye).try_normalize().unwrap_or(Vec3::NEG_Z);
        let requested_up = up.try_normalize().unwrap_or(Vec3::Y);
        let basis_up = if forward.dot(requested_up).abs() >= LOOK_AT_COLLINEAR_DOT_LIMIT {
            least_aligned_cardinal_axis(forward)
        } else {
            requested_up
        };
        let right = forward.cross(basis_up).normalize();
        let corrected_up = right.cross(forward);
        let basis = Mat4::from_cols(
            right.extend(0.0),
            corrected_up.extend(0.0),
            (-forward).extend(0.0),
            eye.extend(1.0),
        );

        Self {
            translation: eye,
            rotation: Quat::from_mat4(&basis).normalize(),
            scale: Vec3::ONE,
        }
    }

    pub fn try_looking_at(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        policy: NumericPolicy,
    ) -> Result<Self, LookAtError> {
        if !is_finite_vec3(eye) {
            return Err(LookAtError::NonFiniteEye);
        }
        if !is_finite_vec3(target) {
            return Err(LookAtError::NonFiniteTarget);
        }

        let forward = UnitDirection3::try_new(target - eye, policy)
            .map_err(|source| LookAtError::InvalidForward { source })?;
        let up = UnitDirection3::try_new(up, policy)
            .map_err(|source| LookAtError::InvalidUp { source })?;
        let right = match UnitDirection3::try_new(forward.as_vec3().cross(up.as_vec3()), policy) {
            Ok(value) => value,
            Err(NumericError::NormTooSmall { .. }) => {
                return Err(LookAtError::CollinearAxes {
                    minimum_squared: policy.minimum_normalized_length_squared(),
                });
            }
            Err(source) => return Err(LookAtError::InvalidBasis { source }),
        };
        let corrected_up =
            UnitDirection3::try_new(right.as_vec3().cross(forward.as_vec3()), policy)
                .map_err(|source| LookAtError::InvalidBasis { source })?;
        let basis = Mat4::from_cols(
            right.as_vec3().extend(0.0),
            corrected_up.as_vec3().extend(0.0),
            (-forward.as_vec3()).extend(0.0),
            eye.extend(1.0),
        );
        let rotation = UnitQuaternion::try_new(Quat::from_mat4(&basis), policy)
            .map_err(|source| LookAtError::InvalidRotation { source })?;

        Ok(Self {
            translation: eye,
            rotation: rotation.into_quat(),
            scale: Vec3::ONE,
        })
    }
}

fn least_aligned_cardinal_axis(direction: Vec3) -> Vec3 {
    let absolute = direction.abs();
    if absolute.y <= absolute.x && absolute.y <= absolute.z {
        Vec3::Y
    } else if absolute.x <= absolute.z {
        Vec3::X
    } else {
        Vec3::Z
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

pub fn compose_trs(translation: Vec3, rotation: Quat, scale: Vec3) -> Mat4 {
    Mat4::from_scale_rotation_translation(scale, rotation, translation)
}

pub fn transform_to_mat4(transform: Transform) -> Mat4 {
    compose_trs(transform.translation, transform.rotation, transform.scale)
}

pub fn affine_inverse(matrix: Mat4) -> Mat4 {
    matrix.inverse()
}

pub fn view_matrix(transform: Transform) -> Mat4 {
    affine_inverse(transform_to_mat4(transform))
}

/// A finite, normalized direction that cannot silently collapse to zero.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct UnitDirection3(Vec3);

impl UnitDirection3 {
    pub fn try_new(value: Vec3, policy: NumericPolicy) -> Result<Self, NumericError> {
        Ok(Self(normalize_vec3(
            value,
            NumericValue::Direction,
            policy.minimum_normalized_length_squared(),
        )?))
    }

    pub const fn as_vec3(self) -> Vec3 {
        self.0
    }

    pub const fn into_vec3(self) -> Vec3 {
        self.0
    }
}

impl<'de> Deserialize<'de> for UnitDirection3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Vec3::deserialize(deserializer)?;
        Self::try_new(value, NumericPolicy::STRICT).map_err(serde::de::Error::custom)
    }
}

/// A finite, normalized quaternion intended for transform rotation boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct UnitQuaternion(Quat);

impl UnitQuaternion {
    pub fn try_new(value: Quat, policy: NumericPolicy) -> Result<Self, NumericError> {
        Ok(Self(normalize_quat(
            value,
            NumericValue::Rotation,
            policy.minimum_normalized_length_squared(),
        )?))
    }

    pub const fn as_quat(self) -> Quat {
        self.0
    }

    pub const fn into_quat(self) -> Quat {
        self.0
    }
}

impl<'de> Deserialize<'de> for UnitQuaternion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Quat::deserialize(deserializer)?;
        Self::try_new(value, NumericPolicy::STRICT).map_err(serde::de::Error::custom)
    }
}

/// A transform admitted through a single finite, normalized, nondegenerate boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValidatedTransform(Transform);

impl ValidatedTransform {
    pub fn try_new(transform: Transform, policy: NumericPolicy) -> Result<Self, NumericError> {
        if !is_finite_vec3(transform.translation) {
            return Err(NumericError::NonFinite {
                value: NumericValue::Translation,
            });
        }
        if !is_finite_vec3(transform.scale) {
            return Err(NumericError::NonFinite {
                value: NumericValue::Scale,
            });
        }

        validate_scale(transform.scale, policy.minimum_scale_absolute())?;
        let rotation = UnitQuaternion::try_new(transform.rotation, policy)?.into_quat();
        Ok(Self(Transform {
            translation: transform.translation,
            rotation,
            scale: transform.scale,
        }))
    }

    pub const fn as_transform(self) -> Transform {
        self.0
    }

    pub const fn into_transform(self) -> Transform {
        self.0
    }
}

impl Serialize for ValidatedTransform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ValidatedTransform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let transform = Transform::deserialize(deserializer)?;
        Self::try_new(transform, NumericPolicy::STRICT).map_err(serde::de::Error::custom)
    }
}

fn normalize_vec3(
    value: Vec3,
    value_kind: NumericValue,
    minimum_squared: Real,
) -> Result<Vec3, NumericError> {
    if !is_finite_vec3(value) {
        return Err(NumericError::NonFinite { value: value_kind });
    }
    let squared_norm = value.length_squared();
    if !squared_norm.is_finite() {
        return Err(NumericError::NonFinite { value: value_kind });
    }
    if squared_norm <= minimum_squared {
        return Err(NumericError::NormTooSmall {
            value: value_kind,
            minimum_squared,
        });
    }

    let normalized = value * squared_norm.sqrt().recip();
    if !is_finite_vec3(normalized) {
        return Err(NumericError::NonFinite { value: value_kind });
    }
    Ok(normalized)
}

fn normalize_quat(
    value: Quat,
    value_kind: NumericValue,
    minimum_squared: Real,
) -> Result<Quat, NumericError> {
    if !is_finite_quat(value) {
        return Err(NumericError::NonFinite { value: value_kind });
    }
    let squared_norm = value.length_squared();
    if !squared_norm.is_finite() {
        return Err(NumericError::NonFinite { value: value_kind });
    }
    if squared_norm <= minimum_squared {
        return Err(NumericError::NormTooSmall {
            value: value_kind,
            minimum_squared,
        });
    }

    let normalized = value * squared_norm.sqrt().recip();
    if !is_finite_quat(normalized) {
        return Err(NumericError::NonFinite { value: value_kind });
    }
    Ok(normalized)
}

fn validate_scale(scale: Vec3, minimum_absolute: Real) -> Result<(), NumericError> {
    for (axis, component) in [
        (Axis3::X, scale.x),
        (Axis3::Y, scale.y),
        (Axis3::Z, scale.z),
    ] {
        if component.abs() <= minimum_absolute {
            return Err(NumericError::ScaleTooSmall {
                axis,
                minimum_absolute,
            });
        }
    }
    Ok(())
}
