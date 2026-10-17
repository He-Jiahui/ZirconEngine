use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;

use crate::ui::accessibility::UiAccessibilityContract;
use crate::ui::layout::UiPixelSnappingPolicy;
use crate::ui::template::{UiBindingRef, UiLocalizedTextRef};
use crate::ui::v2::UiTemplateNodeInstancePathStep;
use crate::ui::widget::UiWidgetContract;

/// 随 retained 节点携带的模板作者数据，供 Runtime 布局、绘制、输入和 Editor 预览复用。
/// 构树时从模板生成；虚拟行等调用方可复制并补写实例属性，不能假定它始终等于原模板。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiTemplateNodeMetadata {
    /// 构建器保留的组件名称，供后续按组件解释属性。
    pub component: String,
    /// 可选控件查询标识；本结构不检查唯一性，构建器和控件索引另行约束。
    pub control_id: Option<String>,
    /// 模板来源路径；缺失时不从节点路径推断来源。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    /// 来源模板中的节点标识，与运行时节点编号分别保存。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_node_id: Option<String>,
    /// 模板实例展开路径；旧构建路径可不提供这些来源信息。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_path: Option<Vec<UiTemplateNodeInstancePathStep>>,
    /// 作者指定的像素对齐策略，供绘制提取时解析。
    #[serde(default)]
    pub pixel_snapping: UiPixelSnappingPolicy,
    /// 作者声明的样式类列表。
    pub classes: Vec<String>,
    /// 节点当前属性值；显式属性写入可更新此映射。
    pub attributes: BTreeMap<String, Value>,
    /// 显式写入即使与当前值相同，也会解除对应本地化绑定；该清理由属性写入方执行。
    /// Active localized property bindings retain source identity after text resolution.
    /// Explicit property writes clear the corresponding binding, including empty resets.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub localized_text_references: BTreeMap<String, UiLocalizedTextRef>,
    /// 父子边的槽位作者属性，供构建器生成布局槽。
    pub slot_attributes: BTreeMap<String, Value>,
    /// 节点级样式覆盖值，与原始属性和样式令牌分别保存。
    pub style_overrides: BTreeMap<String, Value>,
    /// 作者指定的样式令牌引用。
    pub style_tokens: BTreeMap<String, String>,
    /// 保留的绑定声明；存入此列表本身不执行绑定。
    pub bindings: Vec<UiBindingRef>,
    /// 无障碍作者契约；由相应消费者解释。
    #[serde(default)]
    pub a11y: UiAccessibilityContract,
    /// 控件作者契约；不在此结构内实例化控件行为。
    #[serde(default)]
    pub widget: UiWidgetContract,
}
