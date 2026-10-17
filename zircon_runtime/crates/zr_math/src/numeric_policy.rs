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
/// 阈值依次作用于归一化长度平方、每轴缩放绝对值和仿射行列式绝对值；try_new 要求其有限且非负。
/// 派生的 Deserialize 不经过该构造校验，因此反序列化值不保证满足相同约束。
/// TODO: [ZCA-M001-001] 确认策略反序列化的接纳契约，并验证负缩放阈值与 ValidatedTransform::try_new 接受零缩放的组合。
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NumericPolicy {
    minimum_normalized_length_squared: Real,
    minimum_scale_absolute: Real,
    minimum_matrix_determinant_absolute: Real,
}

/// Named numeric thresholds used to construct a checked policy.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NumericPolicyThresholds {
    /// 方向或旋转归一化时要求超过的范数平方下限。
    pub normalized_length_squared: Real,
    /// 每个缩放轴要求超过的绝对值下限。
    pub scale_absolute: Real,
    /// 仿射求逆时要求超过的行列式绝对值下限。
    pub matrix_determinant_absolute: Real,
}

impl NumericPolicy {
    // No implicit epsilon: existing f32 behavior treats only exact zero as degenerate.
    /// 所有下限均为零；退化判断比较计算出的范数平方、缩放绝对值或行列式绝对值，不含隐式 epsilon。
    pub const STRICT: Self = Self {
        minimum_normalized_length_squared: 0.0,
        minimum_scale_absolute: 0.0,
        minimum_matrix_determinant_absolute: 0.0,
    };

    /// 从外部阈值构造策略时使用，要求每个下限有限且非负；各运算仍分别检查输入和计算结果。
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
