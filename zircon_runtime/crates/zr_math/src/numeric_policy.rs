use super::{Axis3, Real};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NumericPolicyField {
    NormalizedLengthSquared,
    ScaleAbsolute,
    MatrixDeterminantAbsolute,
}

#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum NumericPolicyError {
    #[error("numeric policy minimum for {field:?} must be finite")]
    NonFiniteMinimum { field: NumericPolicyField },
    #[error("numeric policy minimum for {field:?} must not be negative")]
    NegativeMinimum { field: NumericPolicyField },
}

/// Per-operation lower bounds. Domain-specific policy registries can provide stricter values later.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NumericPolicy {
    minimum_normalized_length_squared: Real,
    minimum_scale_absolute: Real,
    minimum_matrix_determinant_absolute: Real,
}

/// Named numeric thresholds used to construct a checked policy.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NumericPolicyThresholds {
    pub normalized_length_squared: Real,
    pub scale_absolute: Real,
    pub matrix_determinant_absolute: Real,
}

impl NumericPolicy {
    // No implicit epsilon: existing f32 behavior treats only exact zero as degenerate.
    pub const STRICT: Self = Self {
        minimum_normalized_length_squared: 0.0,
        minimum_scale_absolute: 0.0,
        minimum_matrix_determinant_absolute: 0.0,
    };

    pub fn try_new(thresholds: NumericPolicyThresholds) -> Result<Self, NumericPolicyError> {
        validate_policy_minimum(
            thresholds.normalized_length_squared,
            NumericPolicyField::NormalizedLengthSquared,
        )?;
        validate_policy_minimum(thresholds.scale_absolute, NumericPolicyField::ScaleAbsolute)?;
        validate_policy_minimum(
            thresholds.matrix_determinant_absolute,
            NumericPolicyField::MatrixDeterminantAbsolute,
        )?;
        Ok(Self {
            minimum_normalized_length_squared: thresholds.normalized_length_squared,
            minimum_scale_absolute: thresholds.scale_absolute,
            minimum_matrix_determinant_absolute: thresholds.matrix_determinant_absolute,
        })
    }

    pub const fn minimum_normalized_length_squared(self) -> Real {
        self.minimum_normalized_length_squared
    }

    pub const fn minimum_scale_absolute(self) -> Real {
        self.minimum_scale_absolute
    }

    pub const fn minimum_matrix_determinant_absolute(self) -> Real {
        self.minimum_matrix_determinant_absolute
    }

    pub const fn thresholds(self) -> NumericPolicyThresholds {
        NumericPolicyThresholds {
            normalized_length_squared: self.minimum_normalized_length_squared,
            scale_absolute: self.minimum_scale_absolute,
            matrix_determinant_absolute: self.minimum_matrix_determinant_absolute,
        }
    }
}

fn validate_policy_minimum(
    value: Real,
    field: NumericPolicyField,
) -> Result<(), NumericPolicyError> {
    if !value.is_finite() {
        return Err(NumericPolicyError::NonFiniteMinimum { field });
    }
    if value < 0.0 {
        return Err(NumericPolicyError::NegativeMinimum { field });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NumericValue {
    Translation,
    Direction,
    Rotation,
    Scale,
}

#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum NumericError {
    #[error("{value:?} must be finite")]
    NonFinite { value: NumericValue },
    #[error("{value:?} squared norm must exceed {minimum_squared}")]
    NormTooSmall {
        value: NumericValue,
        minimum_squared: Real,
    },
    #[error("scale {axis:?} absolute value must exceed {minimum_absolute}")]
    ScaleTooSmall { axis: Axis3, minimum_absolute: Real },
}
