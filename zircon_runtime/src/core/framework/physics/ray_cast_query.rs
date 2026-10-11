//! 定义对指定同步世界发起射线查询的请求载荷。
//!
//! 调用者提供世界空间起点、方向、范围与目标筛选；本模块不读取场景或推进物理步。
use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;

use super::{PhysicsQueryFilter, PhysicsQueryMode};

/// 面向已同步世界的射线查询；调用方须提供有限非零方向和正距离，无效输入视为空命中。
///
/// # 使用时机
///
/// 场景已向物理服务提交同步状态后，用于视线遮挡或沿直线查找目标。
/// 当前 manager 读取最近写入的同步快照；构造请求不会自动同步刚修改的场景。
///
/// # 输入与结果边界
///
/// 当前 manager 要求起点、方向和最大距离有限，方向能够归一化、距离严格为正。
/// 方向不必预先为单位向量，服务归一化后以世界长度单位解释距离。
/// 无效输入、缺失同步世界和没有命中均可得到空集合，不能仅凭空集合证明路径安全或 provider 支持全部几何。
///
/// # 所有权与生命周期
///
/// 请求拥有筛选描述，世界句柄只是身份键，不延长世界存活；调用只借用本值。
/// 随后修改请求不会改变已经返回的命中载荷。
///
/// # 参考
///
/// [`WorldHandle`]、[`PhysicsQueryFilter`]、[`PhysicsQueryMode`]、[`super::PhysicsQueryInterface::ray_cast`]。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsRayCastQuery {
    /// 用于查找同步状态的世界身份键；当前 manager 找不到此键时返回空集合。
    /// 持有句柄不代表世界仍登记、实体仍有效或场景变更已同步。
    pub world: WorldHandle,
    /// 世界空间射线起点；当前 manager 要求各分量有限，不为调用者转换局部坐标。
    pub origin: [Real; 3],
    /// 世界空间射线方向；当前 manager 会归一化，允许非单位输入，但拒绝无法归一化的方向。
    /// 方向的长度不参与 `max_distance` 的单位换算。
    pub direction: [Real; 3],
    /// 沿归一化方向查找的最大世界长度；当前 manager 要求有限且严格大于零。
    /// 零值不在该射线入口中表达起点重叠检查。
    pub max_distance: Real,
    /// 控制命中数量与顺序；当前射线按距离比较，缺省反序列化采用 `Closest`。
    /// 使用 `First` 时不要假定返回的是最近目标。
    #[serde(default)]
    pub mode: PhysicsQueryMode,
    /// 这次查询拥有的目标筛选条件；默认策略排除传感器，自身排除须显式填写实体 ID。
    pub filter: PhysicsQueryFilter,
}
