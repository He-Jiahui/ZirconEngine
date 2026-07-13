use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentPropertyDescriptor {
    pub name: String,
    pub value_type: String,
    pub editable: bool,
}
