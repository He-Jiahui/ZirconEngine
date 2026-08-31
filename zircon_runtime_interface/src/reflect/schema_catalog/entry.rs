use serde::{Deserialize, Serialize};

use crate::reflect::ReflectTypeRegistration;

/// One neutral reflection registration and its explicit schema dependencies.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReflectSchemaCatalogEntry {
    pub registration: ReflectTypeRegistration,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

impl ReflectSchemaCatalogEntry {
    pub fn new(registration: ReflectTypeRegistration) -> Self {
        Self {
            registration,
            dependencies: Vec::new(),
        }
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }
}
