//! 定义射线查询的拥有型命中载荷，供插件消费者读取实体与世界空间几何结果。
//!
//! 本模块不验证手工构造的数据，也不拥有碰撞体、实体或 provider 的生命周期。
use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;
use crate::core::math::Real;

/// 描述射线查询返回的一次几何命中，供遮挡判断等调用者比较距离和读取命中位置。
///
/// # 字段与约束
///
/// 当前 manager 将请求方向归一化，`distance` 因而使用世界坐标的长度单位，
/// `position` 与 `normal` 也在查询使用的世界空间表达。内部求交可能使用近似几何；
/// 本类型不保证命中来自原生后端，也不在手工构造或反序列化时验证距离与法线。
///
/// # 所有权与生命周期
///
/// 字段是复制的身份与数值，不借用查询或碰撞体；返回后仍可保存载荷，
/// 但实体可能已被移除或移动，不能将旧载荷当作当前世界状态。
///
/// # 参考
///
/// [`super::PhysicsRayCastQuery`]、[`super::PhysicsQueryInterface::ray_cast`]。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsRayCastHit {
    /// 命中的目标实体 ID，需结合产生该载荷的查询世界解释；它不持有实体或保证之后仍存在。
    pub entity: EntityId,
    /// 沿归一化射线从请求起点到所报告位置的距离，使用世界长度单位。
    /// 当前 manager 的有效命中在请求最大距离内；公开字段本身不强制有限值或非负值。
    pub distance: Real,
    /// 所报告的世界空间射线位置；当前共享 helper 按起点加归一化方向乘距离构造。
    /// 当查询几何为近似体时，此值也只是该近似体的求交结果。
    pub position: [Real; 3],
    /// 求交 helper 报告的世界空间几何法线；退化数值情形可能留下零向量。
    /// 需依赖单位法线的消费者应另行检查，不可仅凭此数组类型认定它已归一化。
    pub normal: [Real; 3],
}
