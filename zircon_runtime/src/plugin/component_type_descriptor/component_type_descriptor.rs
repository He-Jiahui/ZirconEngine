use serde::{Deserialize, Serialize};

use super::ComponentPropertyDescriptor;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentTypeDescriptor {
    pub type_id: String,
    pub plugin_id: String,
    pub display_name: String,
    #[serde(default)]
    pub properties: Vec<ComponentPropertyDescriptor>,
}
