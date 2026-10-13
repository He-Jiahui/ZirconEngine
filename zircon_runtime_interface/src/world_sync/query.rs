//! Editor 通过网关请求 Runtime 持有的世界投影；传输层仅承载查询和结果 DTO。
//!
//! 本模块描述四类只读请求及其结果：反射组件行、层级行、单实体 Inspector 字段和局部
//! 变换快照。请求由 World::query_world 或动态会话入口交给 Runtime 执行；Editor 网关把
//! 查询编码为 JSON，并将返回值还原为 WorldQueryResult。这里不访问 ECS 存储或解析反射值。
//!
//! # 使用时机与生命周期
//!
//! 前三类查询可附带 generation_hint；只有缓存投影确实对应该代时才应提供提示。Runtime
//! 在代数相等且未饱和时可返回 NotModified，调用方须保留可复用的旧投影。变换快照不带
//! 代数提示，结果同时携带 World 替换代数，供事务调用方限定读取所属的运行目标。
//!
//! # 约束
//!
//! 请求和结果使用 kind 与 data 的 snake_case 标记，并拒绝未知字段。EntityId 只是 u64
//! 别名，不在传输层证明实体仍存在；缺失实体由 WorldQueryResult 显式表达。
//!
//! # 参考
//!
//! Runtime 查询构造见 zircon_runtime/src/scene/inspection/snapshot.rs；动态入口见
//! zircon_runtime/src/dynamic_api/session/world_sync.rs；Editor 调用者包括层级、Inspector
//! 和变换编辑。契约测试见 zircon_runtime_interface/src/tests/world_sync_contracts.rs。

use std::collections::BTreeMap;

use crate::math::Transform;
use crate::reflect::ReflectedValue;
use serde::{Deserialize, Serialize};

/// Stable runtime-world entity identity used by the transport contract.
///
/// Runtime 查询、层级行和变化事实共用的实体编号。该别名不携带 World 身份，也不负责
/// 查找或校验编号；接收方须结合对应的会话或世界上下文解释它。
pub type EntityId = u64;

/// One reflected component value requested from a matching entity.
///
/// # 使用时机与生命周期
///
/// 放入 ComponentWorldQuery 的 select 列表，选择要投影到 EntityRow 的反射组件。
/// type_name 使用反射注册中的完整类型路径；Runtime 仅投影可供 Editor 使用且当前实体
/// 实际持有的组件，未匹配的选择器不会产生组件项。
///
/// # 参数
///
/// new 的 type_name 被原样保存；构造器不解析或验证类型路径。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentSelector {
    /// 要选择的反射组件完整类型路径。
    pub type_name: String,
}

impl ComponentSelector {
    /// 创建一个尚未验证类型路径的组件选择器。
    ///
    /// # 参数
    ///
    /// type_name 可转换为 String，并应与 Runtime 反射注册的类型路径一致。
    ///
    /// # 返回值
    ///
    /// 返回可加入 ComponentWorldQuery.select 的选择器。
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
        }
    }
}

/// Presence constraints applied before selected component values are read.
///
/// Runtime 先在当前实体的 Editor 可见反射字段中匹配完整组件类型路径，再读取 select 中
/// 请求的字段。with 中每项都须匹配字段路径，without 中每项都不得匹配；任一空列表不增加
/// 限制。未知或不可见的 with 路径不能匹配；同类 without 路径不会排除实体。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryFilter {
    /// 实体必须持有的反射组件完整类型路径。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub with: Vec<String>,
    /// 实体不得持有的反射组件完整类型路径。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub without: Vec<String>,
}

/// Reflected-component projection against one runtime-world generation.
///
/// 请求返回所有满足 filter 的实体行，并将 select 指定的可见反射字段放入每行的
/// components 映射。select 为空时仍可返回匹配实体，但其 components 为空。generation_hint
/// 只表示调用方已有同代缓存，不会筛选实体。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentWorldQuery {
    /// 先于组件投影应用的存在与缺失条件。
    #[serde(default)]
    pub filter: QueryFilter,
    /// 按完整类型路径选择要投影的反射组件；未选类型不会出现在结果映射。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub select: Vec<ComponentSelector>,
    /// 调用方已持有的投影代数；仅精确匹配且代数未饱和时 Runtime 才可短路。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_hint: Option<u64>,
}

/// Runtime-owned hierarchy projection against one inspection generation.
///
/// 不带实体范围筛选；Runtime 返回当前检查快照中的层级行。generation_hint 应来自调用方
/// 当前保留的层级投影。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldHierarchyQuery {
    /// 调用方缓存的检查快照代数；相等时结果可为 NotModified。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_hint: Option<u64>,
}

/// Focused entity Inspector projection against one runtime-world generation.
///
/// 仅请求 entity 指定实体的 Editor 可见反射字段。实体不存在时返回 EntityMissing；匹配
/// generation_hint 时 Runtime 可先返回 NotModified，因此调用方应只传入自己可复用的缓存代数。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldInspectionFieldsQuery {
    /// 要检查的运行时实体编号。
    pub entity: EntityId,
    /// 调用方缓存的 Runtime 世界代数。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_hint: Option<u64>,
}

/// Exact local-transform read used to begin a world-qualified editor transaction.
///
/// Runtime 读取目标实体的局部 Transform；结果还携带查询时的世界替换代数。该请求不带
/// generation_hint，不能用普通层级投影的 NotModified 响应代替精确变换值。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldTransformSnapshotQuery {
    /// 要读取局部变换的实体编号。
    pub entity: EntityId,
}

/// Transport-neutral world query with no invalid filter/projection combinations.
///
/// 该枚举把互不兼容的请求形状限制在各自载荷中，序列化标签为 snake_case。Editor 网关
/// 将它交给 Runtime 查询入口；调用方依据对应结果变体更新缓存或继续编辑。
///
/// # 使用时机与生命周期
///
/// Components、Hierarchy 与 InspectionFields 可使用各自的 generation_hint。TransformSnapshot
/// 始终读取当前局部变换并返回世界替换代数。若同代缓存已不存在，不应发送其代数提示。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WorldQuery {
    /// 按反射组件条件筛选实体并投影所选组件字段。
    Components(ComponentWorldQuery),
    /// 读取 Runtime 检查快照的层级行。
    Hierarchy(WorldHierarchyQuery),
    /// 读取单个实体的 Editor 可见反射字段。
    InspectionFields(WorldInspectionFieldsQuery),
    /// 读取实体的精确局部变换和当前世界替换代数。
    TransformSnapshot(WorldTransformSnapshotQuery),
}

/// 默认查询保持组件查询形状，并使用空筛选、空选择和无代数提示。
impl Default for WorldQuery {
    fn default() -> Self {
        Self::Components(ComponentWorldQuery::default())
    }
}

impl WorldQuery {
    /// 构造层级查询，可用缓存代数请求 NotModified。
    ///
    /// # 参数
    ///
    /// generation_hint 是调用方保留的层级投影代数；None 表示请求完整层级结果。
    ///
    /// # 返回值
    ///
    /// 返回 WorldQuery::Hierarchy 请求。
    pub fn hierarchy(generation_hint: Option<u64>) -> Self {
        Self::Hierarchy(WorldHierarchyQuery { generation_hint })
    }

    /// 构造单实体 Inspector 字段查询。
    ///
    /// # 参数
    ///
    /// entity 是目标实体编号；generation_hint 是调用方已保留的 Inspector 投影代数。
    ///
    /// # 返回值
    ///
    /// 返回 WorldQuery::InspectionFields 请求。
    pub fn inspection_fields(entity: EntityId, generation_hint: Option<u64>) -> Self {
        Self::InspectionFields(WorldInspectionFieldsQuery {
            entity,
            generation_hint,
        })
    }

    /// 构造精确局部变换快照查询。
    ///
    /// # 参数
    ///
    /// entity 是要读取的实体编号。
    ///
    /// # 返回值
    ///
    /// 返回 WorldQuery::TransformSnapshot 请求；其结果不使用 generation_hint。
    pub fn transform_snapshot(entity: EntityId) -> Self {
        Self::TransformSnapshot(WorldTransformSnapshotQuery { entity })
    }

    /// 读取此请求携带的缓存代数提示。
    ///
    /// # 返回值
    ///
    /// Components、Hierarchy 和 InspectionFields 返回各自载荷的提示；TransformSnapshot
    /// 始终返回 None。
    pub fn generation_hint(&self) -> Option<u64> {
        match self {
            Self::Components(query) => query.generation_hint,
            Self::Hierarchy(query) => query.generation_hint,
            Self::InspectionFields(query) => query.generation_hint,
            Self::TransformSnapshot(_) => None,
        }
    }

    /// 为可缓存的查询替换 generation_hint。
    ///
    /// # 参数
    ///
    /// generation_hint 是调用方保留的结果代数，None 表示不请求代数短路。
    ///
    /// # 返回值
    ///
    /// 返回更新后的查询值。TransformSnapshot 不包含提示字段，因此该变体保持不变。
    pub fn with_generation_hint(mut self, generation_hint: Option<u64>) -> Self {
        match &mut self {
            Self::Components(query) => query.generation_hint = generation_hint,
            Self::Hierarchy(query) => query.generation_hint = generation_hint,
            Self::InspectionFields(query) => query.generation_hint = generation_hint,
            Self::TransformSnapshot(_) => {}
        }
        self
    }

    /// 给动态库 JSON 入口计算请求预算，返回的是请求项数而非结果行数。
    ///
    /// `zircon_runtime/src/dynamic_api/session/ffi.rs` 的 `query_world` 将此方法作为
    /// `bounded_json::decode` 的业务项计数回调：解码器反序列化成功后将返回值与 `max_items`
    /// 比较；超限时返回项数错误，由该入口映射为无效或超限载荷状态。此处统计请求项，不是结果行数。
    ///
    /// Components 按 with、without、select 的项数相加后再加一个固定项；其余三类查询
    /// 各计为一项。加法使用饱和运算，结果不取决于匹配实体数或返回字段数。
    ///
    pub fn request_item_count(&self) -> usize {
        match self {
            Self::Components(query) => query
                .filter
                .with
                .len()
                .saturating_add(query.filter.without.len())
                .saturating_add(query.select.len())
                .saturating_add(1),
            Self::Hierarchy(_) | Self::InspectionFields(_) | Self::TransformSnapshot(_) => 1,
        }
    }

    /// Applies the authoritative component-generation short-circuit and canonical row order.
    ///
    /// # 使用时机与生命周期
    ///
    /// Runtime 在完成组件筛选和字段读取后调用此构造器。generation 是本次读取使用的世界
    /// 代数；rows 是尚未排序的匹配行。
    ///
    /// # 返回值
    ///
    /// 若 generation 未饱和且与请求提示相等，返回不含行数据的 NotModified；否则按 entity
    /// 升序排列 rows 并返回 ComponentRows。
    ///
    /// # 约束
    ///
    /// self 必须是 Components 查询。该前提由 debug_assert 检查，调用者在发布构建中仍须
    /// 保证匹配。generation 为 u64::MAX 时不允许短路。
    pub fn component_result_for_generation(
        &self,
        generation: u64,
        mut rows: Vec<EntityRow>,
    ) -> WorldQueryResult {
        debug_assert!(matches!(self, Self::Components(_)));
        if generation != u64::MAX && self.generation_hint() == Some(generation) {
            WorldQueryResult::NotModified { generation }
        } else {
            rows.sort_by_key(|row| row.entity);
            WorldQueryResult::ComponentRows { generation, rows }
        }
    }

    /// Applies the authoritative hierarchy-generation short-circuit.
    ///
    /// # 使用时机与生命周期
    ///
    /// Runtime 在取得某一检查快照的层级行后调用；generation 应与 rows 属于同一快照。
    ///
    /// # 返回值
    ///
    /// 未饱和且提示与 generation 相等时返回 NotModified，否则原样保留 rows 的层级顺序并
    /// 返回 HierarchyRows。
    ///
    /// # 约束
    ///
    /// self 必须是 Hierarchy 查询；generation 为 u64::MAX 时不允许短路。
    pub fn hierarchy_result_for_generation(
        &self,
        generation: u64,
        rows: Vec<WorldHierarchyRow>,
    ) -> WorldQueryResult {
        debug_assert!(matches!(self, Self::Hierarchy(_)));
        if generation != u64::MAX && self.generation_hint() == Some(generation) {
            WorldQueryResult::NotModified { generation }
        } else {
            WorldQueryResult::HierarchyRows { generation, rows }
        }
    }

    /// Applies the focused Inspector short-circuit while preserving missing-entity state.
    ///
    /// # 使用时机与生命周期
    ///
/// Runtime 为请求实体取得同一检查快照的字段后调用。None 表示目标实体不存在；Some(empty)
/// 表示实体存在但当前没有可投影字段行。
    ///
    /// # 返回值
    ///
/// 未饱和且提示匹配时优先返回 NotModified；否则 Some(fields) 返回 InspectionFields，即使
/// fields 为空；None 返回 EntityMissing。字段向量由 Runtime 按组件显示名、字段显示名、
/// 类型路径和字段名排序。
    ///
    /// # 约束
    ///
    /// self 必须是 InspectionFields 查询，entity 必须与取字段时使用的请求实体相同；代数
    /// 为 u64::MAX 时不允许短路。
    pub fn inspection_fields_result_for_generation(
        &self,
        generation: u64,
        entity: EntityId,
        fields: Option<Vec<WorldInspectionFieldRow>>,
    ) -> WorldQueryResult {
        debug_assert!(matches!(self, Self::InspectionFields(_)));
        if generation != u64::MAX && self.generation_hint() == Some(generation) {
            WorldQueryResult::NotModified { generation }
        } else if let Some(fields) = fields {
            WorldQueryResult::InspectionFields {
                generation,
                entity,
                fields,
            }
        } else {
            WorldQueryResult::EntityMissing { generation, entity }
        }
    }

    /// 将当前读取到的局部变换封装为快照结果。
    ///
    /// # 使用时机与生命周期
    ///
    /// Runtime 查询入口在读取当前世界的局部变换后调用；world_replacement_epoch 标识读取时
    /// 所属的世界替换代数，调用方可据此拒绝跨替换复用的变换。
    ///
    /// # 参数
    ///
    /// generation 是读取使用的世界代数；entity 是本次请求的目标；transform 为 None 表示
    /// 该实体没有可读的局部变换。
    ///
    /// # 返回值
    ///
    /// Some(transform) 返回携带 generation 和 world_replacement_epoch 的 TransformSnapshot；
    /// None 返回 EntityMissing。
    ///
    /// # 约束
    ///
    /// self 必须是 TransformSnapshot 查询；缺失结果不携带世界替换代数。
    pub fn transform_snapshot_result(
        &self,
        generation: u64,
        world_replacement_epoch: u64,
        entity: EntityId,
        transform: Option<Transform>,
    ) -> WorldQueryResult {
        debug_assert!(matches!(self, Self::TransformSnapshot(_)));
        match transform {
            Some(transform) => WorldQueryResult::TransformSnapshot {
                generation,
                world_replacement_epoch,
                entity,
                transform,
            },
            None => WorldQueryResult::EntityMissing { generation, entity },
        }
    }
}

/// One entity and the deterministically ordered component values selected for it.
///
/// Runtime 组件查询按实体编号升序返回行；components 只包含 select 命中的可见反射组件，
/// 每个值为该组件所选字段的 JSON 对象。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityRow {
    /// 该行对应的 Runtime 实体编号。
    pub entity: EntityId,
    /// 以反射组件完整类型路径为键的字段值映射；空映射在 JSON 中省略。
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub components: BTreeMap<String, serde_json::Value>,
}

/// One runtime-owned hierarchy row shared by inspection and remote editor projections.
///
/// 层级构建器按前序遍历产生行，并在后序阶段计算子树摘要。Editor 的增量锚点会用 parent、
/// depth 和 subtree_hash 判断局部层级片段是否仍对应原结构。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldHierarchyRow {
    /// 该层级节点的实体编号。
    pub entity: EntityId,
    /// 父节点实体编号；根节点为 None。
    pub parent: Option<EntityId>,
    /// 从本次遍历根开始的层级深度，根节点为零。
    pub depth: u32,
    /// Runtime 场景节点当前的显示名称。
    pub display_name: String,
    /// Runtime 节点种类对应的显示标签。
    pub kind: String,
    /// 基于显示名称、直接子节点数和有序子树锚点计算的稳定摘要。
    pub subtree_hash: u64,
    /// Runtime 查询该节点时得到的层级激活状态。
    pub active_in_hierarchy: bool,
    /// 该节点在当前层级快照中是否至少有一个直接子节点。
    pub has_children: bool,
}

/// One editor-visible reflected field in a focused runtime Inspector projection.
///
/// Runtime 只为 Editor 可见且当前实体实际持有的反射组件生成此行。字段属性来自反射注册；
/// writable 是编辑能力标志，不代表 Editor 的其他保存或所有权策略也已满足。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldInspectionFieldRow {
    /// 反射组件的完整类型路径。
    pub component_type_path: String,
    /// 反射注册提供的组件显示名。
    pub component_display_name: String,
    /// 反射注册中的稳定字段名。
    pub field_name: String,
    /// 反射注册提供的字段显示名。
    pub field_display_name: String,
    /// 该字段值类型的完整反射路径。
    pub value_type_path: String,
    /// Runtime 读取到的反射值。
    pub value: ReflectedValue,
    /// 反射注册是否允许编辑该字段。
    pub writable: bool,
    /// 反射注册是否允许序列化该字段。
    pub serializable: bool,
    /// 该组件是否由插件注册所有。
    pub plugin_owned: bool,
}

/// Generation-qualified query response or an explicit same-generation short circuit.
///
/// NotModified 不携带可投影数据；Editor 只有在保留着相同 generation 的旧投影时才能复用
/// 缓存。序列化标签和字段形状由 world_sync_contracts.rs 固定。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WorldQueryResult {
    /// 返回按实体编号升序排列的组件投影。
    ComponentRows {
        /// 构造这些行时使用的世界代数。
        generation: u64,
        /// 满足筛选条件的实体行。
        rows: Vec<EntityRow>,
    },
    /// 返回 Runtime 检查快照的前序层级行。
    HierarchyRows {
        /// 层级快照代数。
        generation: u64,
        /// 按父子层级顺序排列的行。
        rows: Vec<WorldHierarchyRow>,
    },
    /// 返回目标实体的 Inspector 反射字段。
    InspectionFields {
        /// 生成字段投影时使用的检查代数。
        generation: u64,
        /// 字段所属的目标实体。
        entity: EntityId,
        /// Runtime 当前可见且已注册的字段行。
        fields: Vec<WorldInspectionFieldRow>,
    },
    /// 返回实体当前的精确局部变换。
    TransformSnapshot {
        /// 读取局部变换时的世界代数。
        generation: u64,
        /// 读取时所属 World 的替换代数。
        world_replacement_epoch: u64,
        /// 变换所属实体。
        entity: EntityId,
        /// 当前实体的局部变换值。
        transform: Transform,
    },
    /// 查询目标在该代数读取时不存在或没有可返回的目标值。
    EntityMissing {
        /// Runtime 判定缺失时使用的世界代数。
        generation: u64,
        /// 未找到的目标实体编号。
        entity: EntityId,
    },
    /// 请求提示与未饱和的当前代数相同，因此没有重复发送投影数据。
    NotModified {
        /// 与请求提示匹配的当前代数；调用方须保留该代的旧投影。
        generation: u64,
    },
}
