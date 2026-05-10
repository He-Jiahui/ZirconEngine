use serde::{Deserialize, Serialize};

use super::ReflectTypeRegistration;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectSchemaFilter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_path: Option<String>,
    pub include_components: bool,
    pub include_resources: bool,
    pub editor_visible: bool,
    pub remote_visible: bool,
    pub include_plugin_owned: bool,
}

impl ReflectSchemaFilter {
    pub fn editor_visible() -> Self {
        Self {
            include_components: true,
            include_resources: true,
            editor_visible: true,
            ..Self::default()
        }
    }

    pub fn remote_visible() -> Self {
        Self {
            include_components: true,
            include_resources: true,
            remote_visible: true,
            ..Self::default()
        }
    }

    pub fn for_type(type_path: impl Into<String>) -> Self {
        Self {
            type_path: Some(type_path.into()),
            include_components: true,
            include_resources: true,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectSchemaRequest {
    pub filter: ReflectSchemaFilter,
}

impl ReflectSchemaRequest {
    pub fn new(filter: ReflectSchemaFilter) -> Self {
        Self { filter }
    }

    pub fn editor_visible() -> Self {
        Self::new(ReflectSchemaFilter::editor_visible())
    }

    pub fn remote_visible() -> Self {
        Self::new(ReflectSchemaFilter::remote_visible())
    }

    pub fn for_type(type_path: impl Into<String>) -> Self {
        Self::new(ReflectSchemaFilter::for_type(type_path))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectSchemaResponse {
    pub registrations: Vec<ReflectTypeRegistration>,
}

impl ReflectSchemaResponse {
    pub fn new(registrations: Vec<ReflectTypeRegistration>) -> Self {
        Self { registrations }
    }
}
