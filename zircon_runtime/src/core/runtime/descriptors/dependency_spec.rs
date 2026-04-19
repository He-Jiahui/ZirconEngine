use serde::{Deserialize, Serialize};

use super::RegistryName;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencySpec {
    pub name: RegistryName,
}

impl DependencySpec {
    pub fn named(name: RegistryName) -> Self {
        Self { name }
    }
}
