use std::fmt::Write as _;

use crate::core::framework::scene::ScenePropertyValue;
use crate::scene::EntityId;
use crate::scene::components::MeshRenderer;

use super::super::super::World;

const MESH_RENDERER_MORPH_WEIGHT_PATH_PREFIX: &str = "MeshRenderer.morph_weights.";

impl World {
    pub(super) fn visit_mesh_property_entries<F>(&self, entity: EntityId, visitor: &mut F) -> bool
    where
        F: FnMut(&str, &mut dyn FnMut() -> ScenePropertyValue, bool) -> bool,
    {
        macro_rules! push_entry {
            ($path:expr, $value:expr, $animatable:expr $(,)?) => {
                let mut build_value = || $value;
                if !visitor($path, &mut build_value, $animatable) {
                    return false;
                }
            };
        }

        if let Some(mesh) = self.get::<MeshRenderer>(entity) {
            push_entry!(
                "MeshRenderer.model",
                ScenePropertyValue::Resource(mesh.model.id().to_string()),
                false,
            );
            if let Some(mesh_handle) = mesh.mesh {
                push_entry!(
                    "MeshRenderer.mesh",
                    ScenePropertyValue::Resource(mesh_handle.id().to_string()),
                    false,
                );
            }
            push_entry!(
                "MeshRenderer.material",
                ScenePropertyValue::Resource(mesh.material.id().to_string()),
                false,
            );
            push_entry!(
                "MeshRenderer.render_queue",
                ScenePropertyValue::Integer(mesh.render_queue.into()),
                true,
            );
            push_entry!(
                "MeshRenderer.material_queue",
                ScenePropertyValue::Integer(mesh.material_queue.into()),
                true,
            );
            push_entry!(
                "MeshRenderer.order_in_layer",
                ScenePropertyValue::Integer(mesh.order_in_layer.into()),
                true,
            );
            push_entry!(
                "MeshRenderer.depth_bias",
                ScenePropertyValue::Scalar(mesh.depth_bias),
                true,
            );
            push_entry!(
                "MeshRenderer.primitive_binding_count",
                ScenePropertyValue::Unsigned(mesh.primitives.len() as u64),
                false,
            );
            push_entry!(
                "MeshRenderer.lod_level_count",
                ScenePropertyValue::Unsigned(mesh.lods.len() as u64),
                false,
            );
            push_entry!(
                "MeshRenderer.morph_weight_count",
                ScenePropertyValue::Unsigned(mesh.morph_weights.len() as u64),
                false,
            );
            let mut morph_weight_index = 0;
            while morph_weight_index < mesh.morph_weights.len() {
                let path = mesh_renderer_morph_weight_path(morph_weight_index);
                let weight = mesh.morph_weights[morph_weight_index];
                push_entry!(&path, ScenePropertyValue::Scalar(weight), true,);
                morph_weight_index += 1;
            }
            push_entry!(
                "MeshRenderer.tint",
                ScenePropertyValue::Vec4(mesh.tint.to_array()),
                true,
            );
        }

        true
    }

    pub(super) fn mesh_property_entry_capacity_hint(&self, entity: EntityId) -> usize {
        let mut capacity = 0;
        if let Some(mesh) = self.get::<MeshRenderer>(entity) {
            capacity += 10 + mesh.morph_weights.len();
            if mesh.mesh.is_some() {
                capacity += 1;
            }
        }
        capacity
    }
}

fn mesh_renderer_morph_weight_path(index: usize) -> String {
    let prefix_len = MESH_RENDERER_MORPH_WEIGHT_PATH_PREFIX.len();
    let mut path = String::with_capacity(prefix_len + decimal_digit_count(index));
    path.push_str(MESH_RENDERER_MORPH_WEIGHT_PATH_PREFIX);
    write!(&mut path, "{index}").expect("writing to a String cannot fail");
    path
}

fn decimal_digit_count(mut value: usize) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}
