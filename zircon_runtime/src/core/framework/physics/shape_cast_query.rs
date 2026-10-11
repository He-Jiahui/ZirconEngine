//! 定义从世界空间起始姿态沿方向扫掠形状的请求。
//!
//! 请求只表达查询意图，不创建刚体、不提交运动命令，也不拥有世界或 provider。
use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;
use crate::core::math::{Real, Transform};

use super::{PhysicsColliderShape, PhysicsQueryFilter, PhysicsQueryMode};

/// 以世界空间起始姿态扫掠形状；退化方向、距离或形状在查询服务中返回空命中。
///
/// # 使用时机
///
/// 在已有物理同步状态上检查形状移动路径，结果由 [`super::PhysicsQueryInterface::shape_cast`] 返回。
/// 当前 manager 使用共享快照几何 helper；查询本身不会移动场景对象或调用求解器步进。
///
/// # 输入与能力约束
///
/// 当前实现要求变换与方向有限、形状有效、方向能够归一化，以及有限的非负最大距离。
/// 零距离可用于检查起始重叠，仍须提供有效方向；负距离才直接被该距离检查拒绝。
/// 当前共享实现先判断起始重叠，再以查询 AABB 半尺寸膨胀目标 AABB 执行射线查询；
/// 不支持的资源或复合几何会给出空结果，不能据此推断精确形状之间没有碰撞。
///
/// # 所有权与生命周期
///
/// 请求拥有几何描述和筛选器，资源引用不代表资源已加载；世界句柄不保证世界存活或快照新鲜。
/// 调用只借用请求，返回载荷独立保存查询时的数据，不会随本值修改而更新。
///
/// # 参考
///
/// [`PhysicsColliderShape`]、[`Transform`]、[`PhysicsQueryMode`]、[`super::PhysicsShapeCastHit`]。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsShapeCastQuery {
    /// 待查询同步世界的身份键；当前 manager 缺少此世界快照时返回空集合，不自动建立同步状态。
    pub world: WorldHandle,
    /// 扫掠形状的拥有型几何描述，尺寸先按 `origin_transform` 的缩放解释。
    /// 当前共享 helper 的几何支持范围决定能否查询，构造描述本身不验证或加载资源。
    pub shape: PhysicsColliderShape,
    /// 形状在世界空间的起始姿态；当前入口要求分量有限，但共享 AABB 近似不承诺完整旋转几何。
    /// 它不是待写回实体的运动命令，也不包含自动的局部到世界转换。
    pub origin_transform: Transform,
    /// 世界空间扫掠方向，当前共享 helper 将其归一化；即使最大距离为零也须能归一化。
    /// 方向长度不缩放最大距离。
    pub direction: [Real; 3],
    /// 沿归一化方向扫掠的最大世界长度；当前入口接受有限非负值。
    /// 零值可保留起始重叠检查，负值或非有限值给出空结果。
    pub max_distance: Real,
    /// 控制结果数量与顺序；当前实现按扫掠命中距离比较，缺省反序列化使用 `Closest`。
    /// 起始重叠的零距离可参与最近候选选择。
    #[serde(default)]
    pub mode: PhysicsQueryMode,
    /// 本次请求的目标筛选条件；只筛选目标碰撞体，不修改请求形状或目标的物理属性。
    pub filter: PhysicsQueryFilter,
}
