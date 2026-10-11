//! 定义在指定同步世界内检查形状重叠的请求载荷。
//!
//! 调用者提供世界空间姿态与筛选策略，服务生成目标元数据副本；本模块不修改实体或求解接触。
use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;
use crate::core::math::Transform;

use super::{PhysicsColliderShape, PhysicsQueryFilter, PhysicsQueryMode};

/// 检查世界空间形状与已同步碰撞体的重叠；与射线查询共用筛选条件。
///
/// # 使用时机
///
/// 用于区域占用、候选目标收集等一次查询，在场景向物理服务提交同步状态后调用。
/// 当前 manager 读取同步快照；请求本身不会刷新刚修改的场景。
///
/// # 输入与结果约束
///
/// 当前共享 helper 要求有效形状与有限变换；资源或复合几何的支持范围由实现决定。
/// `Closest` 与 `All` 使用目标中心到查询平移位置的平方距离度量；
/// 结果表达目标重叠描述，不表达穿透深度、表面最近距离或唯一实体集合。
/// 无效输入、缺失同步状态、无命中或不支持的几何均可得到空集合。
///
/// # 所有权与生命周期
///
/// 请求拥有形状与筛选器；世界句柄和形状资源引用不保持世界存活或证明资源已加载。
/// 查询只借用本值，返回的目标描述独立于请求及之后的同步状态。
///
/// # 参考
///
/// [`PhysicsColliderShape`]、[`Transform`]、[`PhysicsQueryMode`]、[`super::PhysicsShapeOverlapHit`]。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsShapeOverlapQuery {
    /// 待查询同步快照的世界身份键；当前 manager 没有对应同步状态时返回空集合。
    /// 身份键本身不保证世界存活或快照包含刚修改的场景。
    pub world: WorldHandle,
    /// 要检查重叠的几何描述，尺寸按 `transform` 的缩放解释；它不是创建碰撞体的请求。
    /// 当前共享 helper 的支持与有效性判断决定能否生成候选。
    pub shape: PhysicsColliderShape,
    /// 查询形状的世界空间姿态，分量须满足当前实现的有效性检查。
    /// 其平移位置也是当前最近与全部排序的距离参考点；共享近似不保证完整旋转几何。
    pub transform: Transform,
    /// 控制结果数量和顺序，缺省反序列化采用 `Closest`；最近度量为目标中心距离。
    /// 需要全部目标时显式选择 `All`，不能将默认结果当作完整重叠集合。
    #[serde(default)]
    pub mode: PhysicsQueryMode,
    /// 这次查询拥有的目标筛选描述；当前共享实现与射线和扫掠使用同一组筛选条件。
    pub filter: PhysicsQueryFilter,
}
