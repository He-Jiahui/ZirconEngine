use serde::{Deserialize, Serialize};

use super::value::UiResourceRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 来源枚举区分资源值由哪个文档字段收集，便于诊断定位和失效依赖汇总。
pub enum UiResourceDependencySource {
    DocumentImport,
    NodeProp,
    NodeLayout,
    NodeStyleOverride,
    ChildMountSlot,
    StyleRuleDeclaration,
    TokenValue,
    ImportedWidget,
    ImportedStyle,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 收集结果将完整资源引用与声明来源路径配对；后续解析和热重载按此保留归属。
pub struct UiResourceDependency {
    pub reference: UiResourceRef,
    pub source: UiResourceDependencySource,
    pub path: String,
}
