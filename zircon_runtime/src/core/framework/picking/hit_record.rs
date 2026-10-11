//! 定义把目标身份、命中数据和悬停策略绑定在一起的后端候选值。
//!
//! 拾取后端或编辑器适配层生成 [`HitRecord`]，再交给 [`super::PointerHits`]；
//! [`super::sorted_hits_for_pointer`] 合并排序，
//! [`super::hovered_hits_for_pointer`] 和 [`super::PickingHoverMap`] 消费遮挡与悬停策略。
//!
//! 记录拥有字段值，不借用或保活场景实体、相机和后端。构造仅绑定输入，
//! 身份有效性、几何空间及数据时效由生成方和使用方约定；
//! [`super::PickingEventState`] 可保留先前帧的命中数据。
use super::{HitData, HitTarget, Pickable};

#[derive(Clone, Debug, PartialEq)]
/// 将目标、空间命中和穿透策略绑定为一条候选；解析悬停时按策略决定是否继续看更低层目标。
///
/// # 生成与使用
/// 拾取后端或编辑器适配层把 [`HitTarget`] 与 [`HitData`] 组合为候选，再放入
/// [`super::PointerHits`]。统一排序使用目标类别、批次顺序和命中深度；
/// 悬停投影随后读取 [`Self::pickable`]，决定是否纳入当前记录和继续遍历下层记录。
///
/// # 构造与约束
/// [`Self::new`] 默认采用可悬停且阻挡下层的 [`Pickable`]，
/// [`Self::with_pickable`] 可原样替换该策略。公开字段也允许直接构造或修改；
/// 类型不检查目标是否存在、目标与几何是否来自同一来源，或命中数据是否仍适用于当前帧。
///
/// # 所有权与生命周期
/// 实例拥有身份、命中和策略值。克隆得到字段值的快照，不借用或保活实体、相机和后端；
/// 后续场景变化不会自动刷新它。事件状态可保留其中的命中数据供后续帧使用，
/// 操作实际对象前仍由使用方解析身份并判断时效。
///
/// # 参考
/// 参见 [`super::sorted_hits_for_pointer`]、[`super::hovered_hits_for_pointer`] 与 [`super::PickingEventState`]。
pub struct HitRecord {
    /// 这次候选的完整目标身份，供类别排序、悬停成员查询和事件状态索引使用。
    ///
    /// [`HitTarget`] 的变体、owner 和轴柄的轴名共同区分目标；同一 owner 的不同类别或轴
    /// 可以对应不同目标。值不保活实体，访问实际对象时由使用方在对应场景中解析并验证身份。
    pub target: HitTarget,
    /// 生成方为本候选提供的命中数据快照。
    ///
    /// 相机身份、深度单位以及可选位置和法线的空间约定见 [`HitData`]；
    /// 记录不会重新求交或验证这些数据。排序读取其深度，事件可克隆并保留先前帧的值，
    /// 因此收到记录不证明当前帧仍有相同几何命中。
    pub hit: HitData,
    /// 后续悬停投影采用的参与和遮挡策略，两个布尔量独立生效。
    ///
    /// [`Pickable::is_hoverable`] 决定是否纳入本记录，
    /// [`Pickable::should_block_lower`] 决定是否停止向下遍历；不可悬停记录也可以阻挡下层。
    /// [`Self::new`] 默认把两者设为 `true`。策略不会改变目标类别或命中深度的排序键。
    pub pickable: Pickable,
}

impl HitRecord {
    /// 绑定目标身份和生成方命中数据，创建采用默认悬停策略的候选记录。
    ///
    /// # 参数
    /// - `target`：与本候选对应的 [`HitTarget`]，按值保存；构造时不查询或验证实体。
    /// - `hit`：本次命中的 [`HitData`]，其所有权移入记录；深度、空间与身份约定由生成方提供。
    ///
    /// # 返回
    /// 返回拥有两个参数值的 [`Self`]，策略采用 [`Pickable::default`]，即可悬停并阻挡下层目标。
    /// 返回只证明记录已经构造，不执行拾取、排序、几何检查或场景对象验证。
    ///
    /// # 调用时机与生命周期
    /// 在后端或适配层已取得候选数据后使用，再交给 [`super::PointerHits`]。
    /// 结果不借用来源资源；保存或克隆不会保活实体，也不会刷新已保存的命中几何。
    ///
    /// # 参考
    /// 参见 [`Self::with_pickable`]、[`HitData`] 与 [`super::hovered_hits_for_pointer`]。
    pub fn new(target: HitTarget, hit: HitData) -> Self {
        Self {
            target,
            hit,
            pickable: Pickable::default(),
        }
    }

    /// 消费当前候选记录，替换其悬停和遮挡策略并返回更新后的值。
    ///
    /// # 参数
    /// - `self`：要更新的记录，所有权进入本次调用。
    /// - `pickable`：新的 [`Pickable`]，两个布尔字段原样保存，任何组合都可使用。
    ///
    /// # 返回
    /// 返回目标和命中数据保持原值、策略已替换的 [`Self`]，可用于链式构造。
    /// 此调用只更新本实例；已生成的克隆、批次和悬停快照不会被同步修改。
    ///
    /// # 调用时机与生命周期
    /// 生成候选时可用此方法表达忽略、穿透或不可悬停的阻挡策略。
    /// 实际纳入与截止由 [`super::hovered_hits_for_pointer`] 后续执行；
    /// 本次调用不重新求交或重排命中，结果仍拥有原字段值且不保活对象。
    ///
    /// # 参考
    /// 参见 [`Self::new`]、[`Self::pickable`] 与 [`Pickable`]。
    pub fn with_pickable(mut self, pickable: Pickable) -> Self {
        self.pickable = pickable;
        self
    }
}
