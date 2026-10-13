//! Runtime 缓存和实例化使用的扁平 UI 资产原型。
//! schema migrator 将树形 [`UiAssetDocument`](crate::ui::template::UiAssetDocument) 转成节点表；
//! 子节点关系用句柄保存，文件缓存共享原型，实例化器再按句柄遍历。
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;

use super::{
    UiAssetHeader, UiAssetImports, UiComponentParamSchema, UiComponentPublicContract,
    UiNamedSlotSchema, UiNodeDefinitionKind, UiStyleDeclarationBlock, UiStyleScope, UiStyleSheet,
};
use crate::ui::accessibility::UiAccessibilityContract;
use crate::ui::focus::UiFocusContract;
use crate::ui::navigation::UiNavigationContract;
use crate::ui::picking::UiPickPolicy;
use crate::ui::template::UiBindingRef;
use crate::ui::widget::UiWidgetContract;

/// 扁平原型节点表的索引；迁移器分配句柄，实例化器再按句柄读取节点。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 原型节点表内的数值索引，不拥有节点，也不延长原型生命周期。
/// 句柄只在创建它的 `UiRawAssetPrototype` 中有意义；迁移器生成的句柄有界，但公开构造与反序列化值须经原型查表，越界访问返回 `None`。
pub struct UiPrototypeNodeHandle(pub u32);

impl UiPrototypeNodeHandle {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// 扁平节点形式的资产原型；文件缓存保存它，实例化器从节点表展开树。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Runtime loader 将已迁移的资产文档压平后缓存的原型数据。
/// 内部句柄由迁移阶段按同一节点表生成；公开构造或 serde 解码仍可产生无效索引，实例化与访问应通过 `node` 取值。
pub struct UiRawAssetPrototype {
    pub asset: UiAssetHeader,
    #[serde(default)]
    pub imports: UiAssetImports,
    #[serde(default)]
    pub tokens: BTreeMap<String, Value>,
    pub document: UiDocumentPrototype,
    #[serde(default)]
    pub components: BTreeMap<String, UiComponentPrototype>,
    #[serde(default)]
    pub styles: Vec<UiStylePrototype>,
}

impl UiRawAssetPrototype {
    /// 句柄按节点表索引读取；越界时返回 None，让迁移/实例化调用方处理无效引用而不越界访问。
    /// 在本原型自己的节点表中借用句柄目标；跨原型句柄不具备稳定身份，越界时返回 None。
    pub fn node(&self, handle: UiPrototypeNodeHandle) -> Option<&UiNodePrototype> {
        self.document.nodes.get(handle.index())
    }

    /// 节点数与扁平表长度一致，配合句柄范围校验而非树遍历计数。
    pub fn node_count(&self) -> usize {
        self.document.nodes.len()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 扁平文档节点表；迁移器把树节点放入 `nodes` 并写入 root/child 位置索引，实例化器按该表遍历。
/// 字段可公开构造或反序列化，结构体本身不检查 root 与 child 索引是否有效。
pub struct UiDocumentPrototype {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// 可实例化文档根；纯组件/样式资产可不声明文档根。
    pub root: Option<UiPrototypeNodeHandle>,
    #[serde(default)]
    /// 唯一节点存储；句柄不拥有节点，也不延长原型本身的生命周期。
    pub nodes: Vec<UiNodePrototype>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 扁平原型中的组件导出项；migration 保留组件 root 与参数/槽契约，组件验证通过并注册后由实例化器按 root 展开。
pub struct UiComponentPrototype {
    pub root: UiPrototypeNodeHandle,
    #[serde(default)]
    pub style_scope: UiStyleScope,
    #[serde(default)]
    pub contract: UiComponentPublicContract,
    #[serde(default)]
    pub params: BTreeMap<String, UiComponentParamSchema>,
    #[serde(default)]
    pub slots: BTreeMap<String, UiNamedSlotSchema>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 原型中保留的独立样式声明；loader 迁移但不在此结构中解析选择器，样式编译阶段再解析并建立级联。
pub struct UiStylePrototype {
    pub stylesheet: UiStyleSheet,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 扁平节点保留展开器需要的源属性和契约；children 改用索引句柄以支持缓存存储。
/// 节点树压平后的单项记录；子节点引用同一 `UiDocumentPrototype.nodes` 表中的句柄。
/// `component_ref` 等跨资产声明仍需在 Runtime 组件解析/实例化阶段解析，反序列化本身不校验引用或句柄。
pub struct UiNodePrototype {
    #[serde(default)]
    pub node_id: String,
    #[serde(default)]
    pub kind: UiNodeDefinitionKind,
    #[serde(default, rename = "type")]
    pub widget_type: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    /// 可选跨资产组件引用，仍按源引用格式在展开阶段验证。
    pub component_ref: Option<String>,
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
    /// 子节点挂载顺序及其节点表句柄。
    pub children: Vec<UiPrototypeChildMount>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 扁平子挂载以句柄指向节点表，mount 与 slot 继续承载树形源文档中的挂载信息。
pub struct UiPrototypeChildMount {
    #[serde(default)]
    pub mount: Option<String>,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
    pub child: UiPrototypeNodeHandle,
}
