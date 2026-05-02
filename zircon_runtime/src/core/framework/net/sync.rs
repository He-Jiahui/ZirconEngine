use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncAuthority {
    Server,
    ClientOwned,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncFieldDescriptor {
    pub name: String,
    pub value_type: String,
    pub delta_compressed: bool,
}

impl SyncFieldDescriptor {
    pub fn new(name: impl Into<String>, value_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value_type: value_type.into(),
            delta_compressed: true,
        }
    }

    pub fn delta_compressed(mut self, enabled: bool) -> Self {
        self.delta_compressed = enabled;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncComponentDescriptor {
    pub component_type: String,
    pub authority: SyncAuthority,
    pub fields: Vec<SyncFieldDescriptor>,
    pub update_hz: u16,
    pub interest_group: Option<String>,
}

impl SyncComponentDescriptor {
    pub fn new(component_type: impl Into<String>, authority: SyncAuthority) -> Self {
        Self {
            component_type: component_type.into(),
            authority,
            fields: Vec::new(),
            update_hz: 20,
            interest_group: None,
        }
    }

    pub fn with_field(mut self, field: SyncFieldDescriptor) -> Self {
        self.fields.push(field);
        self
    }

    pub fn with_update_hz(mut self, update_hz: u16) -> Self {
        self.update_hz = update_hz;
        self
    }

    pub fn with_interest_group(mut self, group: impl Into<String>) -> Self {
        self.interest_group = Some(group.into());
        self
    }
}
