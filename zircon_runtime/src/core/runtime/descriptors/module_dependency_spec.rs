use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleDependencySpec {
    pub module_name: String,
}

impl ModuleDependencySpec {
    pub fn named(module_name: impl Into<String>) -> Self {
        Self {
            module_name: module_name.into(),
        }
    }
}
