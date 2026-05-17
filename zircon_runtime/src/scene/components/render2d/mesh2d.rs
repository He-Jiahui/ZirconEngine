use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderMaterialAlphaMode;
use crate::core::math::Vec4;
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mesh2dComponent {
    pub mesh: ResourceHandle<ModelMarker>,
    pub material: ResourceHandle<MaterialMarker>,
    pub color: Vec4,
    pub z_order: i32,
    #[serde(default)]
    pub material_alpha_mode: RenderMaterialAlphaMode,
}

impl Default for Mesh2dComponent {
    fn default() -> Self {
        Self {
            mesh: ResourceHandle::new(ResourceId::from_stable_label("builtin://quad")),
            material: ResourceHandle::new(ResourceId::from_stable_label(
                "builtin://material/default",
            )),
            color: Vec4::ONE,
            z_order: 0,
            material_alpha_mode: RenderMaterialAlphaMode::Opaque,
        }
    }
}
