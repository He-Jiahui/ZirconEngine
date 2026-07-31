use crate::core::framework::render::{MaterialPropertyOverrideBlock, RenderMaterialAlphaMode};
use crate::core::math::{Real, Vec4};
use crate::core::resource::{MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshRendererPrimitiveBinding {
    pub mesh: ResourceHandle<MeshMarker>,
    pub material: ResourceHandle<MaterialMarker>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshRendererLodLevel {
    #[serde(default)]
    pub min_distance: Real,
    pub model: ResourceHandle<ModelMarker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh: Option<ResourceHandle<MeshMarker>>,
    pub material: ResourceHandle<MaterialMarker>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primitives: Vec<MeshRendererPrimitiveBinding>,
}

impl MeshRendererLodLevel {
    pub fn from_handles(
        min_distance: Real,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> Self {
        Self {
            min_distance,
            model,
            mesh: None,
            material,
            primitives: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::MeshRenderer",
    script_visibility = "public"
)]
pub struct MeshRenderer {
    #[zr_reflect(
        value_type_path = "Resource",
        editor_hint = "Resource",
        read = "super::reflection::mesh_renderer::read_model",
        readonly
    )]
    pub model: ResourceHandle<ModelMarker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[zr_reflect(
        value_type_path = "Resource",
        editor_hint = "Resource",
        read = "super::reflection::mesh_renderer::read_mesh",
        readonly
    )]
    pub mesh: Option<ResourceHandle<MeshMarker>>,
    #[zr_reflect(
        value_type_path = "Resource",
        editor_hint = "Resource",
        read = "super::reflection::mesh_renderer::read_material",
        readonly
    )]
    pub material: ResourceHandle<MaterialMarker>,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub render_queue: i32,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub material_queue: i32,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub order_in_layer: i32,
    #[serde(default, skip_serializing_if = "is_zero_real")]
    pub depth_bias: Real,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[zr_reflect(
        value_type_path = "List",
        editor_hint = "None",
        read = "super::reflection::mesh_renderer::read_morph_weights",
        readonly
    )]
    pub morph_weights: Vec<Real>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[zr_reflect(
        value_type_path = "List",
        editor_hint = "None",
        read = "super::reflection::mesh_renderer::read_primitives",
        readonly
    )]
    pub primitives: Vec<MeshRendererPrimitiveBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[zr_reflect(
        value_type_path = "List",
        editor_hint = "None",
        read = "super::reflection::mesh_renderer::read_lods",
        readonly
    )]
    pub lods: Vec<MeshRendererLodLevel>,
    #[serde(
        default,
        skip_serializing_if = "MaterialPropertyOverrideBlock::is_empty"
    )]
    #[zr_reflect(skip)]
    pub material_property_overrides: MaterialPropertyOverrideBlock,
    pub tint: Vec4,
    #[serde(default)]
    #[zr_reflect(skip)]
    pub material_alpha_mode: RenderMaterialAlphaMode,
}

impl MeshRenderer {
    pub fn from_handles(
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> Self {
        Self {
            model,
            mesh: None,
            material,
            render_queue: 0,
            material_queue: 0,
            order_in_layer: 0,
            depth_bias: 0.0,
            morph_weights: Vec::new(),
            primitives: Vec::new(),
            lods: Vec::new(),
            material_property_overrides: MaterialPropertyOverrideBlock::default(),
            tint: Vec4::ONE,
            material_alpha_mode: RenderMaterialAlphaMode::Opaque,
        }
    }
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self::from_handles(
            ResourceHandle::new(ResourceId::from_stable_label("builtin://cube")),
            ResourceHandle::new(ResourceId::from_stable_label("builtin://material/default")),
        )
    }
}

fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}

fn is_zero_real(value: &Real) -> bool {
    *value == 0.0
}
