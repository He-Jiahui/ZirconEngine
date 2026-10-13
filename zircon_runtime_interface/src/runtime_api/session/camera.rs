//! 编辑器模拟视口的相机覆盖数据，按 JSON 放入会话事件负载。
//! 宿主在同步事件调用期间保留编码字节；运行时解码后保存相机值，后续只覆盖渲染提取结果。
//! 参数准入由 `validate_editor_camera` 完成，本模块不负责改变场景中的相机节点。

use serde::{Deserialize, Serialize};

use crate::math::Transform;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/// 编辑器模拟视口相机的 JSON 事件载体。
/// 构造器只封装字段；运行时收到事件后再验证版本、变换与投影参数。
///
/// # 使用时机与生命周期
/// Editor 的 `route_simulate_camera` 编码此值，经相机事件送入当前 Play gateway；
/// `RuntimeCameraController::apply_editor_camera` 接纳后持有值副本，不借用编码缓冲区。
///
/// # 约束
/// 当前接纳入口要求 V1 版本、有限变换、非退化缩放与归一化旋转，
/// 并按投影种类检查视场角或正交尺寸及近远裁剪面。构造与反序列化不会替代这些检查。
///
/// # 参考
/// [`crate::runtime_api::ZrRuntimeEventV1::viewport_camera`]、
/// `RuntimeDynamicSession::handle_viewport_camera`、`validate_editor_camera`。
pub struct ZrRuntimeViewportCameraV1 {
    /// 相机负载版本，由运行时相机验证入口检查。
    pub abi_version: u32,
    /// 渲染相机的位姿与缩放；接纳前检查有限性、缩放退化和旋转归一化。
    pub transform: Transform,
    /// 透视或正交投影的协议种类；未知值在接纳入口返回错误。
    pub projection_kind: u32,
    /// 垂直视场角，单位为弧度；透视分支要求位于零与 π 之间。
    pub fov_y_radians: f32,
    /// 正交投影尺寸；正交分支要求为正，透视分支仍检查其有限性。
    pub ortho_size: f32,
    /// 正的近裁剪距离；接纳入口同时检查它小于远裁剪距离。
    pub z_near: f32,
    /// 有限的远裁剪距离；必须严格大于近裁剪距离。
    pub z_far: f32,
}

impl ZrRuntimeViewportCameraV1 {
    /// 封装一个待编码、待运行时验证的相机描述值。
    ///
    /// # 参数
    /// - `abi_version`：相机负载版本。
    /// - `transform`：编辑器视口相机的变换。
    /// - `projection_kind`：透视或正交投影种类。
    /// - `fov_y_radians`、`ortho_size`：两种投影各自的几何参数。
    /// - `z_near`、`z_far`：近远裁剪距离。
    ///
    /// # 返回值
    /// 原样保存参数的值，不执行验证、编码或会话调用。
    ///
    /// # 参考
    /// `EditorHostEventController::simulate_preview_camera`、
    /// `RuntimeCameraController::apply_editor_camera` 与 [`Self`] 的接纳约束。
    pub const fn new(
        abi_version: u32,
        transform: Transform,
        projection_kind: u32,
        fov_y_radians: f32,
        ortho_size: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        Self {
            abi_version,
            transform,
            projection_kind,
            fov_y_radians,
            ortho_size,
            z_near,
            z_far,
        }
    }
}
