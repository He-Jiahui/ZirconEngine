//! 保留节点的导航创作配置：索引准入、分组、方向目标与边界意图。
//!
//! 这些值拥有字符串和节点标识，不查树、不建立导航索引，也不执行焦点转移。
//! 序列化默认值与显式构造的差异属于公共契约；具体导航消费边界保留在既有待确认项中。

use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

/// 导航分组的自有字符串标识，用于按值比较、排序和查找。
/// 创建该标识不校验空串或分组是否存在，生命周期不依赖外部字符串。
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiNavigationGroupId(pub String);

impl UiNavigationGroupId {
    /// 接收可转换为自有字符串的分组名并原样保存；返回标识，不注册或校验分组。
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// 显式 Tab 顺序及准入；缺省反序列化的 `tabbable` 为 false，`new` 才启用准入。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTabIndex {
    pub order: i32,
    pub tabbable: bool,
}

impl UiTabIndex {
    /// 创建指定顺序且允许 Tab 准入的索引；参数仅用于候选排序。
    pub const fn new(order: i32) -> Self {
        Self {
            order,
            tabbable: true,
        }
    }

    /// 创建关闭 Tab 准入的索引；保留零顺序值，不代表默认候选。
    pub const fn disabled() -> Self {
        Self {
            order: 0,
            tabbable: false,
        }
    }
}

/// 为导航索引声明分组、模态根及排序；焦点候选资格仍由节点状态决定。
///
/// 父组与根节点均为可选引用；模态位、循环位和顺序由索引消费者解释。
/// 该记录拥有标识而不借用目标，构造或反序列化不会验证目标存在或分组关系无环。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiNavigationGroup {
    pub group_id: UiNavigationGroupId,
    pub parent: Option<UiNavigationGroupId>,
    pub root: Option<UiNodeId>,
    pub modal: bool,
    pub wrap: bool,
    pub order: i32,
}

/// 分别保存四个方向的目标声明，未提供的方向使用自动目标。
/// 它只保存策略值，不检查目标有效性，也不自行查找下一个节点。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiDirectionalNavigation {
    pub up: UiDirectionalNavigationTarget,
    pub down: UiDirectionalNavigationTarget,
    pub left: UiDirectionalNavigationTarget,
    pub right: UiDirectionalNavigationTarget,
}

// TODO: [CR-UINAV-0001] 确认 boundary 的运行时消费边界；当前只见序列化和测试读取，导航索引按分组与模态状态选目标；下一步补各边界值的派发契约测试。
/// 声明导航边界处的意图；具体执行需由运行时导航路径解释。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "target")]
pub enum UiNavigationBoundary {
    #[default]
    Escape,
    Wrap,
    Stop,
    Explicit(UiNodeId),
    Trap,
}

/// 单个方向的目标选择：自动、指定节点、指定分组或阻止该方向。
/// 序列化以种类标签和可选目标负载表示；载荷标识不保证对应节点或分组仍然存在。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "target")]
pub enum UiDirectionalNavigationTarget {
    #[default]
    Auto,
    Node(UiNodeId),
    Group(UiNavigationGroupId),
    Blocked,
}

/// 聚合一个节点的可选 Tab 索引、分组、方向声明和边界意图。
///
/// # 默认值与约束
/// 缺失的可选配置保持为空，缺失边界使用逃逸；显式索引的反序列化准入规则见其类型说明。
/// 配置拥有全部数据，不借用保留树；创建该值不会执行边界策略或改变焦点。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiNavigationContract {
    pub tab_index: Option<UiTabIndex>,
    pub group: Option<UiNavigationGroup>,
    pub directional: Option<UiDirectionalNavigation>,
    #[serde(default)]
    pub boundary: UiNavigationBoundary,
}
