use serde::{Deserialize, Serialize};

use super::{
    UiComponentCategory, UiComponentEventKind, UiDragPayloadKind, UiDropPolicy, UiValue,
    UiValueKind,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiOptionDescriptor {
    pub id: String,
    pub label: String,
    pub value: UiValue,
    pub disabled: bool,
    pub special_condition: Option<String>,
}

impl UiOptionDescriptor {
    /// Creates an option descriptor with a stable id, display label, and typed value.
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: UiValue) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value,
            disabled: false,
            special_condition: None,
        }
    }

    /// Marks whether the option should reject selection.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the option as a special-condition row such as a mixed inspector value.
    pub fn special_condition(mut self, condition: impl Into<String>) -> Self {
        self.special_condition = Some(condition.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPropSchema {
    pub name: String,
    pub value_kind: UiValueKind,
    pub required: bool,
    pub default_value: Option<UiValue>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub options: Vec<UiOptionDescriptor>,
}

impl UiPropSchema {
    /// Creates a prop or retained-state schema with a stable name and value kind.
    pub fn new(name: impl Into<String>, value_kind: UiValueKind) -> Self {
        Self {
            name: name.into(),
            value_kind,
            required: false,
            default_value: None,
            min: None,
            max: None,
            step: None,
            options: Vec::new(),
        }
    }

    /// Marks whether the prop is required by authored component nodes.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Sets the typed default value for the prop or retained-state schema.
    pub fn default_value(mut self, value: UiValue) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Sets an inclusive numeric range for numeric schemas.
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }

    /// Sets the numeric step used by drag, slider, or spinner-style controls.
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Attaches structured option metadata to enum-like props.
    pub fn with_options(mut self, options: impl IntoIterator<Item = UiOptionDescriptor>) -> Self {
        self.options = options.into_iter().collect();
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSlotSchema {
    pub name: String,
    pub required: bool,
    pub multiple: bool,
}

impl UiSlotSchema {
    /// Creates a content slot schema with a stable slot name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required: false,
            multiple: false,
        }
    }

    /// Marks whether the slot must be authored by component nodes.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Marks whether the slot accepts multiple child nodes.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }
}

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
