//! 定义重叠查询返回的目标碰撞体元数据副本。
//!
//! 载荷保留几何、姿态与筛选属性，不提供穿透深度、接触点或持续的实体访问权限。
use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;
use crate::core::math::Transform;

use super::PhysicsColliderShape;

/// 重叠查询返回的目标碰撞体快照，供调用方继续判断传感器和碰撞分组。
///
/// # 使用时机与内容
///
/// 在形状重叠查询后读取目标身份、几何及筛选元数据，供业务继续区分传感器、层或碰撞组。
/// 当前共享 helper 从已同步碰撞体复制这些字段，没有附带接触点、法线或穿透深度。
///
/// # 所有权与生命周期
///
/// 结果拥有形状描述并复制变换与属性，不借用请求或同步表；资源引用也不代表资源已解析。
/// 数据只反映查询所读快照，后续场景修改、同步或实体移除不会更新已返回的载荷。
/// 继续操作实体前应在原查询世界确认当前身份与状态。
///
/// # 参考
///
/// [`PhysicsColliderShape`]、[`super::PhysicsShapeOverlapQuery`]、[`super::PhysicsQueryInterface::shape_overlap`]。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsShapeOverlapHit {
    /// 产生这项重叠结果的目标实体 ID；必须结合原查询世界解释，载荷不延长实体生命周期。
    pub entity: EntityId,
    /// 同步目标的几何描述副本；当前共享 helper 克隆目标形状，不返回已经加载的后端形状对象。
    pub shape: PhysicsColliderShape,
    /// 目标碰撞体在所读快照中的世界空间姿态；不是查询形状的姿态。
    /// 当前重叠的最近模式比较此平移位置与查询平移位置，不比较表面距离。
    pub transform: Transform,
    /// 该同步目标是否是传感器；默认筛选排除传感器，允许传感器时此字段仍保留原属性。
    pub sensor: bool,
    /// 目标的层号，区别于层位集合；查询层掩码以此编号取位后匹配。
    /// 它是快照元数据，不授予对碰撞层配置的修改能力。
    pub layer: u32,
    /// 目标的碰撞组号；当前查询筛选通过数值相等判断指定组，不把该数值展开为位掩码。
    pub collision_group: u32,
}
