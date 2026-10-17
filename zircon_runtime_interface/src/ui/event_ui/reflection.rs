//! UI 控制视图与保留树诊断快照的拥有数据。
//!
//! 控制视图描述可查询属性和动作；诊断快照另外保存布局、输入及属性来源。
//! 构造器和链式方法仅组装数据，树一致性、权限检查、失效调度和实际写入由消费方负责。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ui::binding::UiEventKind;
use crate::ui::component::{UiValue, UiValueKind};
use crate::ui::layout::{UiFrame, UiPoint};
use crate::ui::tree::UiVisibility;
use crate::ui::tree::{UiDirtyFlags, UiInputPolicy};

use super::UiRouteId;

/// 生成原样保存数值的节点身份包装；节点注册与查找由树或管理器承担。
macro_rules! define_id {
    ($name:ident) => {
        /// 可比较、排序和序列化的节点编号；默认零值不代表已注册节点。
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
        )]
        pub struct $name(pub u64);

        impl $name {
            /// 保存任意 `value` 并返回编号，不验证节点存在或所属树。
            pub const fn new(value: u64) -> Self {
                Self(value)
            }
        }
    };
}

define_id!(UiNodeId);

/// 拥有树身份文本；比较、排序和哈希沿用原始字符串，不做规范化或树查找。
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiTreeId(pub String);

impl UiTreeId {
    /// 将 `value` 转成拥有的字符串并返回身份；空值和命名约束由使用方决定。
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// 拥有节点定位文本；不解析文件路径，也不保证对应节点当前存在。
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiNodePath(pub String);

impl UiNodePath {
    /// 将 `value` 转成拥有的字符串并原样保存，返回值不借用输入。
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UiNodePath {
    /// 直接写出保存的文本，将写入失败交还格式化器。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// 控制面 JSON 值或参数的类型描述；此枚举不自行检查实际值与描述是否一致。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiValueType {
    Any,
    String,
    Unsigned,
    Signed,
    Float,
    Bool,
    Null,
    Array,
    Object,
}

/// 八个独立的节点状态位，默认均关闭。
///
/// 与显式可见性、输入策略及反射快照中的焦点状态分别保存，改变一项不会自动同步其他项。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStateFlags {
    pub visible: bool,
    pub enabled: bool,
    pub clickable: bool,
    pub hoverable: bool,
    pub focusable: bool,
    pub pressed: bool,
    pub checked: bool,
    pub dirty: bool,
}

/// 诊断生产者观察到的控件阶段，默认是声明阶段。
///
/// 该值用于快照展示，不驱动生命周期转换，也不保证每个节点依次经历所有变体。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiWidgetLifecycleState {
    #[default]
    Declared,
    Constructed,
    PropertiesSynchronized,
    Arranged,
    Visible,
    Interactive,
    Detached,
}

/// 反射值的来源标签；默认表示作者提供的数据，不会据此重新解析或应用该数据。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiReflectedPropertySource {
    #[default]
    Authored,
    DescriptorDefault,
    InferredDefault,
    RuntimeState,
    Binding,
    SystemState,
}

/// 属性变更的失效提示，由管线域标记和反射刷新标记共同组成。
///
/// # 使用时机
/// 随诊断属性交给消费方解释；保存提示不会立即调度布局、绘制或反射更新。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPropertyInvalidationReason {
    pub dirty: UiDirtyFlags,
    pub reflection: bool,
}

impl UiPropertyInvalidationReason {
    /// 返回所有管线域和反射刷新均关闭的提示。
    pub const fn none() -> Self {
        Self {
            dirty: UiDirtyFlags {
                layout: false,
                hit_test: false,
                render: false,
                style: false,
                text: false,
                input: false,
                visible_range: false,
            },
            reflection: false,
        }
    }

    /// 返回仅反射刷新开启、管线域均关闭的提示。
    pub const fn reflection_only() -> Self {
        Self {
            dirty: UiDirtyFlags {
                layout: false,
                hit_test: false,
                render: false,
                style: false,
                text: false,
                input: false,
                visible_range: false,
            },
            reflection: true,
        }
    }

    /// 原样保留 `dirty` 并开启反射刷新；即使管线域全空，返回值仍表示有提示。
    pub const fn with_dirty(dirty: UiDirtyFlags) -> Self {
        Self {
            dirty,
            reflection: true,
        }
    }

    /// 按值读取反射位或任一管线域是否开启，不消费或清除外部待处理状态。
    pub const fn any(self) -> bool {
        self.reflection || self.dirty.any()
    }
}

/// 诊断属性的解析值、来源与编辑提示，拥有所有属性数据。
///
/// # 字段约束
/// 构造器从解析值取得类型；作者值和描述符默认值是独立备查槽，链式设置不会重新解析该值。
/// 可读/可写、失效原因和校验文本均为元数据，实际读取、写入和校验由消费方负责。
/// 反序列化使用缺省值补齐缺失字段，不额外核验类型与值的对应关系。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiReflectedProperty {
    pub name: String,
    pub value_kind: UiValueKind,
    pub resolved_value: UiValue,
    pub authored_value: Option<UiValue>,
    pub descriptor_default_value: Option<UiValue>,
    pub source: UiReflectedPropertySource,
    pub readable: bool,
    pub writable: bool,
    pub invalidation: UiPropertyInvalidationReason,
    pub visibility_affecting: bool,
    pub validation_message: Option<String>,
}

impl Default for UiReflectedProperty {
    /// 构造空名称、任意类型及空值的可读只读属性，其余可选信息留空、失效提示关闭。
    fn default() -> Self {
        Self {
            name: String::new(),
            value_kind: UiValueKind::Any,
            resolved_value: UiValue::Null,
            authored_value: None,
            descriptor_default_value: None,
            source: UiReflectedPropertySource::default(),
            readable: true,
            writable: false,
            invalidation: UiPropertyInvalidationReason::none(),
            visibility_affecting: false,
            validation_message: None,
        }
    }
}

impl UiReflectedProperty {
    /// 从名称、来源和解析值组装属性，剩余字段沿用缺省值。
    ///
    /// # 参数与返回
    /// `name` 转成拥有文本，`resolved_value` 移入结果并决定初始类型；`source` 仅记录来源。
    /// 返回值不借用输入，也不应用属性或验证业务约束。
    pub fn new(
        name: impl Into<String>,
        source: UiReflectedPropertySource,
        resolved_value: UiValue,
    ) -> Self {
        Self {
            name: name.into(),
            value_kind: resolved_value.kind(),
            resolved_value,
            source,
            ..Self::default()
        }
    }

    /// 保存可写标记并返回自身；不会安装写入处理器。
    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    /// 将作者值移入备查槽并返回自身，解析值和类型保持原样。
    pub fn authored_value(mut self, value: UiValue) -> Self {
        self.authored_value = Some(value);
        self
    }

    /// 将描述符默认值移入备查槽并返回自身，不重新计算解析值。
    pub fn descriptor_default_value(mut self, value: UiValue) -> Self {
        self.descriptor_default_value = Some(value);
        self
    }

    /// 保存失效提示并返回自身，消费方随后决定如何处理这些域。
    pub fn invalidation(mut self, invalidation: UiPropertyInvalidationReason) -> Self {
        self.invalidation = invalidation;
        self
    }

    /// 保存可见性影响提示并返回自身，不修改节点可见性。
    pub fn visibility_affecting(mut self, visibility_affecting: bool) -> Self {
        self.visibility_affecting = visibility_affecting;
        self
    }

    /// 将文本保存为已有校验说明并返回自身，不运行校验器；空文本也保留为有值。
    pub fn validation_message(mut self, message: impl Into<String>) -> Self {
        self.validation_message = Some(message.into());
        self
    }
}

/// 保留树诊断节点的拥有副本，供检查器查看结构、几何、输入状态和属性来源。
///
/// # 约束与生命周期
/// 各字段是生产者采集时的观察值，不持有实际节点、焦点或布局资源的借用。
/// 原始/有效可见性、节点状态位以及末尾输入状态分别记录，不会在此类型内自动同步。
/// 几何沿用生产者的布局坐标系；构造器不进行排布、缩放或树关系校验。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiReflectorNode {
    pub node_id: UiNodeId,
    pub node_path: UiNodePath,
    pub parent: Option<UiNodeId>,
    pub children: Vec<UiNodeId>,
    pub class_name: String,
    pub display_name: String,
    pub lifecycle: UiWidgetLifecycleState,
    pub visibility: UiVisibility,
    pub effective_visibility: UiVisibility,
    pub state_flags: UiStateFlags,
    pub input_policy: UiInputPolicy,
    pub z_index: i32,
    pub paint_order: u64,
    pub clip_to_bounds: bool,
    pub frame: UiFrame,
    pub clip_frame: UiFrame,
    pub dirty: UiDirtyFlags,
    pub properties: BTreeMap<String, UiReflectedProperty>,
    pub actions: BTreeMap<String, UiActionDescriptor>,
    pub focused: bool,
    pub hovered: bool,
    pub captured: bool,
    pub pressed: bool,
    pub source_asset: Option<String>,
    pub source_template_path: Option<String>,
}

impl UiReflectorNode {
    /// 保存身份、路径及两个名称，其余诊断字段沿用缺省值。
    ///
    /// 返回的是待填充的拥有数据；调用者仍需从实际树/布局/输入状态采集其余信息。
    pub fn new(
        node_id: UiNodeId,
        node_path: UiNodePath,
        class_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            node_id,
            node_path,
            class_name: class_name.into(),
            display_name: display_name.into(),
            ..Self::default()
        }
    }

    /// 按属性自身名称移入映射并返回自身；同名项被本次值替换，不写回实际节点。
    pub fn with_property(mut self, property: UiReflectedProperty) -> Self {
        self.properties.insert(property.name.clone(), property);
        self
    }
}

/// 一次诊断命中查询的拥有结果：查询点、目标、命中栈和拒绝说明。
///
/// 默认不带目标且列表为空；保存这些数据不会重新进行命中测试。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiReflectorHitContext {
    pub query_point: UiPoint,
    pub hit_target: Option<UiNodeId>,
    pub hit_stack: Vec<UiNodeId>,
    pub rejected: Vec<String>,
}

/// 保留树的诊断快照，含布局、输入与命中上下文；它供工作台检查器查看，不替代控制面的属性快照。
///
/// # 约束与生命周期
/// 快照拥有节点与列表；其诊断值不会随 Runtime 后续变化自动刷新。
/// 根列表、父子引用和各输入标记由生产者保证一致，本类型不修复或验证它们。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiReflectorSnapshot {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: BTreeMap<UiNodeId, UiReflectorNode>,
    pub focused: Option<UiNodeId>,
    pub captured: Option<UiNodeId>,
    pub hovered: Vec<UiNodeId>,
    pub hit_context: Option<UiReflectorHitContext>,
}

impl UiReflectorSnapshot {
    /// 将树身份、根列表和节点列表移入新的诊断快照。
    ///
    /// # 返回与约束
    /// 节点按编号收集为有序映射，重复编号保留后出现的节点；根列表不重排、不核验。
    /// 焦点、捕获、悬停和命中上下文从空状态开始，由生产者按需填充。
    pub fn new(tree_id: UiTreeId, roots: Vec<UiNodeId>, nodes: Vec<UiReflectorNode>) -> Self {
        Self {
            tree_id,
            roots,
            nodes: nodes.into_iter().map(|node| (node.node_id, node)).collect(),
            focused: None,
            captured: None,
            hovered: Vec::new(),
            hit_context: None,
        }
    }

    /// 按编号借用快照内节点；不存在时返回空，引用只在本次 `self` 借用期间有效。
    pub fn node(&self, node_id: UiNodeId) -> Option<&UiReflectorNode> {
        self.nodes.get(&node_id)
    }
}

impl UiStateFlags {
    /// 仅返回两个旧状态位的合取，不检查显式可见性、祖先或输入策略。
    pub const fn visible_enabled(&self) -> bool {
        self.visible && self.enabled
    }
}

/// 动作参数的名称、类型和可选性描述；本身不验证实参或提供默认值。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiParameterDescriptor {
    pub name: String,
    pub value_type: UiValueType,
    pub optional: bool,
}

impl UiParameterDescriptor {
    /// 保存拥有的名称和类型，返回默认不可省略的参数描述。
    pub fn new(name: impl Into<String>, value_type: UiValueType) -> Self {
        Self {
            name: name.into(),
            value_type,
            optional: false,
        }
    }

    /// 保存可选标记并返回自身，不增删调用参数。
    pub fn optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }
}

/// 控制面可查询的 JSON 属性描述，和诊断属性的来源/失效提示分别建模。
///
/// 类型、读写标记和当前值独立保存；实际写入路径负责检查可写性，本声明不检查值类型。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPropertyDescriptor {
    pub name: String,
    pub value_type: UiValueType,
    pub readable: bool,
    pub writable: bool,
    pub reflected_value: Value,
}

impl UiPropertyDescriptor {
    /// 保存名称、类型和当前 JSON 值，返回默认可读、不可写的属性。
    ///
    /// 名称转成拥有文本，值移入返回对象；不校验类型对应，也不写入 Runtime 节点。
    pub fn new(name: impl Into<String>, value_type: UiValueType, reflected_value: Value) -> Self {
        Self {
            name: name.into(),
            value_type,
            readable: true,
            writable: false,
            reflected_value,
        }
    }

    /// 保存可写标记并返回自身；实际控制服务随后决定是否允许写入。
    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }
}

/// 可查询动作的事件、绑定符号、参数模式与远程调用配置。
///
/// # 约束
/// 远程可调用标记和路由编号分别配置；记录编号不会注册路由或保证路由仍然有效。
/// 参数模式供消费方解释，不会在构造或链式设置时校验实参。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiActionDescriptor {
    pub action_id: String,
    pub event_kind: UiEventKind,
    pub binding_symbol: String,
    pub parameter_schema: Vec<UiParameterDescriptor>,
    pub callable_from_remote: bool,
    pub route_id: Option<UiRouteId>,
}

impl UiActionDescriptor {
    /// 保存动作名、事件种类和绑定符号，返回尚未开放远程调用且没有路由的动作。
    ///
    /// 两个文本参数转成拥有字符串；参数模式从空列表开始。
    pub fn new(
        action_id: impl Into<String>,
        event_kind: UiEventKind,
        binding_symbol: impl Into<String>,
    ) -> Self {
        Self {
            action_id: action_id.into(),
            event_kind,
            binding_symbol: binding_symbol.into(),
            parameter_schema: Vec::new(),
            callable_from_remote: false,
            route_id: None,
        }
    }

    /// 追加一个参数描述并返回自身，保留调用顺序，不按名称去重。
    pub fn with_parameter(mut self, parameter: UiParameterDescriptor) -> Self {
        self.parameter_schema.push(parameter);
        self
    }

    /// 保存远程可调用标记并返回自身，不注册路由或执行权限检查。
    pub fn with_callable_from_remote(mut self, callable: bool) -> Self {
        self.callable_from_remote = callable;
        self
    }

    /// 保存路由编号并返回自身；不验证管理器项目，也不延长其生命周期。
    pub fn with_route_id(mut self, route_id: UiRouteId) -> Self {
        self.route_id = Some(route_id);
        self
    }
}

/// 控制面节点的结构、状态、JSON 属性和动作视图，拥有全部可查询数据。
///
/// # 约束
/// 子节点列表与映射只是描述，构造和链式方法不验证树关系或修改真实树。
/// 可见性与状态位独立保存；属性和动作分别按名称与动作编号存入映射。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiNodeDescriptor {
    pub node_id: UiNodeId,
    pub node_path: UiNodePath,
    pub class_name: String,
    pub display_name: String,
    pub children: Vec<UiNodeId>,
    pub state_flags: UiStateFlags,
    #[serde(default)]
    pub visibility: UiVisibility,
    pub properties: BTreeMap<String, UiPropertyDescriptor>,
    pub actions: BTreeMap<String, UiActionDescriptor>,
}

impl UiNodeDescriptor {
    /// 保存节点身份、路径、类名和显示名，返回空子列表及空属性/动作映射的视图。
    ///
    /// 名称转成拥有字符串；旧状态位均关闭，而显式可见性从可见开始，调用者需独立填充状态。
    pub fn new(
        node_id: UiNodeId,
        node_path: UiNodePath,
        class_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            node_id,
            node_path,
            class_name: class_name.into(),
            display_name: display_name.into(),
            children: Vec::new(),
            state_flags: UiStateFlags::default(),
            visibility: UiVisibility::Visible,
            properties: BTreeMap::new(),
            actions: BTreeMap::new(),
        }
    }

    /// 追加子节点编号并返回自身，不去重、不建立父子引用或查找节点。
    pub fn with_child(mut self, child: UiNodeId) -> Self {
        self.children.push(child);
        self
    }

    /// 替换整组状态位并返回自身，显式可见性保持原样。
    pub fn with_state_flags(mut self, state_flags: UiStateFlags) -> Self {
        self.state_flags = state_flags;
        self
    }

    /// 保存显式可见性并返回自身，不同步旧状态位。
    pub fn with_visibility(mut self, visibility: UiVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// 按属性名称移入描述并返回自身，同名属性替换旧值。
    pub fn with_property(mut self, property: UiPropertyDescriptor) -> Self {
        self.properties.insert(property.name.clone(), property);
        self
    }

    /// 按动作编号移入描述并返回自身，同编号动作替换旧值，不注册处理器。
    pub fn with_action(mut self, action: UiActionDescriptor) -> Self {
        let key = action.action_id.clone();
        self.actions.insert(key, action);
        self
    }
}

/// 控制面按树发布的可查询属性与动作视图；事件管理器以此计算差异并响应节点查询。
///
/// 快照拥有各节点描述；发布、差异计算和查询由事件管理器消费，不在此数据类型内执行。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiReflectionSnapshot {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: BTreeMap<UiNodeId, UiNodeDescriptor>,
}

impl UiReflectionSnapshot {
    /// 保存树身份和根列表，将节点按编号收集成控制面快照。
    ///
    /// # 参数与约束
    /// 三个输入均移入结果；重复编号保留后出现的节点，根列表及子引用不做一致性校验。
    pub fn new(tree_id: UiTreeId, roots: Vec<UiNodeId>, nodes: Vec<UiNodeDescriptor>) -> Self {
        Self {
            tree_id,
            roots,
            nodes: nodes.into_iter().map(|node| (node.node_id, node)).collect(),
        }
    }
}

/// 供内部反射更新路径消费的节点补丁，拥有路径和要替换的 JSON 值。
///
/// # 约束
/// 空属性映射表示不更新属性，`pressed` 空值表示不更新该位，显式假值表示清除该位。
/// 应用方先验证目标和属性存在再写入；这条内部路径与检查可写性的控制面设置请求分别处理。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiReflectionNodePatch {
    pub node_path: UiNodePath,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pressed: Option<bool>,
}

impl UiReflectionNodePatch {
    /// 保存目标路径并返回空补丁；不查找节点或应用更新。
    pub fn new(node_path: UiNodePath) -> Self {
        Self {
            node_path,
            ..Self::default()
        }
    }

    /// 保存一个属性替换值并返回自身，同名项后值覆盖；JSON 空值仍是要写入的值。
    pub fn with_property(mut self, name: impl Into<String>, value: Value) -> Self {
        self.properties.insert(name.into(), value);
        self
    }

    /// 保存显式按压状态并返回自身；传入假值也会生成更新项。
    pub fn with_pressed(mut self, pressed: bool) -> Self {
        self.pressed = Some(pressed);
        self
    }
}

/// 按树记录变化与删除节点的拥有通知数据。
///
/// 列表的生成、去重及两者关系由生产者决定；此类型不查找节点或自动修复列表。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiReflectionDiff {
    pub tree_id: UiTreeId,
    pub changed_nodes: Vec<UiNodeId>,
    pub removed_nodes: Vec<UiNodeId>,
}

impl UiReflectionDiff {
    /// 仅在变化列表和删除列表都为空时返回真，树身份不参与该判定。
    pub fn is_empty(&self) -> bool {
        self.changed_nodes.is_empty() && self.removed_nodes.is_empty()
    }
}
