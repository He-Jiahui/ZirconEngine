//! 定义运行时与编辑器共享的拾取目标身份及类别优先级。
//!
//! [`HitTarget`] 保存所属实体和交互类别，轴柄还保存 [`PickingAxis`]；
//! [`HitTarget::priority`] 投影出 [`PickingTargetPriority`]，供
//! [`super::sorted_hits_for_pointer`] 作为首个排序键。
//!
//! 这些枚举都是可复制的身份或分类值，不持有实体、手柄、几何或渲染资源。
//! 构造和访问器不查询场景，也不验证目标存活；需要操作实际对象时，
//! 由宿主在对应场景中解析身份并核对数据时效及操作约束。
use crate::core::framework::scene::EntityId;

/// 为轴柄目标保存 X、Y 或 Z 的轴名，区分同一实体的不同交互轴。
///
/// 轴名与 owner 一起组成 [`HitTarget::HandleAxis`] 的完整身份，
/// 编辑器路由据此往返转换轴选择。该枚举不携带方向向量、坐标空间或变换；
/// 实际轴几何与局部或世界空间约定由生成方提供。
///
/// # 值与比较
/// 无载荷值可以复制、哈希和保存；派生比较的顺序为 `X < Y < Z`。
/// 统一命中排序通过 [`HitTarget::priority`] 比较目标类别，轴名不成为其中的额外排序键。
///
/// # 参考
/// 参见 [`HitTarget::HandleAxis`] 与 [`HitTarget::handle_axis`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PickingAxis {
    /// X 轴的名称身份，随 [`HitTarget::HandleAxis`] 的 owner 一起区分轴柄。
    ///
    /// 具体方向向量和坐标空间由生成方的手柄几何约定，标签本身不携带这些数据。
    X,
    /// Y 轴的名称身份，随 [`HitTarget::HandleAxis`] 的 owner 一起区分轴柄。
    ///
    /// 具体方向向量和坐标空间由生成方的手柄几何约定，标签本身不携带这些数据。
    Y,
    /// Z 轴的名称身份，随 [`HitTarget::HandleAxis`] 的 owner 一起区分轴柄。
    ///
    /// 具体方向向量和坐标空间由生成方的手柄几何约定，标签本身不携带这些数据。
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// 运行时与编辑器共同使用的可选取身份；轴柄、场景 gizmo 和普通可渲染体按交互优先级排序。
///
/// # 身份与使用
/// 拾取后端和编辑器路由以这个值标识候选；悬停和事件状态用完整值进行成员判断和索引。
/// 变体、owner 和轴柄的 axis 都参与身份比较，因此同一 owner 的不同类别或轴可作为不同目标。
///
/// # 排序边界
/// [`Self::priority`] 只取得类别，统一命中排序再依次比较批次顺序、深度和来源索引。
/// 类型自身派生的 `Ord` 在同一变体内还比较字段；需要统一命中次序时，
/// 使用 [`super::sorted_hits_for_pointer`] 的完整规则。
///
/// # 所有权与有效性
/// 实例只有 [`EntityId`] 和可选轴名，能够按值复制；不借用或保活场景对象。
/// 三个构造函数原样保存标识，没有非零、对象存在性或可交互性检查。
/// 类型也没有世界域或代次字段；实际操作前由宿主在对应场景中解析并验证 owner，
/// 保存的身份不证明当前帧仍然命中该对象。
///
/// # 参考
/// 参见 [`Self::owner`]、[`Self::priority`]、[`super::PickingHoverMap`] 与 [`super::PickingEventState`]。
pub enum HitTarget {
    /// 某个实体的指定轴手柄身份，属于优先处理的轴柄类别。
    ///
    /// # 字段
    /// - `owner`：手柄关联的 [`EntityId`]，原样保存；访问实体时由宿主验证场景归属和有效性。
    /// - `axis`：[`PickingAxis`] 轴名，参与完整目标身份；具体几何空间、方向和交互模式由生成方提供。
    ///
    /// 值不持有手柄对象或几何。同一 owner 的不同轴是不同目标，
    /// [`Self::priority`] 则为它们返回相同的轴柄类别。
    HandleAxis { owner: EntityId, axis: PickingAxis },
    /// 某个实体的场景 gizmo 身份，类别顺序在轴柄之后、可渲染体之前。
    ///
    /// # 字段
    /// - `owner`：gizmo 关联的 [`EntityId`]，作为完整目标身份的一部分原样保存；
    ///   宿主在访问对象时验证其场景归属和有效性。
    ///
    /// 不同 gizmo 形状可以关联同一身份，形状及其命中数据由生成方另外提供；
    /// 这个值不保存子形状或保活 gizmo。类别投影见 [`Self::priority`]。
    SceneGizmo { owner: EntityId },
    /// 某个实体的可渲染体身份，在统一类别顺序中跟随轴柄与场景 gizmo。
    ///
    /// # 字段
    /// - `owner`：可渲染体关联的 [`EntityId`]，原样保存；
    ///   对实际实体或渲染资源的访问由宿主解析并验证。
    ///
    /// 此值没有网格、子对象或相交几何载荷，生成方可采用射线或屏幕空间近似提供候选。
    /// 保存类别不证明精确网格命中，也不会保活渲染资源；类别投影见 [`Self::priority`]。
    Renderable { owner: EntityId },
}

impl HitTarget {
    /// 创建指定实体和轴名的轴柄身份，供候选生成或编辑器路由转换使用。
    ///
    /// # 参数
    /// - `owner`：生成方关联的 [`EntityId`]，按值保存；任意数字都能构造，函数不验证实体存在性。
    /// - `axis`：[`PickingAxis`] 轴名，原样保存在身份中；几何方向和空间由生成方约定。
    ///
    /// # 返回
    /// 返回字段与参数逐值相同的 [`Self::HandleAxis`]；结果只表达候选身份，
    /// 不会生成命中几何、查询场景或完成对象操作。
    ///
    /// # 调用时机与生命周期
    /// 在已有所属实体标识时构造，可用于常量或运行时上下文。
    /// 返回值可复制且不借用或保活实体；后续操作对象前仍由宿主解析并验证身份与时效。
    ///
    /// # 参考
    /// 参见 [`Self::owner`] 与 [`Self::priority`]。
    pub const fn handle_axis(owner: EntityId, axis: PickingAxis) -> Self {
        Self::HandleAxis { owner, axis }
    }

    /// 创建指定实体的场景 gizmo 身份，供候选生成或编辑器路由转换使用。
    ///
    /// # 参数
    /// - `owner`：生成方关联的 [`EntityId`]，按值保存；任意数字都能构造，函数不验证实体存在性。
    ///
    /// # 返回
    /// 返回字段与参数逐值相同的 [`Self::SceneGizmo`]；结果只表达候选身份，
    /// 不会生成命中几何、查询场景或完成对象操作。
    ///
    /// # 调用时机与生命周期
    /// 在已有所属实体标识时构造，可用于常量或运行时上下文。
    /// 返回值可复制且不借用或保活实体；后续操作对象前仍由宿主解析并验证身份与时效。
    ///
    /// # 参考
    /// 参见 [`Self::owner`] 与 [`Self::priority`]。
    pub const fn scene_gizmo(owner: EntityId) -> Self {
        Self::SceneGizmo { owner }
    }

    /// 创建指定实体的可渲染体身份，供候选生成或编辑器路由转换使用。
    ///
    /// # 参数
    /// - `owner`：生成方关联的 [`EntityId`]，按值保存；任意数字都能构造，函数不验证实体存在性。
    ///
    /// # 返回
    /// 返回字段与参数逐值相同的 [`Self::Renderable`]；结果只表达候选身份，
    /// 不会生成命中几何、查询场景或完成对象操作。
    ///
    /// # 调用时机与生命周期
    /// 在已有所属实体标识时构造，可用于常量或运行时上下文。
    /// 返回值可复制且不借用或保活实体；后续操作对象前仍由宿主解析并验证身份与时效。
    ///
    /// # 参考
    /// 参见 [`Self::owner`] 与 [`Self::priority`]。
    pub const fn renderable(owner: EntityId) -> Self {
        Self::Renderable { owner }
    }

    /// 提取目标保存的所属实体标识，供宿主定位候选关联的实体。
    ///
    /// # 参数
    /// - `self`：要读取的目标身份，按值传入；[`Self`] 是 `Copy`，原身份仍可继续使用。
    ///
    /// # 返回
    /// 对三种变体都原样返回其 `owner` 字段，结果类型是 [`EntityId`]。
    /// 轴柄的轴名不影响结果；返回值没有经过当前场景查询、非零检查或存活验证。
    ///
    /// # 调用时机与生命周期
    /// 从候选或编辑器路由取得目标后，可用它提取数字身份。需要访问实际对象时，
    /// 调用方负责在对应场景中解析并验证标识。读取同步完成，结果不借用目标或保活实体。
    ///
    /// # 参考
    /// 参见 [`Self::HandleAxis`]、[`Self::SceneGizmo`] 与 [`Self::Renderable`]。
    pub const fn owner(self) -> EntityId {
        match self {
            Self::HandleAxis { owner, .. }
            | Self::SceneGizmo { owner }
            | Self::Renderable { owner } => owner,
        }
    }

    /// 取得目标的交互类别，供统一命中排序首先比较类别优先级。
    ///
    /// # 参数
    /// - `self`：按值读取的目标身份；owner 和轴名不参与类别投影。
    ///
    /// # 返回
    /// 分别把轴柄、场景 gizmo 和可渲染体映射为同名的 [`PickingTargetPriority`]。
    /// 同类别的不同 owner 或轴得到相同结果；类别比较顺序为轴柄、gizmo、可渲染体。
    ///
    /// # 调用时机与生命周期
    /// [`super::sorted_hits_for_pointer`] 使用该值作为首个键，再比较后端顺序、深度和来源索引。
    /// 此调用只同步读取变体，不对命中列表排序或验证目标存在性。
    /// 结果是可复制的分类值，不借用或保活目标资源。
    ///
    /// # 参考
    /// 参见 [`PickingTargetPriority::sort_key`] 与 [`Self::owner`]。
    pub const fn priority(self) -> PickingTargetPriority {
        match self {
            Self::HandleAxis { .. } => PickingTargetPriority::HandleAxis,
            Self::SceneGizmo { .. } => PickingTargetPriority::SceneGizmo,
            Self::Renderable { .. } => PickingTargetPriority::Renderable,
        }
    }
}

/// 表示命中排序首先比较的交互类别，供完整目标身份投影和编辑器类别常量使用。
///
/// # 比较与字段边界
/// [`HitTarget::priority`] 产生此值。派生的升序为 `HandleAxis < SceneGizmo < Renderable`，
/// [`Self::sort_key`] 同序映射为 `0`、`1`、`2`。
/// [`super::sorted_hits_for_pointer`] 在类别之后再处理后端顺序、命中深度与来源索引。
///
/// 标签没有 owner、轴名或几何数据，不能作为完整目标身份。它可复制和保存，
/// 不持有资源，也不证明任何实体当前存活或已经产生命中。
///
/// # 参考
/// 参见 [`HitTarget`]、[`HitTarget::priority`] 与 [`Self::sort_key`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PickingTargetPriority {
    /// 轴柄类别，统一类别顺序中的第一类。
    ///
    /// 由 [`HitTarget::HandleAxis`] 的 [`HitTarget::priority`] 产生，
    /// [`Self::sort_key`] 返回 `0`。标签只保存类别，不包含具体目标或其深度。
    HandleAxis,
    /// 场景 gizmo 类别，统一类别顺序在轴柄之后、可渲染体之前。
    ///
    /// 由 [`HitTarget::SceneGizmo`] 的 [`HitTarget::priority`] 产生，
    /// [`Self::sort_key`] 返回 `1`。标签只保存类别，不包含具体目标或其深度。
    SceneGizmo,
    /// 可渲染体类别，统一类别顺序在轴柄与场景 gizmo 之后。
    ///
    /// 由 [`HitTarget::Renderable`] 的 [`HitTarget::priority`] 产生，
    /// [`Self::sort_key`] 返回 `2`。标签只保存类别，不包含具体目标或其深度。
    Renderable,
}

impl PickingTargetPriority {
    /// 把交互类别转换为升序数值键，供类别常量和数值比较使用。
    ///
    /// # 参数
    /// - `self`：要转换的 [`Self`] 类别，按值读取且可复制。
    ///
    /// # 返回
    /// 返回 `u8`：轴柄为 `0`、场景 gizmo 为 `1`、可渲染体为 `2`，与本枚举派生的 `Ord` 同序。
    /// 结果只表示类别先后，不包含目标身份、后端顺序或命中深度，也不定义枚举的内存表示。
    ///
    /// # 调用时机与生命周期
    /// 可在常量或运行时上下文使用；编辑器以此初始化三种候选类别的数值常量。
    /// 调用同步返回固定键，不查询场景或产生后续工作，结果不借用任何目标或资源。
    ///
    /// # 参考
    /// 参见 [`HitTarget::priority`] 与 [`super::sorted_hits_for_pointer`]。
    pub const fn sort_key(self) -> u8 {
        match self {
            Self::HandleAxis => 0,
            Self::SceneGizmo => 1,
            Self::Renderable => 2,
        }
    }
}
