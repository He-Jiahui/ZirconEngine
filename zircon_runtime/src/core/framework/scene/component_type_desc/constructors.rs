use super::{ComponentPropertyDescriptor, ComponentTypeDescriptor};

impl ComponentPropertyDescriptor {
    pub fn new(name: impl Into<String>, value_type: impl Into<String>, editable: bool) -> Self {
        Self {
            name: name.into(),
            value_type: value_type.into(),
            editable,
        }
    }
}

impl ComponentTypeDescriptor {
    pub fn new(
        type_id: impl Into<String>,
        plugin_id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            type_id: type_id.into(),
            plugin_id: plugin_id.into(),
            display_name: display_name.into(),
            properties: Vec::new(),
        }
    }

    pub fn with_property(
        mut self,
        name: impl Into<String>,
        value_type: impl Into<String>,
        editable: bool,
    ) -> Self {
        self.properties
            .push(ComponentPropertyDescriptor::new(name, value_type, editable));
        self
    }
}
