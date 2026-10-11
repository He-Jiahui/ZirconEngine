//! 提供拾取后端使用的世界空间射线值与视口像素投影。
//!
//! [`ray_from_viewport_point`] 按本次相机快照和视口尺寸计算 [`PointerRay`]；
//! [`crate::core::framework::picking::RayMap`] 在指针与相机视口配对后调用它。
//! 射线拥有数值，不保留相机借用；后续帧需要根据新的位置与快照重新构造。
use crate::core::framework::render::{ProjectionMode, ViewportCameraSnapshot};
use crate::core::math::{Real, UVec2, Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
/// 后端共用的世界空间射线；通过 new 创建可保证方向单位化并拒绝非有限输入。
///
/// # 使用约束与生命周期
///
/// `origin` 与 `direction` 采用相机变换所在的世界空间。两个字段均公开；通过字面量组装
/// 或改写字段时，调用方负责维持有限原点与单位、非零、有限方向。受检构造见 [`Self::new`]。
/// [`crate::core::framework::picking::PrimitivePickingBackend`] 的球求交采用单位方向前提。
///
/// 值可复制，副本拥有独立数值；复制、比较和丢弃不会更新相机或执行外部状态清理。
pub struct PointerRay {
    /// 射线在世界空间中的起点，使用相机变换的长度单位并随射线按值复制。
    ///
    /// 透视射线从相机平移位置出发；正交射线在相机平面内偏移。
    /// [`Self::new`] 检查分量有限，直接赋值时由调用方维持该约束。
    pub origin: Vec3,
    /// 世界空间的前进方向；后端使用时应为单位、非零且各分量有限。
    ///
    /// [`Self::new`] 生成浮点归一化结果。公开字段允许原样赋值，调用方须在改写后
    /// 继续维持方向约束；复制该向量不重新验证或关联相机状态。
    pub direction: Vec3,
}

impl PointerRay {
    /// 归一化输入方向并构造通过有限性与非零检查的射线。
    ///
    /// # 参数
    ///
    /// - `origin`：世界空间起点，按值传入，各分量须有限；本函数不换算坐标或长度单位。
    /// - `direction`：世界空间朝向，按值传入；方向长度由归一化处理，不要求预先为单位向量。
    ///
    /// # 返回值
    ///
    /// 原点有限且归一化后的方向有限、平方长度大于零时返回 `Some`，否则返回 `None`。
    /// 零向量、非有限方向以及导致浮点范数下溢或上溢的某些有限方向都可能被拒绝。
    ///
    /// # 调用时机与生命周期
    ///
    /// 在向后端交付射线前使用；结果拥有原点与归一化方向，不保留外部借用。
    /// 该检查只针对这两个数值结果，相机参数和视口配对仍由调用方管理。
    ///
    /// # 参考
    ///
    /// 相机与局部像素位置生成射线的入口见 [`ray_from_viewport_point`]。
    pub fn new(origin: Vec3, direction: Vec3) -> Option<Self> {
        let direction = direction.normalize_or_zero();
        (origin.is_finite() && direction.is_finite() && direction.length_squared() > 0.0)
            .then_some(Self { origin, direction })
    }
}

/// 把视口局部像素点投影到相机射线；实际视口尺寸决定宽高比，越界或退化相机返回 None。
///
/// # 参数
///
/// - `camera`：本次计算借用的 [`ViewportCameraSnapshot`]，提供投影模式、变换与对应投影参数。
/// - `viewport_size`：与位置配对的视口宽高，单位为像素；任一维度为零时返回 `None`。
/// - `position`：该视口的局部像素点，左上为原点，x 向右、y 向下。
///   有效范围为 `[0, width] × [0, height]`，比较边界使用转为 `Real` 的尺寸。
///
/// # 返回值
///
/// 满足下述旋转前提时，返回拥有世界空间数值的射线；越界、非有限位置或派生射线未通过 [`PointerRay::new`]
/// 的检查时返回 `None`。NaN 不会触发外层越界比较，会在后续派生值检查中被拒绝。
///
/// # 坐标与准入约束
///
/// 像素转为横向 `[-1, 1]`、纵向 `[1, -1]` 的标准化坐标；
/// 宽高比取本次 `viewport_size`，投影分支对用于计算的宽高比采用 `0.001` 下限。
/// 透视分支要求半个竖直视场角的正切有限且为正；正交分支对半高采用 `0.001` 下限。
/// 相机变换缩放、快照保存的宽高比、投影覆盖矩阵、近远裁面与活动标记不参与这次计算。
/// 调用方负责有意义的相机参数以及位置、视口尺寸与快照之间的配对。
///
/// # 调用时机与生命周期
///
/// 在拾取后端查询前，根据当前输入与相机快照计算。函数只在本次调用借用相机，
/// 返回值不随之后的相机变化自动更新；再次采样须重新调用。
///
/// # 参考
///
/// 两种投影分类见 [`ProjectionMode`]。
///
/// # 旋转前提
///
/// 相机旋转须满足单位四元数前提，见 [`crate::core::math::Quat::mul_vec3`]。
/// 启用 `glam-assert`，或在调试断言与 `debug-glam-assert` 同时启用时，
/// 无效旋转可能在乘向量时触发 panic，发生在射线结果检查之前。
pub fn ray_from_viewport_point(
    camera: &ViewportCameraSnapshot,
    viewport_size: UVec2,
    position: Vec2,
) -> Option<PointerRay> {
    if viewport_size.x == 0 || viewport_size.y == 0 {
        return None;
    }
    if position.x < 0.0
        || position.y < 0.0
        || position.x > viewport_size.x as Real
        || position.y > viewport_size.y as Real
    {
        return None;
    }

    let ndc_x = position.x / viewport_size.x as Real * 2.0 - 1.0;
    let ndc_y = 1.0 - position.y / viewport_size.y as Real * 2.0;
    let aspect_ratio = viewport_size.x.max(1) as Real / viewport_size.y.max(1) as Real;
    match camera.projection_mode {
        ProjectionMode::Perspective => perspective_ray(camera, aspect_ratio, ndc_x, ndc_y),
        ProjectionMode::Orthographic => orthographic_ray(camera, aspect_ratio, ndc_x, ndc_y),
    }
}

/// 为透视投影计算从相机平移位置出发的世界空间射线。
///
/// # 参数
///
/// - `camera`：借用本次相机快照，读取变换及以弧度表示的竖直视场角。
/// - `aspect_ratio`：外层根据非零视口宽高生成的比例，横向计算使用 `max(0.001)`。
/// - `ndc_x`、`ndc_y`：外层转换的标准化坐标，横向右为正、纵向上为正。
///
/// # 返回值
///
/// 半角正切非有限或不为正时返回 `None`；否则旋转局部
/// `(ndc_x * aspect * tan, ndc_y * tan, -1)` 并交给 [`PointerRay::new`]，返回其结果。
/// 此处不限定视场角的完整取值区间，也不独立验证旋转四元数。
///
/// # 调用时机与生命周期
///
/// 由 [`ray_from_viewport_point`] 的透视分支调用；位置换算与视口准入已在外层处理。
/// 只借用快照进行数值计算，结果拥有字段；变换缩放和近远裁面不参与构造。
///
/// # 旋转前提
///
/// 相机旋转须满足单位四元数前提，见 [`crate::core::math::Quat::mul_vec3`]。
/// 启用 `glam-assert`，或在调试断言与 `debug-glam-assert` 同时启用时，
/// 无效旋转可能在乘向量时触发 panic，发生在射线结果检查之前。
///
/// # 参考
///
/// 调用入口见 [`ray_from_viewport_point`]，派生值统一交给 [`PointerRay::new`]。
fn perspective_ray(
    camera: &ViewportCameraSnapshot,
    aspect_ratio: Real,
    ndc_x: Real,
    ndc_y: Real,
) -> Option<PointerRay> {
    let half_fov_tan = (camera.fov_y_radians * 0.5).tan();
    if !half_fov_tan.is_finite() || half_fov_tan <= 0.0 {
        return None;
    }
    let local_direction = Vec3::new(
        ndc_x * aspect_ratio.max(0.001) * half_fov_tan,
        ndc_y * half_fov_tan,
        -1.0,
    );
    PointerRay::new(
        camera.transform.translation,
        camera.transform.rotation * local_direction,
    )
}

/// 在正交相机平面内偏移起点，并生成与相机前向平行的射线。
///
/// # 参数
///
/// - `camera`：借用本次快照，读取变换及作为半高使用的 `ortho_size`。
/// - `aspect_ratio`：外层计算的宽高比，半宽计算使用 `max(0.001)`。
/// - `ndc_x`、`ndc_y`：外层转换的标准化坐标，分别确定右向与上向偏移。
///
/// # 返回值
///
/// 原点为相机平移加上 `right * (ndc_x * half_width)` 与 `up * (ndc_y * half_height)`，
/// 方向取相机 `forward`；两者交由 [`PointerRay::new`] 检查，失败返回 `None`。
/// 半高使用 `ortho_size.max(0.001)`：零、负数、负无穷或 NaN 会采用下限；
/// 其它非有限或退化派生值仍按最终射线检查处理。
///
/// # 调用时机与生命周期
///
/// 由 [`ray_from_viewport_point`] 的正交分支调用。右、上、前向由旋转后的基向量
/// 归一化取得；变换缩放与近裁面不参与起点计算。返回值不保留快照借用。
///
/// # 旋转前提
///
/// 相机旋转须满足单位四元数前提，见 [`crate::core::math::Quat::mul_vec3`]。
/// 启用 `glam-assert`，或在调试断言与 `debug-glam-assert` 同时启用时，
/// 无效旋转可能在乘向量时触发 panic，发生在射线结果检查之前。
///
/// # 参考
///
/// 相机像素入口见 [`ray_from_viewport_point`]，最终数值准入见 [`PointerRay::new`]。
fn orthographic_ray(
    camera: &ViewportCameraSnapshot,
    aspect_ratio: Real,
    ndc_x: Real,
    ndc_y: Real,
) -> Option<PointerRay> {
    let half_height = camera.ortho_size.max(0.001);
    let half_width = half_height * aspect_ratio.max(0.001);
    let origin = camera.transform.translation
        + camera.transform.right() * (ndc_x * half_width)
        + camera.transform.up() * (ndc_y * half_height);
    PointerRay::new(origin, camera.transform.forward())
}
