use serde::{Deserialize, Serialize};

use super::value::UiResourceRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 来源枚举区分资源值由哪个文档字段收集，便于诊断定位和失效依赖汇总。
/// 这些标签描述声明来源；Serde 保留变体名，标签本身不表示资源已加载。
pub enum UiResourceDependencySource {
    /// 文档导入清单中的显式资源声明。
    DocumentImport,
    /// 节点属性中的资源值。
    NodeProp,
    /// 节点布局字段中的资源值。
    NodeLayout,
    /// 节点局部样式覆盖中的资源值。
    NodeStyleOverride,
    /// 子挂载槽位数据中的资源值。
    ChildMountSlot,
    /// 样式规则声明中的资源值。
    StyleRuleDeclaration,
    /// 文档 token 值中的资源声明。
    TokenValue,
    /// 导入 Widget 文档中的资源声明。
    ImportedWidget,
    /// 导入 Style 文档中的资源声明。
    ImportedStyle,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 收集结果将完整资源引用与声明来源路径配对；后续解析和热重载按此保留归属。
pub struct UiResourceDependency {
    /// 拥有的完整声明；种类、原始 URI 和回退策略都参与声明身份。
    pub reference: UiResourceRef,
    /// 保留的声明来源类别；来源归并策略由收集器决定。
    pub source: UiResourceDependencySource,
    /// 作者可定位的声明位置标签，不是待打开的文件路径。
    pub path: String,
}
