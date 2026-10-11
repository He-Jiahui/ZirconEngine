//! 定义形状扫掠返回的拥有型命中载荷，承载目标身份与本实现报告的路径几何。
//!
//! 载荷不移动实体，也不保证位置和法线代表真实形状表面的接触几何。
use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;
use crate::core::math::Real;

/// 描述形状扫掠中的一次目标命中，供调用者比较路径距离或检查起始重叠。
///
/// # 几何与约束
///
/// 当前共享实现对起始重叠返回零距离、查询 AABB 中心和零法线；
/// 其他命中来自膨胀目标 AABB 上的射线求交，其位置表示移动查询中心的路径位置。
/// 因此本载荷不统一承诺目标表面接触点、真实接触法线或连续精确几何求解。
/// 公开字段和反序列化不执行几何或有限值验证。
///
/// # 所有权与生命周期
///
/// 结果拥有实体 ID 和数值副本，不借用请求或目标，也不会追随后续物理同步。
/// 保存载荷不会使目标实体或 provider 保持存活。
///
/// # 参考
///
/// [`super::PhysicsShapeCastQuery`]、[`super::PhysicsQueryInterface::shape_cast`]。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsShapeCastHit {
    /// 命中的目标实体 ID，需结合产生结果的查询世界使用；旧载荷不能证明该实体仍存在。
    pub entity: EntityId,
    /// 沿归一化扫掠方向报告的世界长度；当前共享实现以零表示起始重叠。
    /// 数值不携带时间或碰撞后运动状态，不能直接当作求解器的步进结果。
    pub distance: Real,
    /// 当前共享实现报告的查询 AABB 中心路径位置；起始重叠时为该起始中心。
    /// 它不保证位于目标或扫掠形状的真实表面，消费者不能直接将其作为接触点。
    pub position: [Real; 3],
    /// 当前膨胀 AABB 求交报告的法线；起始重叠路径固定为零向量。
    /// 需要真实表面接触方向的调用者须另行取得适用的几何结果。
    pub normal: [Real; 3],
}
