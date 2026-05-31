use std::sync::Arc;

use crate::asset::{AssetReference, MeshAsset, ModelAsset, ModelPrimitiveAsset};
use crate::core::resource::ResourceId;

use super::super::GpuMeshResource;
use super::GpuModelResource;

impl GpuModelResource {
    pub(in crate::graphics::scene::resources) fn from_asset_with_mesh_assets<F>(
        device: &wgpu::Device,
        id: ResourceId,
        asset: &ModelAsset,
        load_mesh_asset: F,
    ) -> Self
    where
        F: FnMut(&AssetReference) -> Option<MeshAsset>,
    {
        Self {
            id,
            meshes: model_primitives_preferring_mesh_assets(asset, load_mesh_asset)
                .into_iter()
                .map(|primitive| Arc::new(GpuMeshResource::from_asset(device, primitive)))
                .collect(),
        }
    }
}

pub(in crate::graphics::scene::resources) fn model_primitives_preferring_mesh_assets<F>(
    asset: &ModelAsset,
    mut load_mesh_asset: F,
) -> Vec<ModelPrimitiveAsset>
where
    F: FnMut(&AssetReference) -> Option<MeshAsset>,
{
    asset
        .primitives
        .iter()
        .map(|primitive| {
            primitive
                .mesh
                .as_ref()
                .and_then(|reference| load_mesh_asset(reference))
                .and_then(|mesh| mesh.to_model_primitive().ok())
                .unwrap_or_else(|| primitive.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::model_primitives_preferring_mesh_assets;
    use crate::asset::{
        AssetReference, AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MeshVertex,
        ModelAsset, ModelPrimitiveAsset, MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION,
        MESH_ATTRIBUTE_UV0,
    };
    use crate::core::framework::render::RenderMeshTopology;

    #[test]
    fn model_render_primitives_use_referenced_mesh_asset_payload_when_available() {
        let mesh_reference = asset_reference("res://models/hero.gltf#Mesh0/Primitive0");
        let model = model_with_primitive(legacy_primitive(1.0, Some(mesh_reference.clone())));
        let mesh_asset = mesh_asset("res://models/hero.gltf#Mesh0/Primitive0", 10.0);

        let selected = model_primitives_preferring_mesh_assets(&model, |reference| {
            assert_eq!(reference, &mesh_reference);
            Some(mesh_asset.clone())
        });

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].vertices[0].position, [10.0, 0.0, 0.0]);
        assert_eq!(selected[0].indices, vec![0, 2, 1]);
        assert!(selected[0].mesh.is_none());
    }

    #[test]
    fn model_render_primitives_keep_legacy_payload_when_mesh_reference_unresolved() {
        let legacy = legacy_primitive(
            1.0,
            Some(asset_reference("res://models/hero.gltf#Mesh0/Primitive0")),
        );
        let model = model_with_primitive(legacy.clone());

        let selected = model_primitives_preferring_mesh_assets(&model, |_| None);

        assert_eq!(selected, vec![legacy]);
    }

    fn model_with_primitive(primitive: ModelPrimitiveAsset) -> ModelAsset {
        ModelAsset {
            uri: AssetUri::parse("res://models/hero.gltf").unwrap(),
            primitives: vec![primitive],
        }
    }

    fn legacy_primitive(x: f32, mesh: Option<AssetReference>) -> ModelPrimitiveAsset {
        ModelPrimitiveAsset {
            vertices: vec![
                vertex([x, 0.0, 0.0]),
                vertex([x + 1.0, 0.0, 0.0]),
                vertex([x, 1.0, 0.0]),
            ],
            indices: vec![0, 1, 2],
            mesh,
            virtual_geometry: None,
        }
    }

    fn mesh_asset(uri: &str, x: f32) -> MeshAsset {
        let mut attributes = BTreeMap::new();
        attributes.insert(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[x, 0.0, 0.0], [x + 1.0, 0.0, 0.0], [x, 1.0, 0.0]]),
        );
        attributes.insert(
            MESH_ATTRIBUTE_NORMAL.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
        );
        attributes.insert(
            MESH_ATTRIBUTE_UV0.to_string(),
            MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
        );

        MeshAsset::new(
            AssetUri::parse(uri).unwrap(),
            RenderMeshTopology::TriangleList,
            attributes,
            Some(MeshIndices::U32(vec![0, 2, 1])),
        )
        .unwrap()
    }

    fn vertex(position: [f32; 3]) -> MeshVertex {
        MeshVertex {
            position,
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            joint_indices: [0, 0, 0, 0],
            joint_weights: [0.0, 0.0, 0.0, 0.0],
        }
    }

    fn asset_reference(uri: &str) -> AssetReference {
        AssetReference::from_locator(AssetUri::parse(uri).unwrap())
    }
}
