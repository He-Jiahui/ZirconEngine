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
    pub component: String,
    pub control_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_path: Option<Vec<UiTemplateNodeInstancePathStep>>,
    #[serde(default)]
    pub pixel_snapping: UiPixelSnappingPolicy,
    pub classes: Vec<String>,
    pub attributes: BTreeMap<String, Value>,
    /// Active localized property bindings retain source identity after text resolution.
    /// Explicit property writes clear the corresponding binding, including empty resets.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub localized_text_references: BTreeMap<String, UiLocalizedTextRef>,
    pub slot_attributes: BTreeMap<String, Value>,
    pub style_overrides: BTreeMap<String, Value>,
    pub style_tokens: BTreeMap<String, String>,
    pub bindings: Vec<UiBindingRef>,
    #[serde(default)]
    pub a11y: UiAccessibilityContract,
    #[serde(default)]
    pub widget: UiWidgetContract,
}
