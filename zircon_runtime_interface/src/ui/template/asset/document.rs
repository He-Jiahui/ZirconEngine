//! 描述 UI 资产的树形源文档、组件与样式声明，以及加载、验证阶段共享的错误类型。
//! Runtime loader 在迁移后验证这些数据，编译器再把节点展开为模板；编辑器也读取该模型。
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::Value;

use super::{
    UiBindingMissingValuePolicy, UiComponentApiVersion, UiComponentPublicContract, UiResourceRef,
};
use crate::ui::accessibility::UiAccessibilityContract;
use crate::ui::focus::UiFocusContract;
use crate::ui::layout::UiSlotKind;
use crate::ui::navigation::UiNavigationContract;
use crate::ui::picking::UiPickPolicy;
use crate::ui::template::UiBindingRef;
use crate::ui::widget::UiWidgetContract;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// 导入清单用粗资产类别区分布局、组件和样式；加载器会在展开前核对引用类别。
/// 布局可实例化根树，Widget 提供可导入组件，Style 提供可导入样式规则。
pub enum UiAssetKind {
    /// 可直接编译为界面的资产。
    Layout,
    /// 导出组件定义供其他资产引用的资产。
    Widget,
    /// 只提供样式表的资产。
    Style,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// TOML 资产的身份和版本元数据；导入校验用类别与 id 确认引用目标。
pub struct UiAssetHeader {
    pub kind: UiAssetKind,
    /// 注册表和导入声明使用的稳定资产标识。
    pub id: String,
    #[serde(default = "default_asset_version")]
    /// 源 schema 版本；缺省时由 `default_asset_version` 采用初始版本。
    pub version: u32,
    #[serde(default)]
    pub display_name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 声明编译要解析的组件、样式与资源依赖；实际句柄解析留给后续运行时层。
pub struct UiAssetImports {
    #[serde(default)]
    /// 按资产 id 引用的 Widget 组件文档。
    pub widgets: Vec<String>,
    #[serde(default)]
    /// 按资产 id 引用的 Style 文档。
    pub styles: Vec<String>,
    #[serde(default)]
    /// 供资源收集阶段解析的显式资源引用。
    pub resources: Vec<UiResourceRef>,
}

/// 树形 TOML 源文档；加载器先执行版本迁移和权属校验，再交由编译器展开。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 从 TOML 反序列化得到的源模型；构造或反序列化本身不保证树、导入及组件契约合法。
/// Runtime asset loader 先执行 schema migration 与文档验证，再由编译器消费；跨资产引用的句柄解析属于后续 Runtime 装配阶段。
pub struct UiAssetDocument {
    pub asset: UiAssetHeader,
    #[serde(default)]
    pub imports: UiAssetImports,
    #[serde(default)]
    pub tokens: BTreeMap<String, Value>,
    #[serde(default)]
    pub root: Option<UiNodeDefinition>,
    #[serde(default)]
    pub components: BTreeMap<String, UiComponentDefinition>,
    #[serde(default)]
    pub stylesheets: Vec<UiStyleSheet>,
}

impl UiAssetDocument {
    /// 返回文档根节点标识；没有 root 时返回 None，结果借用自该文档。
    pub fn root_node_id(&self) -> Option<&str> {
        self.root.as_ref().map(|node| node.node_id.as_str())
    }

    /// 通过 `components` 表中的键取得组件根节点标识；未找到时返回 None。
    ///
    /// # 返回值
    /// 返回借用自文档的节点 id。
    pub fn component_root_node_id(&self, component_name: &str) -> Option<&str> {
        self.components
            .get(component_name)
            .map(|component| component.root.node_id.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// 决定源节点由本地 Widget、组件展开、外部引用还是命名槽解释。
pub enum UiNodeDefinitionKind {
    #[default]
    /// 由 Widget 描述符直接构造的节点。
    Native,
    /// 在当前或导入组件定义中查找并展开的节点。
    Component,
    /// 通过组件引用语法解析到其他资产的节点。
    Reference,
    /// 在组件实例化时由调用方提供子内容填充的槽节点。
    Slot,
}

/// 源节点保留组件引用、挂载槽和绑定声明，展开后才形成运行时模板节点。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 源树中的单个节点声明，`kind` 与可选组件字段共同决定其迁移和展开方式。
/// TOML/公开字段允许暂时不完整的组合；loader 的迁移与验证阶段检查节点归属和组件引用，之后的实例化器才生成运行时节点。
pub struct UiNodeDefinition {
    #[serde(default)]
    pub node_id: String,
    #[serde(default)]
    pub kind: UiNodeDefinitionKind,
    #[serde(default, rename = "type")]
    pub widget_type: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    /// 跨资产组件引用，格式由 [`parse_component_reference`] 校验。
    pub component_ref: Option<String>,
    #[serde(default)]
    /// 调用方要求的组件公开 API 版本，用于契约兼容性检查。
    pub component_api_version: Option<UiComponentApiVersion>,
    #[serde(default)]
    pub slot_name: Option<String>,
    #[serde(default)]
    pub control_id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
    #[serde(default)]
    pub props: BTreeMap<String, Value>,
    #[serde(default)]
    pub layout: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    pub bindings: Vec<UiBindingRef>,
    #[serde(default)]
    pub style_overrides: UiStyleDeclarationBlock,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<UiFocusContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation: Option<UiNavigationContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub picking: Option<UiPickPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a11y: Option<UiAccessibilityContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget: Option<UiWidgetContract>,
    #[serde(default)]
    pub children: Vec<UiChildMount>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 父调用点对展开后子根施加的布局/挂载属性，不表示子组件的命名槽参数声明。
/// `component_instance_expander::apply_child_mount` 与 `prototype_instancer::apply_prototype_child_mount` 会先按父上下文解析 `slot`，再规范化布局并合并到每个子根的 `slot_attributes`；映射覆盖会递归合并表、替换标量/数组。
pub struct UiChildMount {
    #[serde(default)]
    /// 非空时仅在解析后的 `slot` 缺少 `mui_slot` 键时提供默认挂载名。
    pub mount: Option<String>,
    #[serde(default)]
    /// 子根的挂载属性值；表达式在父调用点的 token/参数上下文解析，布局表随后按规范化规则处理。
    pub slot: BTreeMap<String, Value>,
    pub node: UiNodeDefinition,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 本地组件导出声明；`root` 是后续实例展开入口，参数、槽和公开契约在组件验证时检查。
/// 该结构只保存源定义，不表示组件已通过导入解析或可直接实例化；Runtime prototype store 注册后才供实例化器查找。
pub struct UiComponentDefinition {
    pub root: UiNodeDefinition,
    #[serde(default)]
    pub style_scope: UiStyleScope,
    #[serde(default)]
    pub contract: UiComponentPublicContract,
    #[serde(default)]
    pub params: BTreeMap<String, UiComponentParamSchema>,
    #[serde(default)]
    pub slots: BTreeMap<String, UiNamedSlotSchema>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 组件参数声明记录值种类和可选默认值，实例展开时据此解析参数输入。
/// 默认值保留为 TOML 数据；组件参数解析器在实例展开时应用默认值并验证输入类型，反序列化不会执行表达式求值。
pub struct UiComponentParamSchema {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub default: Option<Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 组件对调用方直接子挂载声明的槽契约；验证阶段据此检查必需性、基数、子组件类别与布局族。
/// 空 `accepts` 不限制子组件身份；仅声明 schema 不会验证调用方传入内容。
pub struct UiNamedSlotSchema {
    #[serde(default)]
    /// 调用方必须至少提供一个子挂载才能满足槽契约。
    pub required: bool,
    #[serde(default)]
    /// 槽是否接收多个直接子项。
    pub multiple: bool,
    /// The child-layout family expected by this named slot when the component owns placement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<UiSlotKind>,
    /// Accepted direct child component identities. An empty set keeps the slot unconstrained.
    #[serde(default)]
    pub accepts: BTreeSet<String>,
}

impl UiNamedSlotSchema {
    /// 空 accepts 集合保留不限制子组件的旧式槽；非空时只接受显式列出的直接组件身份。
    pub fn accepts_component(&self, component: &str) -> bool {
        self.accepts.is_empty() || self.accepts.contains(component)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// TODO: [CR-R02-public_ui_templates-0001] 确认是否应驱动私有样式隔离；当前定义和扁平原型仅保存此值，选择器校验实际按公开部件表拒绝私有目标。
/// 证据缺口是尚无 Open/Closed 跨组件 selector 行为测试；下一步分别断言外部选择器访问私有部件的结果，确认隔离边界。
pub enum UiStyleScope {
    Open,
    #[default]
    Closed,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiStyleSheet {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub rules: Vec<UiStyleRule>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 对匹配节点应用一组样式值的源规则；优先级由解析后的选择器计算。
pub struct UiStyleRule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// 由 [`UiSelector`](crate::ui::template::UiSelector) 解析的选择器文本。
    pub selector: String,
    #[serde(default)]
    pub set: UiStyleDeclarationBlock,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// self 与 slot 分开保存声明值，展开器据两类位置分别应用于节点和槽内容。
pub struct UiStyleDeclarationBlock {
    #[serde(default, rename = "self")]
    pub self_values: BTreeMap<String, Value>,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
}

/// 源资产中的动作路由声明；事件分发端负责解析负载缺失策略。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiActionRef {
    #[serde(default)]
    pub route: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub payload: BTreeMap<String, Value>,
    #[serde(
        default,
        skip_serializing_if = "UiBindingMissingValuePolicy::is_required"
    )]
    /// 动态负载缺失时的处理约定；默认 Required 时从序列化文档省略。
    pub payload_missing_policy: UiBindingMissingValuePolicy,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
/// 源文档读取、schema 迁移、组件/槽验证和选择器解析阶段共用的失败类型。
pub enum UiAssetError {
    #[error("failed to parse ui asset document: {0}")]
    /// TOML 文本无法反序列化为资产模型。
    ParseToml(String),
    #[error("failed to read ui asset document: {0}")]
    /// 文件加载阶段无法读取源字节。
    Io(String),
    #[error(
        "ui asset {asset_id} uses unsupported schema version {version}; current supported version is {current}"
    )]
    /// 资产版本不在当前迁移器支持范围内。
    UnsupportedSchemaVersion {
        asset_id: String,
        version: u32,
        current: u32,
    },
    #[error("ui asset {asset_id} schema migration failed: {detail}")]
    /// 已识别的旧版本无法迁移到当前源模型。
    SchemaMigrationFailed { asset_id: String, detail: String },
    #[error("ui asset {asset_id} is invalid: {detail}")]
    InvalidDocument { asset_id: String, detail: String },
    #[error("ui asset {asset_id} references missing node {node_id}")]
    MissingNode { asset_id: String, node_id: String },
    #[error("ui asset {asset_id} references unknown component {component}")]
    UnknownComponent { asset_id: String, component: String },
    #[error("ui asset reference {reference} is not registered")]
    UnknownImport { reference: String },
    #[error("ui asset reference {reference} expected kind {expected:?} but received {actual:?}")]
    /// 已解析导入的类别与引用位置要求不一致。
    ImportKindMismatch {
        reference: String,
        expected: UiAssetKind,
        actual: UiAssetKind,
    },
    #[error("ui component {component} missing required slot {slot_name}")]
    /// 组件实例遗漏 schema 标记为必需的槽。
    MissingRequiredSlot {
        component: String,
        slot_name: String,
    },
    #[error("ui component {component} received unknown slot {slot_name}")]
    /// 组件实例传入了组件 schema 未声明的槽名。
    UnknownSlot {
        component: String,
        slot_name: String,
    },
    #[error("ui component {component} slot {slot_name} does not accept multiple children")]
    /// 同一单值槽收到多个子挂载。
    SlotDoesNotAcceptMultiple {
        component: String,
        slot_name: String,
    },
    #[error("ui component {component} slot {slot_name} does not accept child {child_component}")]
    /// 子挂载的直接组件身份不在槽许可集合中。
    SlotDoesNotAcceptComponent {
        component: String,
        slot_name: String,
        child_component: String,
    },
    #[error("ui selector is invalid: {0}")]
    /// 选择器文本不符合当前受支持的解析子集。
    InvalidSelector(String),
}

/// Parses an imported component reference in its only supported form: `asset#Component`.
/// 解析跨资产组件引用，只接受恰好一个非空 `#Component` 后缀。
///
/// # 参数
/// `reference` 是导入标识与组件名组成的借用字符串。
///
/// # 返回值
/// 返回分别借用自输入的资产 id 和组件名；格式错误时返回 [`UiAssetError::InvalidDocument`]。
///
/// # 参考
/// Runtime `prototype_store::get_component` 和组件实例展开器用此结果解析已注册原型。
pub fn parse_component_reference(reference: &str) -> Result<(&str, &str), UiAssetError> {
    let Some((asset_id, component)) = reference.split_once('#') else {
        return Err(UiAssetError::InvalidDocument {
            asset_id: reference.to_string(),
            detail: "component references must include a #Component suffix".to_string(),
        });
    };
    if asset_id.is_empty() || component.is_empty() || component.contains('#') {
        return Err(UiAssetError::InvalidDocument {
            asset_id: reference.to_string(),
            detail: "component references must contain exactly one non-empty #Component suffix"
                .to_string(),
        });
    }
    Ok((asset_id, component))
}

/// serde 缺省源版本，与当前初始 TOML schema 约定一致。
const fn default_asset_version() -> u32 {
    1
}
