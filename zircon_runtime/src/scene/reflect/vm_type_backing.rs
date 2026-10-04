use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VmTypeBacking {
    DynamicComponent,
}

#[cfg(test)]
#[path = "tests/vm_type_backing.rs"]
mod tests;
