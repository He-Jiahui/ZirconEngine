use crate::asset::AssetReference;
use crate::core::math::Real;
use serde::ser::SerializeStruct;
use serde::Serializer;
use serde::{Deserialize, Serialize};

use super::defaults::{is_zero_i32, is_zero_real};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneMeshPrimitiveBindingAsset {
    pub mesh: AssetReference,
    pub material: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct SceneMeshLodLevelAsset {
    #[serde(default)]
    pub min_distance: Real,
    pub model: AssetReference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh: Option<AssetReference>,
    pub material: AssetReference,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primitives: Vec<SceneMeshPrimitiveBindingAsset>,
}

impl Serialize for SceneMeshLodLevelAsset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut len = 3;
            if self.mesh.is_some() {
                len += 1;
            }
            if !self.primitives.is_empty() {
                len += 1;
            }

            let mut state = serializer.serialize_struct("SceneMeshLodLevelAsset", len)?;
            state.serialize_field("min_distance", &self.min_distance)?;
            state.serialize_field("model", &self.model)?;
            if let Some(mesh) = &self.mesh {
                state.serialize_field("mesh", mesh)?;
            }
            state.serialize_field("material", &self.material)?;
            if !self.primitives.is_empty() {
                state.serialize_field("primitives", &self.primitives)?;
            }
            state.end()
        } else {
            let mut state = serializer.serialize_struct("SceneMeshLodLevelAsset", 5)?;
            state.serialize_field("min_distance", &self.min_distance)?;
            state.serialize_field("model", &self.model)?;
            state.serialize_field("mesh", &self.mesh)?;
            state.serialize_field("material", &self.material)?;
            state.serialize_field("primitives", &self.primitives)?;
            state.end()
        }
    }
}

impl SceneMeshLodLevelAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::with_capacity(self.direct_reference_count());
        self.append_direct_references(&mut references);
        references
    }

    fn append_direct_references(&self, references: &mut Vec<AssetReference>) {
        references.push(self.model.clone());
        references.extend(self.mesh.iter().cloned());
        references.push(self.material.clone());
        for primitive in &self.primitives {
            references.push(primitive.mesh.clone());
            references.push(primitive.material.clone());
        }
    }

    pub fn direct_mesh_reference_count(&self) -> usize {
        usize::from(self.mesh.is_some()) + self.primitives.len()
    }

    pub fn direct_reference_count(&self) -> usize {
        2 + usize::from(self.mesh.is_some()) + (self.primitives.len() * 2)
    }

    pub fn primitive_binding_count(&self) -> usize {
        self.primitives.len()
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct SceneMeshInstanceAsset {
    pub model: AssetReference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh: Option<AssetReference>,
    pub material: AssetReference,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub render_queue: i32,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub material_queue: i32,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub order_in_layer: i32,
    #[serde(default, skip_serializing_if = "is_zero_real")]
    pub depth_bias: Real,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morph_weights: Vec<Real>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primitives: Vec<SceneMeshPrimitiveBindingAsset>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lods: Vec<SceneMeshLodLevelAsset>,
}

impl Serialize for SceneMeshInstanceAsset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut len = 2;
            if self.mesh.is_some() {
                len += 1;
            }
            if !is_zero_i32(&self.render_queue) {
                len += 1;
            }
            if !is_zero_i32(&self.material_queue) {
                len += 1;
            }
            if !is_zero_i32(&self.order_in_layer) {
                len += 1;
            }
            if !is_zero_real(&self.depth_bias) {
                len += 1;
            }
            if !self.morph_weights.is_empty() {
                len += 1;
            }
            if !self.primitives.is_empty() {
                len += 1;
            }
            if !self.lods.is_empty() {
                len += 1;
            }

            let mut state = serializer.serialize_struct("SceneMeshInstanceAsset", len)?;
            state.serialize_field("model", &self.model)?;
            if let Some(mesh) = &self.mesh {
                state.serialize_field("mesh", mesh)?;
            }
            state.serialize_field("material", &self.material)?;
            if !is_zero_i32(&self.render_queue) {
                state.serialize_field("render_queue", &self.render_queue)?;
            }
            if !is_zero_i32(&self.material_queue) {
                state.serialize_field("material_queue", &self.material_queue)?;
            }
            if !is_zero_i32(&self.order_in_layer) {
                state.serialize_field("order_in_layer", &self.order_in_layer)?;
            }
            if !is_zero_real(&self.depth_bias) {
                state.serialize_field("depth_bias", &self.depth_bias)?;
            }
            if !self.morph_weights.is_empty() {
                state.serialize_field("morph_weights", &self.morph_weights)?;
            }
            if !self.primitives.is_empty() {
                state.serialize_field("primitives", &self.primitives)?;
            }
            if !self.lods.is_empty() {
                state.serialize_field("lods", &self.lods)?;
            }
            state.end()
        } else {
            let mut state = serializer.serialize_struct("SceneMeshInstanceAsset", 10)?;
            state.serialize_field("model", &self.model)?;
            state.serialize_field("mesh", &self.mesh)?;
            state.serialize_field("material", &self.material)?;
            state.serialize_field("render_queue", &self.render_queue)?;
            state.serialize_field("material_queue", &self.material_queue)?;
            state.serialize_field("order_in_layer", &self.order_in_layer)?;
            state.serialize_field("depth_bias", &self.depth_bias)?;
            state.serialize_field("morph_weights", &self.morph_weights)?;
            state.serialize_field("primitives", &self.primitives)?;
            state.serialize_field("lods", &self.lods)?;
            state.end()
        }
    }
}

impl SceneMeshInstanceAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::with_capacity(self.direct_reference_count());
        references.push(self.model.clone());
        references.extend(self.mesh.iter().cloned());
        references.push(self.material.clone());
        for primitive in &self.primitives {
            references.push(primitive.mesh.clone());
            references.push(primitive.material.clone());
        }
        for lod in &self.lods {
            lod.append_direct_references(&mut references);
        }
        references
    }

    pub fn direct_mesh_reference_count(&self) -> usize {
        usize::from(self.mesh.is_some())
            + self.primitives.len()
            + self
                .lods
                .iter()
                .map(SceneMeshLodLevelAsset::direct_mesh_reference_count)
                .sum::<usize>()
    }

    pub fn direct_reference_count(&self) -> usize {
        2 + usize::from(self.mesh.is_some())
            + (self.primitives.len() * 2)
            + self
                .lods
                .iter()
                .map(SceneMeshLodLevelAsset::direct_reference_count)
                .sum::<usize>()
    }

    pub fn primitive_binding_count(&self) -> usize {
        self.primitives.len()
            + self
                .lods
                .iter()
                .map(SceneMeshLodLevelAsset::primitive_binding_count)
                .sum::<usize>()
    }

    pub fn morph_weight_count(&self) -> usize {
        self.morph_weights.len()
    }

    pub fn lod_level_count(&self) -> usize {
        self.lods.len()
    }
}

#[cfg(test)]
#[path = "mesh/lod_append_tests.rs"]
mod lod_append_tests;
