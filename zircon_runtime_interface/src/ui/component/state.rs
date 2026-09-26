//! 每节点组件状态承载类型化值、交互标志和拖放来源；Runtime 状态归约负责状态迁移。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{UiDragSourceMetadata, UiValidationState, UiValue};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiComponentFlags {
    pub focused: bool,
    pub focus_visible: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub dragging: bool,
    pub drop_hovered: bool,
    pub active_drag_target: bool,
    pub popup_open: bool,
    pub expanded: bool,
    pub selected: bool,
    pub checked: bool,
    pub disabled: bool,
    pub loading: bool,
}

/// 保留类型化值与拖放来源，并提供会失效旧来源的写入辅助方法。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentState {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub values: BTreeMap<String, UiValue>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub reference_sources: BTreeMap<String, UiDragSourceMetadata>,
    #[serde(default)]
    pub validation: UiValidationState,
    #[serde(default)]
    pub flags: UiComponentFlags,
}

impl Default for UiComponentState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiComponentState {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
            reference_sources: BTreeMap::new(),
            validation: UiValidationState::normal(),
            flags: UiComponentFlags::default(),
        }
    }

    /// 替换属性值时撤销该属性原有的拖放来源。
    pub fn with_value(mut self, property: impl Into<String>, value: UiValue) -> Self {
        let property = property.into();
        // Replacing a retained value directly invalidates drag/drop provenance for that slot.
        self.reference_sources.remove(&property);
        self.values.insert(property, value);
        self
    }

    pub fn value(&self, property: &str) -> Option<&UiValue> {
        self.values.get(property)
    }

    pub fn reference_source(&self, property: &str) -> Option<&UiDragSourceMetadata> {
        self.reference_sources.get(property)
    }
}
