//! 定义命中记录的悬停纳入与后续命中阻挡策略。
//!
//! [`Pickable`] 的两个布尔值相互独立，按值存入 [`super::HitRecord`] 或 [`super::PickingPrimitive`]。
//! 在按指针排序的命中列表中，先决定当前记录是否纳入悬停，再判断是否停止后续遍历。
//! 本模块只提供策略值；创建或改变值不生成命中、不排序、不重新投影已保存的快照。
//!
//! 报告保留原始和排序命中数，也记录首个阻挡目标；不可悬停计数遍历完整列表，包含被挡在后面的记录。
//! 调用方应依据各指标的含义区分原始候选、可悬停结果和阻挡目标。
//! 参见 [`Pickable::default`]、[`super::hovered_hits_for_pointer`] 与 [`super::PickingPipelineReport`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 命中后的遮挡与悬停策略彼此独立；不可悬停目标仍可阻断更低层目标。
///
/// 为一条命中独立指定是否纳入悬停，以及是否阻挡排序列表中的后续记录。
///
/// # 策略组合
///
/// 下表描述遍历尚未被更早记录阻挡、并已到达当前命中时的行为：
///
/// | 策略 | should_block_lower | is_hoverable | 当前命中 | 后续记录 |
/// | --- | --- | --- | --- | --- |
/// | [`Self::IGNORE`] | `false` | `false` | 不纳入悬停 | 继续处理 |
/// | [`Self::NON_BLOCKING`] | `false` | `true` | 纳入悬停 | 继续处理 |
/// | [`Self::BLOCKING_NON_HOVERABLE`] | `true` | `false` | 不纳入悬停 | 停止悬停遍历 |
/// | [`Self::default`] | `true` | `true` | 纳入悬停 | 停止悬停遍历 |
///
/// “后续”依统一排序列表解释：先目标优先级，再批次顺序、深度与来源索引；不单指几何距离更远的目标。
/// 策略不改变排序或删除原始候选，阻挡不要求当前记录可悬停。
///
/// # 生效时机与报告
///
/// 策略随命中交给 [`super::hovered_hits_for_pointer`] 或由 [`super::PickingHoverMap::from_outputs`] 投影时读取。
/// 球形后端仍先计算几何命中，再把图元策略复制到记录；`IGNORE` 不会单凭此值跳过几何求交。
/// [`super::PickingPipelineReport`] 的原始、排序命中数包含被忽略或被阻挡记录；`top_target` 仍取排序首项。
/// 报告首次遇到阻挡时保存 `blocking_target`，悬停计数截至该点且可包含阻挡记录，不可悬停计数仍扫描全列表。
/// 手工 [`super::PickingHoverMap::new`] 或 [`super::PickingHoverMap::set_pointer_hits`] 直接保存快照，调用方负责先完成预期裁剪。
///
/// # 所有权与生命周期
///
/// 结构只拥有两个布尔值，可复制、比较，不借用或保活目标、场景或后端。
/// 改变某个副本不更新已保存的命中、悬停或报告；要改变后续投影，需更新将被投影的记录或重新生成批次。
/// 公开字段允许任意四种组合，所有组合都有明确值语义，没有需校验的无效组合。
///
/// # 参考
///
/// 参见 [`Self::should_block_lower`]、[`Self::is_hoverable`]、[`super::HitRecord::with_pickable`]、[`super::PickingPrimitive::with_pickable`] 与 [`super::sorted_hits_for_pointer`]。
pub struct Pickable {
    /// 是否在处理当前记录之后停止纳入排序列表中的后续悬停命中。
    ///
    /// # 读取时机与约束
    ///
    /// 当前记录先按 [`Self::is_hoverable`] 决定是否纳入，再检查本字段；即使不可悬停，`true` 仍会阻挡后续项。
    /// “更低层”指按目标优先级、批次顺序、深度与来源索引排在后面的项，不只比较距离。
    /// 报告保存首个 `true` 记录的目标，且不据此删除原始或排序候选；完整列表的不可悬停计数仍继续。
    ///
    /// # 所有权与生命周期
    ///
    /// 字段是随策略复制的布尔值；修改已持有值只在后续投影读取该记录时生效，不刷新已生成快照。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::BLOCKING_NON_HOVERABLE`]、[`Self::default`]、[`super::hovered_hits_for_pointer`] 与 [`super::PickingPipelineReport`]。
    pub should_block_lower: bool,
    /// 是否允许当前已到达的命中记录进入悬停结果。
    ///
    /// # 读取时机与约束
    ///
    /// `true` 且未被更早记录阻挡时，当前命中纳入悬停；`false` 跳过当前记录，但不单独决定是否继续。
    /// 后续是否停止由 [`Self::should_block_lower`] 决定。字段不参与命中排序，也不阻止后端生成原始候选。
    /// 报告悬停计数只统计首个阻挡前及该点可悬停记录；不可悬停计数则统计完整排序列表中的所有 `false` 值。
    ///
    /// # 所有权与生命周期
    ///
    /// 布尔值由命中或图元按值拥有；改变副本不会更新先前形成的快照和报告，后续投影需读取更新后的记录。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::IGNORE`]、[`Self::NON_BLOCKING`]、[`super::PickingHoverMap::from_outputs`] 与 [`super::PickingPipelineReport`]。
    pub is_hoverable: bool,
}

impl Pickable {
    /// 不可悬停且不阻挡后续项的策略值，两字段均为 `false`。
    ///
    /// # 使用时机与结果
    ///
    /// 用 [`super::HitRecord::with_pickable`] 或 [`super::PickingPrimitive::with_pickable`] 将值存入将被投影的候选。
    /// 遍历到该记录时不纳入悬停，并继续后续排序项。原始、排序命中数仍包含它，报告不可悬停计数仍计入它；
    /// 若它位于排序首项，`top_target` 仍可指向该目标。此值不会令当前球形后端提前跳过几何求交。
    ///
    /// # 所有权与生命周期
    ///
    /// 常量提供可复制的两个布尔值，不持有目标。赋给记录后是独立值，不存在主动更新或资源取消行为。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::is_hoverable`]、[`Self::should_block_lower`]、[`super::hovered_hits_for_pointer`] 与 [`super::PickingPipelineReport`]。
    pub const IGNORE: Self = Self {
        should_block_lower: false,
        is_hoverable: false,
    };

    /// 可悬停且不阻挡后续项的策略值，阻挡为 `false`、可悬停为 `true`。
    ///
    /// # 使用时机与结果
    ///
    /// 存入命中或图元后，在未被更早记录阻挡且遍历到当前项时将其纳入悬停，并继续检查后续排序项。
    /// 因此同一指针可保留多个命中；该值不去重目标，也不改变排序键或保证当前项一定被遍历到。
    /// 报告可将它计入首个阻挡之前的悬停数，不计入不可悬停数。
    ///
    /// # 所有权与生命周期
    ///
    /// 常量是可复制标量值，记录拥有赋值副本；之后改变其他副本不会更新已有候选、快照或报告。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::is_hoverable`]、[`Self::should_block_lower`]、[`super::HitRecord::with_pickable`]、[`super::PickingPrimitive::with_pickable`] 与 [`super::hovered_hits_for_pointer`]。
    pub const NON_BLOCKING: Self = Self {
        should_block_lower: false,
        is_hoverable: true,
    };

    /// 不可悬停但仍阻挡后续项的策略值，阻挡为 `true`、可悬停为 `false`。
    ///
    /// # 使用时机与结果
    ///
    /// 遍历到当前候选时不将其纳入悬停，但随即停止后续悬停遍历；它不会被当作普通透明跳过项。
    /// 报告可保留该项为首个 `blocking_target`，并使该指针计入 `blocked_pointer_count`，同时当前项增加不可悬停数。
    /// 阻挡后其他不可悬停项仍计入报告完整列表的计数；原始、排序记录不会因该策略而被删除。
    ///
    /// # 所有权与生命周期
    ///
    /// 常量只提供可复制的布尔组合，需赋给后续投影读取的记录；自身不创建对象或保持目标存活。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::IGNORE`]、[`Self::should_block_lower`]、[`super::HitRecord::with_pickable`] 与 [`super::PickingPipelineReport`]。
    pub const BLOCKING_NON_HOVERABLE: Self = Self {
        should_block_lower: true,
        is_hoverable: false,
    };
}

impl Default for Pickable {
    /// 创建可悬停且阻挡后续排序项的默认策略，两字段均为 `true`。
    ///
    /// # 参数
    ///
    /// 无参数。
    ///
    /// # 返回
    ///
    /// 返回拥有两个 `true` 布尔值的 [`Self`]。投影到该记录时先纳入当前命中，再停止后续悬停遍历；
    /// 若更早记录已阻挡，则该记录不会进入悬停。没有独立错误结果。
    ///
    /// # 调用时机与约束
    ///
    /// [`super::HitRecord::new`] 和 [`super::PickingPrimitive::sphere`] 构造时采用此默认值；
    /// 可用各自 `with_pickable` 方法在后续投影前替换。默认构造只返回策略，不生成命中、验证目标或立即执行过滤。
    /// 报告可将已到达的默认记录计入悬停数并记录首个阻挡目标，但原始和排序命中列表仍保留后续记录。
    ///
    /// # 所有权与生命周期
    ///
    /// 返回值是可复制的自有布尔组合，无借用或资源所有权；多次构造彼此独立，丢弃副本不清理外部状态。
    ///
    /// # 参考
    ///
    /// 参见 `Default`、[`Self::should_block_lower`]、[`Self::is_hoverable`]、[`super::HitRecord::new`] 与 [`super::PickingPrimitive::sphere`]。
    fn default() -> Self {
        Self {
            should_block_lower: true,
            is_hoverable: true,
        }
    }
}
