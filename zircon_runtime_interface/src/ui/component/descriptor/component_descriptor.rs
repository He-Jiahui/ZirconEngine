use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ui::component::{
    UiComponentCategory, UiComponentEventKind, UiDragPayloadKind, UiDropPolicy, UiValue,
};

use super::{
    UiDefaultNodeTemplate, UiHostCapability, UiPaletteMetadata, UiPropSchema, UiRenderCapability,
    UiSlotSchema, UiWidgetFallbackPolicy,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentDescriptor {
    pub id: String,
    pub display_name: String,
    pub category: UiComponentCategory,
    pub role: String,
    pub default_props: Vec<(String, UiValue)>,
    pub prop_schema: Vec<UiPropSchema>,
    pub state_schema: Vec<UiPropSchema>,
    pub slot_schema: Vec<UiSlotSchema>,
    pub events: Vec<UiComponentEventKind>,
    pub drop_policy: UiDropPolicy,
    #[serde(default)]
    pub required_host_capabilities: BTreeSet<UiHostCapability>,
    #[serde(default)]
    pub required_render_capabilities: BTreeSet<UiRenderCapability>,
    #[serde(default)]
    pub palette: Option<UiPaletteMetadata>,
    #[serde(default)]
    pub default_node_template: UiDefaultNodeTemplate,
    #[serde(default)]
    pub fallback_policy: UiWidgetFallbackPolicy,
}

impl UiComponentDescriptor {
    /// Creates a component descriptor with identity, category, and host-rendering role.
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        category: UiComponentCategory,
        role: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            category,
            role: role.into(),
            default_props: Vec::new(),
            prop_schema: Vec::new(),
            state_schema: Vec::new(),
            slot_schema: Vec::new(),
            events: Vec::new(),
            drop_policy: UiDropPolicy::default(),
            required_host_capabilities: BTreeSet::new(),
            required_render_capabilities: BTreeSet::new(),
            palette: None,
            default_node_template: UiDefaultNodeTemplate::default(),
            fallback_policy: UiWidgetFallbackPolicy::default(),
        }
    }

    /// Adds a default authored prop value for this component.
    pub fn default_prop(mut self, name: impl Into<String>, value: UiValue) -> Self {
        self.default_props.push((name.into(), value));
        self
    }

    /// Adds a typed authored prop schema.
    pub fn with_prop(mut self, schema: UiPropSchema) -> Self {
        self.prop_schema.push(schema);
        self
    }

    /// Adds a typed retained-state schema.
    pub fn state(mut self, schema: UiPropSchema) -> Self {
        self.state_schema.push(schema);
        self
    }

    /// Adds a content slot schema.
    pub fn slot(mut self, schema: UiSlotSchema) -> Self {
        self.slot_schema.push(schema);
        self
    }

    /// Adds one supported component event kind.
    pub fn event(mut self, event: UiComponentEventKind) -> Self {
        if !self.events.contains(&event) {
            self.events.push(event);
        }
        self
    }

    /// Adds multiple supported component event kinds, preserving first-seen order.
    pub fn events(mut self, events: impl IntoIterator<Item = UiComponentEventKind>) -> Self {
        for event in events {
            if !self.events.contains(&event) {
                self.events.push(event);
            }
        }
        self
    }

    /// Sets the drag/drop acceptance policy for reference-like components.
    pub fn drop_policy(mut self, policy: UiDropPolicy) -> Self {
        self.drop_policy = policy;
        self
    }

    pub fn requires_host_capability(mut self, capability: UiHostCapability) -> Self {
        let _ = self.required_host_capabilities.insert(capability);
        self
    }

    pub fn requires_render_capability(mut self, capability: UiRenderCapability) -> Self {
        let _ = self.required_render_capabilities.insert(capability);
        self
    }

    pub fn palette(mut self, metadata: UiPaletteMetadata) -> Self {
        self.palette = Some(metadata);
        self
    }

    pub fn default_node_template(mut self, template: UiDefaultNodeTemplate) -> Self {
        if let Some(metadata) = &mut self.palette {
            metadata.default_node = template.clone();
        }
        self.default_node_template = template;
        self
    }

    pub fn fallback_policy(mut self, policy: UiWidgetFallbackPolicy) -> Self {
        self.fallback_policy = policy;
        self
    }

    /// Returns the declared prop schema for a component prop name.
    pub fn prop(&self, name: &str) -> Option<&UiPropSchema> {
        self.prop_schema.iter().find(|schema| schema.name == name)
    }

    /// Returns the declared retained-state schema for a component state name.
    pub fn state_prop(&self, name: &str) -> Option<&UiPropSchema> {
        self.state_schema.iter().find(|schema| schema.name == name)
    }

    /// Returns the declared slot schema for a component slot name.
    pub fn slot_schema(&self, name: &str) -> Option<&UiSlotSchema> {
        self.slot_schema.iter().find(|schema| schema.name == name)
    }

    /// Returns whether the component advertises support for an event kind.
    pub fn supports_event(&self, event: UiComponentEventKind) -> bool {
        self.events.contains(&event)
    }

    /// Returns whether the component accepts a drag payload kind.
    pub fn accepts_drag_payload(&self, kind: UiDragPayloadKind) -> bool {
        self.drop_policy.accepts(kind)
    }
}
