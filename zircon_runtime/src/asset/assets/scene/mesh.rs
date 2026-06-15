use crate::asset::AssetReference;
use crate::core::math::Real;
use serde::{Deserialize, Serialize};

use super::defaults::{is_zero_i32, is_zero_real};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneMeshPrimitiveBindingAsset {
    pub mesh: AssetReference,
    pub material: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

impl SceneMeshLodLevelAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::with_capacity(3 + (self.primitives.len() * 2));
        references.push(self.model.clone());
        references.extend(self.mesh.iter().cloned());
        references.push(self.material.clone());
        for primitive in &self.primitives {
            references.push(primitive.mesh.clone());
            references.push(primitive.material.clone());
        }
        references
    }

    pub fn direct_mesh_reference_count(&self) -> usize {
        usize::from(self.mesh.is_some()) + self.primitives.len()
    }

    pub fn primitive_binding_count(&self) -> usize {
        self.primitives.len()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

impl SceneMeshInstanceAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::with_capacity(
            3 + (self.primitives.len() * 2)
                + self
                    .lods
                    .iter()
                    .map(|lod| 3 + (lod.primitives.len() * 2))
                    .sum::<usize>(),
        );
        references.push(self.model.clone());
        references.extend(self.mesh.iter().cloned());
        references.push(self.material.clone());
        for primitive in &self.primitives {
            references.push(primitive.mesh.clone());
            references.push(primitive.material.clone());
        }
        for lod in &self.lods {
            references.extend(lod.direct_references());
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
