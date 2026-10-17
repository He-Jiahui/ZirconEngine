use super::{is_finite_mat4, DepthDirection, Mat4, NumericPolicy, Real};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum PerspectiveError {
    #[error("perspective inputs must be finite")]
    NonFiniteInput,
    #[error("perspective field of view must be within (0, PI)")]
    FieldOfViewOutOfRange,
    #[error("perspective aspect ratio must be positive")]
    AspectRatioNotPositive,
    #[error("perspective near plane must be positive")]
    NearPlaneNotPositive,
    #[error("perspective far plane must be greater than the near plane")]
    FarPlaneNotAfterNear,
    #[error("perspective matrix must be finite")]
    NonFiniteMatrix,
}

#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum AffineInverseError {
    #[error("matrix to invert must be finite")]
    NonFiniteInput,
    #[error("matrix to invert must be affine")]
    NonAffineInput,
    #[error("matrix determinant must be finite")]
    NonFiniteDeterminant,
    #[error("matrix determinant magnitude must exceed {minimum_absolute}")]
    DeterminantTooSmall { minimum_absolute: Real },
    #[error("matrix inverse must be finite")]
    NonFiniteResult,
}

/// A finite, conventional perspective projection that matches the current coordinate schema.
///
/// The runtime currently uses near-to-far depth with a finite far plane. Reversed-depth and
/// infinite-far projections require their own schema and render-extraction contracts rather than
/// silently changing this value object's meaning.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValidatedPerspective {
    fov_y_radians: Real,
    aspect_ratio: Real,
    z_near: Real,
    z_far: Real,
    matrix: Mat4,
}

impl ValidatedPerspective {
    /// 校验弧度制垂直视场角、宽高比和有限近平面范围，并保存对应投影矩阵。
    /// 返回值只代表当前近到远有限远平面约定下的构造检查通过。
    pub fn new(
        fov_y_radians: Real,
        aspect_ratio: Real,
        z_near: Real,
        z_far: Real,
    ) -> Result<Self, PerspectiveError> {
        let matrix = try_perspective(fov_y_radians, aspect_ratio, z_near, z_far)?;
        Ok(Self {
            fov_y_radians,
            aspect_ratio,
            z_near,
            z_far,
            matrix,
        })
    }

    pub const fn depth_direction(self) -> DepthDirection {
        DepthDirection::NearToFar
    }

    pub const fn fov_y_radians(self) -> Real {
        self.fov_y_radians
    }

    pub const fn aspect_ratio(self) -> Real {
        self.aspect_ratio
    }

    pub const fn z_near(self) -> Real {
        self.z_near
    }

    pub const fn z_far(self) -> Real {
        self.z_far
    }

    pub const fn matrix(self) -> Mat4 {
        self.matrix
    }
}

/// 在需要报告求逆失败时使用：输入必须有限并精确满足仿射齐次行，行列式绝对值须超过策略下限。
/// 成功只表示这些检查及逆矩阵有限性成立，不提供条件数或逆乘残差的误差界。
pub fn try_affine_inverse(matrix: Mat4, policy: NumericPolicy) -> Result<Mat4, AffineInverseError> {
    if !is_finite_mat4(matrix) {
        return Err(AffineInverseError::NonFiniteInput);
    }
    if !is_affine_matrix(matrix) {
        return Err(AffineInverseError::NonAffineInput);
    }

    let determinant = matrix.determinant();
    if !determinant.is_finite() {
        return Err(AffineInverseError::NonFiniteDeterminant);
    }
    if determinant.abs() <= policy.minimum_matrix_determinant_absolute() {
        return Err(AffineInverseError::DeterminantTooSmall {
            minimum_absolute: policy.minimum_matrix_determinant_absolute(),
        });
    }

    let inverse = matrix.inverse();
    if !is_finite_mat4(inverse) {
        return Err(AffineInverseError::NonFiniteResult);
    }
    Ok(inverse)
}

fn is_affine_matrix(matrix: Mat4) -> bool {
    // This is the exact homogeneous-row definition of an affine column-vector matrix.
    matrix.x_axis.w == 0.0
        && matrix.y_axis.w == 0.0
        && matrix.z_axis.w == 0.0
        && matrix.w_axis.w == 1.0
}

/// 便利投影入口，仅把 aspect 和 near 限制到至少 0.001；调用者需保证 FOV、far 和结果符合相机契约。
/// 需要拒绝非有限或退化输入时使用 [`try_perspective`]。
pub fn perspective(fov_y_radians: Real, aspect_ratio: Real, z_near: Real, z_far: Real) -> Mat4 {
    Mat4::perspective_rh(
        fov_y_radians,
        aspect_ratio.max(0.001),
        z_near.max(0.001),
        z_far,
    )
}

/// 在输入来自可失败边界时构造常规透视矩阵：四个输入须有限，FOV 在 (0, PI) 内，aspect 和 near 为正且 far > near。
/// 成功结果有限；该入口使用有限远平面的近到远深度约定。
pub fn try_perspective(
    fov_y_radians: Real,
    aspect_ratio: Real,
    z_near: Real,
    z_far: Real,
) -> Result<Mat4, PerspectiveError> {
    if !fov_y_radians.is_finite()
        || !aspect_ratio.is_finite()
        || !z_near.is_finite()
        || !z_far.is_finite()
    {
        return Err(PerspectiveError::NonFiniteInput);
    }
    if !(0.0 < fov_y_radians && fov_y_radians < std::f32::consts::PI) {
        return Err(PerspectiveError::FieldOfViewOutOfRange);
    }
    if aspect_ratio <= 0.0 {
        return Err(PerspectiveError::AspectRatioNotPositive);
    }
    if z_near <= 0.0 {
        return Err(PerspectiveError::NearPlaneNotPositive);
    }
    if z_far <= z_near {
        return Err(PerspectiveError::FarPlaneNotAfterNear);
    }

    let matrix = Mat4::perspective_rh(fov_y_radians, aspect_ratio, z_near, z_far);
    is_finite_mat4(matrix)
        .then_some(matrix)
        .ok_or(PerspectiveError::NonFiniteMatrix)
}
