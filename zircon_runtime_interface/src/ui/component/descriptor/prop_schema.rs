use serde::{Deserialize, Serialize};

use crate::ui::component::{UiValue, UiValueKind};

use super::UiOptionDescriptor;

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
