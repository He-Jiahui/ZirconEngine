//! 定义插件间物理查询的类型化桥接契约和接口身份。
//!
//! Physics 插件导出服务，AI 等消费者在注册阶段导入调用句柄。
//! 本模块不负责桥接绑定、provider 启停或世界同步，查询载荷也不授予持续的调用许可。
use crate::core::framework::bridge::PluginInterface;

use super::{
    PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsShapeCastHit, PhysicsShapeCastQuery,
    PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery,
};

/// 物理查询服务用于桥接注册、导入和插件 manifest 对齐的稳定接口 ID。
///
/// # 使用边界
///
/// 该 ID 通过 [`PluginInterface::INTERFACE_ID`] 关联 [`PhysicsQueryInterface`]，
/// 不编码世界身份、provider 当前状态或原生插件 ABI 的版本与入口签名。
///
/// # 参考
///
/// [`PhysicsQueryInterface`]、[`PluginInterface`]。
pub const PHYSICS_QUERY_INTERFACE_ID: &str = "physics.query.v1";

/// Runtime-owned query surface for plugin-to-plugin physics calls.
/// 运行时托管的插件间物理查询接口；AI 等消费者经桥接调用，提供者可随插件生命周期撤销。
///
/// # 调用者与实现者的职责
///
/// 调用者提供世界、几何和筛选条件；实现者在自己的查询状态上同步返回拥有的命中载荷。
/// 桥接导出的实现需满足 [`PluginInterface`] 的线程与所有权契约，不能借用 ECS 世界；
/// 当前 Physics 插件通过 manager 保存的同步快照查询，而非在此接口中直接遍历可变场景。
///
/// # 使用时机与生命周期
///
/// 消费者在插件注册阶段导入句柄，最终绑定后通过 [`crate::plugin::BridgeImport::call`] 调用。
/// provider 被停用或撤销时，桥接入口可拒绝后续调用；已取得 provider 的在途回调仍可完成。
/// 直接持有实现的 Rust 引用不经过该桥接准入检查，不能把直接调用当作 owner 当前启用的证明。
///
/// # 结果与能力边界
///
/// 三个方法返回 `Vec`，自身没有单独的“不支持”错误。当前 manager 对缺失同步世界、
/// 被拒绝的输入或没有几何命中均可返回空集合；桥接查找或准入错误由外层 `call` 返回。
/// 当前 manager 使用共享快照几何 helper，其结果不能证明已调用所选 Jolt 后端的原生查询。
///
/// # 参考
///
/// [`PHYSICS_QUERY_INTERFACE_ID`]、[`super::PhysicsManager`]、[`Self::ray_cast`]、
/// [`Self::shape_overlap`]、[`Self::shape_cast`]。
pub trait PhysicsQueryInterface: Send + Sync {
    /// 在指定世界的查询状态上取得射线命中，供视线遮挡等消费者判断沿途目标。
    ///
    /// # 参数
    ///
    /// - `query`：本次借用的世界空间请求；当前 manager 要求起点、方向和距离有限，
    ///   方向能够归一化且距离为正，再应用请求的筛选器与结果模式。
    ///
    /// # 返回与错误
    ///
    /// 返回独立拥有的命中集合，数量和顺序依请求模式；空集合不区分无命中、无同步快照或无效输入。
    /// 此方法不返回桥接错误；通过桥接调用时须同时处理外层 `call` 的准入结果。
    ///
    /// # 调用时机与生命周期
    ///
    /// 当前 manager 读取已同步快照，本调用不推进物理步或自动同步刚修改的场景。
    /// 返回载荷不借用 `query`，其实体 ID 与几何只描述本次查询时点。
    ///
    /// # 参考
    ///
    /// [`PhysicsRayCastQuery`]、[`PhysicsRayCastHit`]、[`crate::plugin::BridgeImport::call`]。
    fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Vec<PhysicsRayCastHit>;
    /// 查询指定世界内与给定世界空间形状重叠的目标，并返回目标碰撞体的元数据快照。
    ///
    /// # 参数
    ///
    /// - `query`：借用的形状、姿态、世界和筛选条件；当前共享 helper 要求几何与变换有效，
    ///   对支持的形状执行重叠判断，`Closest` 使用目标中心到查询平移位置的距离度量。
    ///
    /// # 返回与错误
    ///
    /// 返回拥有的目标描述，数量和顺序依查询模式；没有穿透深度、接触点或自动去重的实体集合保证。
    /// 空集合可表示无重叠、无同步世界或当前实现无法处理输入，桥接错误仍在外层 `call`。
    ///
    /// # 调用时机与生命周期
    ///
    /// 当前 manager 使用最近写入的同步快照；结果中的形状与姿态是复制值，不会追随后续同步。
    /// 若要继续操作目标实体，调用者须在所属世界另行确认它仍可使用。
    ///
    /// # 参考
    ///
    /// [`PhysicsShapeOverlapQuery`]、[`PhysicsShapeOverlapHit`]、[`super::PhysicsQueryMode`]。
    fn shape_overlap(&self, query: &PhysicsShapeOverlapQuery) -> Vec<PhysicsShapeOverlapHit>;
    /// 从给定姿态沿世界空间方向扫掠形状，供调用者检测移动路径上的目标。
    ///
    /// # 参数
    ///
    /// - `query`：借用的起始形状、姿态和扫掠请求；当前 manager 要求有限非负距离、
    ///   有效几何和可归一化方向，零距离仍须提供有效方向，才能检查起始重叠。
    ///
    /// # 返回与错误
    ///
    /// 返回拥有的命中载荷，起始重叠可产生零距离命中；空集合不单独表达不支持或无效输入。
    /// 当前共享实现先检查起始重叠，再对膨胀 AABB 执行射线扫掠；
    /// 因此命中位置和法线不能统一解释为真实形状之间的表面接触点与接触法线。
    ///
    /// # 调用时机与生命周期
    ///
    /// 此查询不移动场景对象或推进求解；当前 manager 读取同步快照，结果不随之后的世界更新改变。
    /// 通过桥接调用仍须处理外层 provider 准入错误。
    ///
    /// # 参考
    ///
    /// [`PhysicsShapeCastQuery`]、[`PhysicsShapeCastHit`]、[`super::PhysicsQueryMode`]。
    fn shape_cast(&self, query: &PhysicsShapeCastQuery) -> Vec<PhysicsShapeCastHit>;
}

impl PluginInterface for dyn PhysicsQueryInterface {
    /// 将物理查询 trait 对象关联到唯一的桥接 ID，供类型化导出和导入使用同一身份。
    ///
    /// # 使用边界
    ///
    /// 这里只提供静态身份；是否已经绑定、当前 provider 是否可调用以及世界是否同步由其他入口判断。
    ///
    /// # 参考
    ///
    /// [`PHYSICS_QUERY_INTERFACE_ID`]、[`PluginInterface`]。
    const INTERFACE_ID: &'static str = PHYSICS_QUERY_INTERFACE_ID;
}
