use serde::{Deserialize, Serialize};

use crate::ui::{component::UiValue, event_ui::UiNodeId, tree::UiDirtyFlags};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingSourceKind {
    #[default]
    RetainedAttribute,
    RuntimeState,
    RuntimeEcs,
    WidgetBehavior,
    AccessibilityAction,
    ComponentEvent,
    HostProjection,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingTargetKind {
    #[default]
    RetainedAttribute,
    RuntimeState,
    ComponentStateValue,
    ComponentStateFlag,
    RuntimeEcs,
    WidgetAlias,
    HostProjection,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiBindingSource {
    pub kind: UiBindingSourceKind,
    pub node_id: Option<UiNodeId>,
    pub property: Option<String>,
    pub path: Option<String>,
}

impl UiBindingSource {
    pub fn retained_attribute(node_id: UiNodeId, property: impl Into<String>) -> Self {
        Self::node_property(UiBindingSourceKind::RetainedAttribute, node_id, property)
    }

    pub fn runtime_state(node_id: UiNodeId, property: impl Into<String>) -> Self {
        Self::node_property(UiBindingSourceKind::RuntimeState, node_id, property)
    }

    pub fn runtime_ecs(path: impl Into<String>) -> Self {
        Self {
            kind: UiBindingSourceKind::RuntimeEcs,
            path: Some(path.into()),
            ..Self::default()
        }
    }

    pub fn widget_behavior(node_id: UiNodeId, property: impl Into<String>) -> Self {
        Self::node_property(UiBindingSourceKind::WidgetBehavior, node_id, property)
    }

    pub fn accessibility_action(node_id: UiNodeId, property: impl Into<String>) -> Self {
        Self::node_property(UiBindingSourceKind::AccessibilityAction, node_id, property)
    }

    pub fn component_event(node_id: UiNodeId, property: impl Into<String>) -> Self {
        Self::node_property(UiBindingSourceKind::ComponentEvent, node_id, property)
    }

    fn node_property(
        kind: UiBindingSourceKind,
        node_id: UiNodeId,
        property: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            node_id: Some(node_id),
            property: Some(property.into()),
            path: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiBindingTarget {
    pub kind: UiBindingTargetKind,
    pub node_id: Option<UiNodeId>,
    pub property: Option<String>,
    pub path: Option<String>,
}

impl UiBindingTarget {
    pub fn retained_attribute(node_id: UiNodeId, property: impl Into<String>) -> Self {
        Self::node_property(UiBindingTargetKind::RetainedAttribute, node_id, property)
    }

    pub fn runtime_state(node_id: UiNodeId, property: impl Into<String>) -> Self {
        Self::node_property(UiBindingTargetKind::RuntimeState, node_id, property)
    }

    pub fn component_state_value(node_id: UiNodeId, property: impl Into<String>) -> Self {
        Self::node_property(UiBindingTargetKind::ComponentStateValue, node_id, property)
    }

    pub fn component_state_flag(node_id: UiNodeId, property: impl Into<String>) -> Self {
        Self::node_property(UiBindingTargetKind::ComponentStateFlag, node_id, property)
    }

    pub fn runtime_ecs(path: impl Into<String>) -> Self {
        Self {
            kind: UiBindingTargetKind::RuntimeEcs,
            path: Some(path.into()),
            ..Self::default()
        }
    }

    pub fn widget_alias(node_id: UiNodeId, property: impl Into<String>) -> Self {
        Self::node_property(UiBindingTargetKind::WidgetAlias, node_id, property)
    }

    fn node_property(
        kind: UiBindingTargetKind,
        node_id: UiNodeId,
        property: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            node_id: Some(node_id),
            property: Some(property.into()),
            path: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingDirtyDomain {
    Layout,
    HitTest,
    Render,
    Style,
    Text,
    Input,
    VisibleRange,
    Accessibility,
    Interaction,
    Schedule,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingUpdateStatus {
    #[default]
    Applied,
    Unchanged,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiBindingUpdate {
    pub source: UiBindingSource,
    pub target: UiBindingTarget,
    pub previous: Option<UiValue>,
    pub value: UiValue,
    pub status: UiBindingUpdateStatus,
    pub dirty: Vec<UiBindingDirtyDomain>,
    pub message: Option<String>,
}

impl Default for UiBindingUpdate {
    fn default() -> Self {
        Self {
            source: UiBindingSource::default(),
            target: UiBindingTarget::default(),
            previous: None,
            value: UiValue::Null,
            status: UiBindingUpdateStatus::Applied,
            dirty: Vec::new(),
            message: None,
        }
    }
}

impl UiBindingUpdate {
    pub fn applied(source: UiBindingSource, target: UiBindingTarget, value: UiValue) -> Self {
        Self {
            source,
            target,
            value,
            status: UiBindingUpdateStatus::Applied,
            ..Self::default()
        }
    }

    pub fn unchanged(source: UiBindingSource, target: UiBindingTarget, value: UiValue) -> Self {
        Self {
            source,
            target,
            value,
            status: UiBindingUpdateStatus::Unchanged,
            ..Self::default()
        }
    }

    pub fn rejected(
        source: UiBindingSource,
        target: UiBindingTarget,
        value: UiValue,
        message: impl Into<String>,
    ) -> Self {
        Self {
            source,
            target,
            value,
            status: UiBindingUpdateStatus::Rejected,
            message: Some(message.into()),
            ..Self::default()
        }
    }

    pub fn with_previous(mut self, previous: Option<UiValue>) -> Self {
        self.previous = previous;
        self
    }

    pub fn with_dirty(mut self, dirty: impl IntoIterator<Item = UiBindingDirtyDomain>) -> Self {
        self.dirty = dirty.into_iter().collect();
        self
    }

    pub fn with_dirty_flags(mut self, flags: UiDirtyFlags) -> Self {
        self.dirty = UiBindingDirtyDomain::from_dirty_flags(flags);
        self
    }
}

impl UiBindingDirtyDomain {
    pub fn from_dirty_flags(flags: UiDirtyFlags) -> Vec<Self> {
        let mut domains = Vec::new();
        if flags.layout {
            domains.push(Self::Layout);
        }
        if flags.hit_test {
            domains.push(Self::HitTest);
        }
        if flags.render {
            domains.push(Self::Render);
        }
        if flags.style {
            domains.push(Self::Style);
        }
        if flags.text {
            domains.push(Self::Text);
        }
        if flags.input {
            domains.push(Self::Input);
        }
        if flags.visible_range {
            domains.push(Self::VisibleRange);
        }
        domains
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiBindingUpdateReport {
    pub updates: Vec<UiBindingUpdate>,
    pub applied_count: u64,
    pub unchanged_count: u64,
    pub rejected_count: u64,
    pub dirty: Vec<UiBindingDirtyDomain>,
}

impl UiBindingUpdateReport {
    pub fn from_updates(updates: Vec<UiBindingUpdate>) -> Self {
        let mut report = Self {
            updates,
            ..Self::default()
        };
        report.recompute();
        report
    }

    pub fn recompute(&mut self) {
        self.applied_count = 0;
        self.unchanged_count = 0;
        self.rejected_count = 0;
        self.dirty.clear();

        for update in &self.updates {
            match update.status {
                UiBindingUpdateStatus::Applied => self.applied_count += 1,
                UiBindingUpdateStatus::Unchanged => self.unchanged_count += 1,
                UiBindingUpdateStatus::Rejected => self.rejected_count += 1,
            }
            for domain in &update.dirty {
                if !self.dirty.contains(domain) {
                    self.dirty.push(*domain);
                }
            }
        }
    }
}
