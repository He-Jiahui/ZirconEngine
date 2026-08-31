use std::sync::Arc;

use crate::asset::{AssetReference, MeshAsset, ModelAsset, ModelPrimitiveAsset};
use crate::core::resource::ResourceId;

use super::super::GpuMeshResource;
use super::GpuModelResource;

impl GpuModelResource {
    pub(in crate::graphics::scene::resources) fn from_primitives(
        device: &wgpu::Device,
        id: ResourceId,
        primitives: Vec<ModelPrimitiveAsset>,
    ) -> Self {
        let mut meshes = Vec::with_capacity(primitives.len());
        for primitive in primitives {
            meshes.push(Arc::new(GpuMeshResource::from_asset(device, primitive)));
        }
        Self { id, meshes }
    }
}

pub(in crate::graphics::scene::resources) fn model_primitives_preferring_mesh_assets<F>(
    asset: &ModelAsset,
    mut load_mesh_asset: F,
) -> Vec<ModelPrimitiveAsset>
where
    F: FnMut(&AssetReference) -> Option<MeshAsset>,
{
    let mut primitives = Vec::with_capacity(asset.primitives.len());
    for primitive in &asset.primitives {
        primitives.push(
            primitive
                .mesh
                .as_ref()
                .and_then(|reference| load_mesh_asset(reference))
                .and_then(|mesh| mesh.to_model_primitive().ok())
                .unwrap_or_else(|| primitive.clone()),
        );
    }
    primitives
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::model_primitives_preferring_mesh_assets;
    use crate::asset::{
        AssetReference, AssetUri, MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION,
        MESH_ATTRIBUTE_UV0, MeshAsset, MeshAttributeValues, MeshIndices, MeshVertex, ModelAsset,
        ModelPrimitiveAsset,
    };
    use crate::core::framework::render::RenderMeshTopology;

    #[test]
    fn model_render_primitives_use_referenced_mesh_asset_payload_when_available() {
        let mesh_reference = asset_reference("res://models/hero.gltf#Mesh0/Primitive0");
        let model = model_with_primitive(embedded_primitive(1.0, Some(mesh_reference.clone())));
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
    fn model_render_primitives_keep_embedded_payload_when_mesh_reference_unresolved() {
        let embedded = embedded_primitive(
            1.0,
            Some(asset_reference("res://models/hero.gltf#Mesh0/Primitive0")),
        );
        let model = model_with_primitive(embedded.clone());

        let selected = model_primitives_preferring_mesh_assets(&model, |_| None);

        assert_eq!(selected, vec![embedded]);
    }

    #[test]
    fn model_resource_paths_reserve_primitive_capacity() {
        let source = include_str!("gpu_model_resource_from_asset.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(primitives.len())"));
        assert!(implementation.contains("Vec::with_capacity(asset.primitives.len())"));
        assert!(implementation.contains("for primitive in primitives"));
        assert!(implementation.contains("for primitive in &asset.primitives"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cc_runtime_model_primitive_capacity_p95() {
        use std::time::Instant;
        const SAMPLE_PAIRS: usize = 17;
        const PRIMITIVES_PER_SAMPLE: usize = 256;
        fn measure(optimized: bool) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..128 {
                let mut output = if optimized {
                    Vec::with_capacity(PRIMITIVES_PER_SAMPLE)
                } else {
                    Vec::new()
                };
                for index in 0..PRIMITIVES_PER_SAMPLE {
                    output.push(index);
                }
                checksum ^= output.len();
            }
            std::hint::black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }
        fn percentile(samples: &[u128], p: usize) -> u128 {
            let mut s = samples.to_vec();
            s.sort_unstable();
            s[(s.len() * p).div_ceil(100).saturating_sub(1)]
        }
        fn csv(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME381_MODEL_PRIMITIVE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} primitives_per_sample={PRIMITIVES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn model_with_primitive(primitive: ModelPrimitiveAsset) -> ModelAsset {
        ModelAsset {
            uri: AssetUri::parse("res://models/hero.gltf").unwrap(),
            primitives: vec![primitive],
        }
    }

    fn embedded_primitive(x: f32, mesh: Option<AssetReference>) -> ModelPrimitiveAsset {
        ModelPrimitiveAsset {
            vertices: vec![
                vertex([x, 0.0, 0.0]),
                vertex([x + 1.0, 0.0, 0.0]),
                vertex([x, 1.0, 0.0]),
            ],
            indices: vec![0, 1, 2],
            mesh,
            mesh_sdf: None,
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
            uv1: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            joint_indices: [0, 0, 0, 0],
            joint_weights: [0.0, 0.0, 0.0, 0.0],
        }
    }

    fn asset_reference(uri: &str) -> AssetReference {
        AssetReference::from_locator(AssetUri::parse(uri).unwrap())
    }
}
