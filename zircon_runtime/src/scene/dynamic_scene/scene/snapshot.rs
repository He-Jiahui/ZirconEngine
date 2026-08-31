use serde::{Deserialize, Serialize};
use zircon_runtime_interface::serialization::PayloadHeader;

use super::super::{DynamicEntity, DynamicResource};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicScene {
    #[serde(
        skip,
        default = "crate::scene::dynamic_scene::document::current_dynamic_scene_header"
    )]
    pub(in crate::scene::dynamic_scene) payload_header: PayloadHeader,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_types: Vec<crate::core::framework::scene::ComponentTypeDescriptor>,
    #[serde(default)]
    pub entities: Vec<DynamicEntity>,
    #[serde(default)]
    pub resources: Vec<DynamicResource>,
}

impl DynamicScene {
    pub fn empty() -> Self {
        Self {
            payload_header: crate::scene::dynamic_scene::document::current_dynamic_scene_header(),
            component_types: Vec::new(),
            entities: Vec::new(),
            resources: Vec::new(),
        }
    }
}
