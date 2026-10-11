//! 定义射线、重叠和扫掠查询共用的目标筛选描述。
//!
//! 本模块只保存请求条件；当前 Physics 插件在遍历同步碰撞体前组合应用这些条件。
//! 筛选不修改碰撞体，也不建立实体或世界的存活保证。
use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;

/// 查询同步碰撞体时的筛选条件；默认排除传感器，实体排除集可避免感知射线命中自身。
///
/// # 使用时机
///
/// 作为查询请求的一部分传入物理服务；AI 视线查询在默认策略上显式排除观察者自身。
/// 当前 Physics 插件将各项条件取交集，允许传感器也不能绕过层、组和实体排除条件。
///
/// # 约束与默认策略
///
/// 层掩码按层号取位，碰撞组按数值相等筛选。默认只排除传感器，不自动排除调用者实体。
/// 这些是查询筛选条件；不等同于求解器中两个碰撞体之间的双向碰撞掩码或碰撞矩阵。
///
/// # 所有权与生命周期
///
/// 请求拥有实体排除列表；服务调用借用筛选器，排除 ID 的解释仍依赖所属查询的世界。
/// 构造或反序列化本类型不会解析实体、同步场景或验证 provider 的几何能力。
///
/// # 参考
///
/// [`Self::default`]、[`super::PhysicsRayCastQuery`]、[`super::PhysicsShapeOverlapQuery`]、
/// [`super::PhysicsShapeCastQuery`]。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicsQueryFilter {
    /// 允许命中的层位集合；当前共享实现以 `1_u32 << collider.layer` 与掩码相交。
    /// `None` 不添加层限制，`Some(0)` 拒绝所有层；它不代表目标碰撞体自己的碰撞掩码。
    pub collision_mask: Option<u32>,
    /// 是否允许传感器进入候选集合；`false` 排除传感器，`true` 仍需满足其余筛选条件。
    /// 此开关只影响这次查询，不修改目标的传感器属性或事件订阅。
    pub include_sensors: bool,
    /// 本次查询世界内须排除的实体 ID；命中实体属于此列表时被过滤，重复 ID 不改变成员语义。
    /// 默认空列表；反序列化缺少此字段也使用空列表，调用者需自行加入观察者或查询发起者。
    #[serde(default)]
    pub excluded_entities: Vec<EntityId>,
    /// 要求目标碰撞组与此值精确相等；当前实现不把组号解释为位集合。
    /// `None` 不限制组，缺省反序列化也取 `None`；`Some(0)` 仅接受组号为零的目标。
    #[serde(default)]
    pub required_collision_group: Option<u32>,
}

impl Default for PhysicsQueryFilter {
    /// 构造允许全部层和碰撞组、只排除传感器的通用查询策略。
    ///
    /// # 返回
    ///
    /// 返回独立拥有的筛选器，实体排除列表为空；结果不代表已验证的世界或可调用的 provider。
    ///
    /// # 使用时机与生命周期
    ///
    /// 适合用结构更新语法构建一次请求，再覆盖业务所需的层、组或自身排除条件。
    /// AI 视线查询显式补充观察者 ID，避免把默认策略误当作自动排除自身。
    /// 本函数不访问服务或场景；返回后调用者可独立修改该值。
    ///
    /// # 参考
    ///
    /// [`PhysicsQueryFilter`]、[`super::PhysicsRayCastQuery`]。
    fn default() -> Self {
        Self {
            collision_mask: None,
            include_sensors: false,
            excluded_entities: Vec::new(),
            required_collision_group: None,
        }
    }
}
