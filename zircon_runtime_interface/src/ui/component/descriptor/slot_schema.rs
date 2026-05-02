use serde::{Deserialize, Serialize};

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
