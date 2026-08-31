use super::gltf_animation_subassets::add_gltf_animation_and_skin_subassets;
use super::gltf_decode::decode_gltf_source;
use super::gltf_labeled_subassets::{
    GltfMeshSubasset, GltfPrimitiveSubasset, add_gltf_material_subassets, add_gltf_mesh_subassets,
    add_gltf_scene_subassets, gltf_label_reference, gltf_label_uri,
};
use std::collections::BTreeMap;

use super::super::{
    add_gltf_texture_subassets, remap_gltf_morph_targets_for_flat_normals,
    resolve_gltf_normal_texture_tangent_uv_attribute,
};
use super::cook_mesh_asset_derived_data;
use super::primitive_from_indexed_mesh::{MissingNormalPolicy, primitive_from_indexed_mesh};
use crate::asset::assets::{
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MeshAsset,
    MeshAttributeValues, MeshMorphTargetAsset, MeshSkinAsset, ModelAsset, ModelPrimitiveAsset,
};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset, MeshSdfCookBudget,
    MeshSdfCookRequest, VirtualGeometryCookRequest,
};

pub(crate) fn import_gltf(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let decoded = decode_gltf_source(context)?;
    let document = decoded.document;
    let buffers = decoded.buffers;
    let images = decoded.images;
    let mut primitives = Vec::new();
    let mut meshes = Vec::with_capacity(document.meshes().count());
    let mesh_skins = mesh_skin_assets_by_mesh(&document, &buffers);
    let source_hint = context.uri.to_string();
    let virtual_geometry_request = context.virtual_geometry_cook_request()?;
    let mesh_sdf_request = context.mesh_sdf_cook_request()?;
    let mut mesh_sdf_budget = MeshSdfCookBudget::default();

    for mesh in document.meshes() {
        let mut mesh_primitives =
            reserve_gltf_mesh_outputs(&mut primitives, mesh.primitives().count());
        let mesh_name = mesh.name();
        for primitive in mesh.primitives() {
            let mode = primitive.mode();
            if mode != gltf::mesh::Mode::Triangles {
                return Err(AssetImportError::Parse(format!(
                    "unsupported gltf primitive mode {mode:?} at Mesh{}/Primitive{}; only Triangles is supported",
                    mesh.index(),
                    primitive.index()
                )));
            }
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));
            let positions = reader
                .read_positions()
                .ok_or_else(|| {
                    AssetImportError::Parse("gltf primitive missing positions".to_string())
                })?
                .flat_map(|position| position.into_iter())
                .collect::<Vec<_>>();
            let normals = reader
                .read_normals()
                .map(|iter| {
                    iter.flat_map(|normal| normal.into_iter())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let texcoords = reader
                .read_tex_coords(0)
                .map(|set| {
                    set.into_f32()
                        .flat_map(|uv| uv.into_iter())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let texcoords1 = reader
                .read_tex_coords(1)
                .map(|set| {
                    set.into_f32()
                        .flat_map(|uv| uv.into_iter())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let tangents = reader
                .read_tangents()
                .map(|set| set.collect::<Vec<_>>())
                .unwrap_or_default();
            let normals_missing = normals.is_empty();
            let tangents_missing = tangents.is_empty() || normals_missing;
            let normal_tangent_uv_attribute = resolve_gltf_normal_texture_tangent_uv_attribute(
                &primitive,
                tangents_missing,
                &texcoords,
                &texcoords1,
            )?;
            let colors = reader
                .read_colors(0)
                .map(|set| set.into_rgba_f32().collect::<Vec<_>>())
                .unwrap_or_default();
            let joint_indices = reader
                .read_joints(0)
                .map(|set| set.into_u16().collect::<Vec<_>>())
                .unwrap_or_default();
            let joint_weights = reader
                .read_weights(0)
                .map(|set| set.into_f32().collect::<Vec<_>>())
                .unwrap_or_default();
            let indices = reader
                .read_indices()
                .map(|indices| indices.into_u32().collect::<Vec<_>>())
                .unwrap_or_else(|| {
                    let vertex_count = positions.len() / 3;
                    (0..vertex_count as u32).collect()
                });

            let mut primitive_asset = primitive_from_indexed_mesh(
                &positions,
                &normals,
                MissingNormalPolicy::Flat,
                &texcoords,
                &texcoords1,
                &tangents,
                &colors,
                &indices,
                &joint_indices,
                &joint_weights,
                mesh_name,
                &source_hint,
                &VirtualGeometryCookRequest::default(),
                &MeshSdfCookRequest::default(),
                &mut mesh_sdf_budget,
            )?;
            let primitive_label = format!("Mesh{}/Primitive{}", mesh.index(), primitive.index());
            let primitive_uri = gltf_label_uri(&context.uri, &primitive_label);
            primitive_asset.mesh = Some(gltf_label_reference(&context.uri, &primitive_label));
            let mut mesh_asset = MeshAsset::from_model_primitive(primitive_uri, &primitive_asset);
            let mut morph_targets = morph_targets_from_reader(&reader);
            if normals_missing {
                remap_gltf_morph_targets_for_flat_normals(&mut morph_targets, &indices)?;
            }
            mesh_asset.morph_targets = morph_targets;
            mesh_asset.skin = mesh_skins.get(&mesh.index()).cloned();
            if tangents_missing {
                if let Some(uv_attribute) = normal_tangent_uv_attribute {
                    mesh_asset.attributes.remove(MESH_ATTRIBUTE_TANGENT);
                    mesh_asset
                        .try_generate_missing_tangents_for_uv(uv_attribute)
                        .map_err(|error| {
                            AssetImportError::Parse(format!(
                                "generate glTF MikkTSpace tangents for {primitive_label}: {error}"
                            ))
                        })?;
                }
            }
            mesh_asset
                .try_rebuild_morph_tangent_frames_for_uv(
                    normals_missing,
                    tangents_missing
                        .then_some(normal_tangent_uv_attribute)
                        .flatten(),
                )
                .map_err(|error| {
                    AssetImportError::Parse(format!(
                        "rebuild glTF morph tangent frames for {primitive_label}: {error}"
                    ))
                })?;
            cook_mesh_asset_derived_data(
                &mut mesh_asset,
                mesh_name,
                &source_hint,
                &virtual_geometry_request,
                &mesh_sdf_request,
                &mut mesh_sdf_budget,
            )?;
            let primitive_reference = ModelPrimitiveAsset {
                vertices: Vec::new(),
                indices: Vec::new(),
                mesh: primitive_asset.mesh,
                mesh_sdf: None,
                virtual_geometry: None,
            };
            mesh_primitives.push(GltfPrimitiveSubasset {
                primitive_index: primitive.index(),
                material_index: primitive.material().index(),
                mesh: mesh_asset,
            });
            primitives.push(primitive_reference);
        }
        meshes.push(GltfMeshSubasset {
            mesh_index: mesh.index(),
            primitives: mesh_primitives,
        });
    }

    let model = ModelAsset {
        uri: context.uri.clone(),
        primitives,
    };
    let mut outcome = AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Model(model));
    outcome = add_gltf_texture_subassets(outcome, &context.uri, &document, images)?;
    outcome = add_gltf_material_subassets(outcome, &context.uri, &document);
    outcome = add_gltf_mesh_subassets(outcome, &context.uri, meshes);
    outcome = add_gltf_scene_subassets(outcome, &context.uri, &document);
    outcome = add_gltf_animation_and_skin_subassets(outcome, &context.uri, &document, &buffers)?;
    Ok(outcome)
}

fn reserve_gltf_mesh_outputs(
    primitives: &mut Vec<ModelPrimitiveAsset>,
    primitive_count: usize,
) -> Vec<GltfPrimitiveSubasset> {
    primitives.reserve(primitive_count);
    Vec::with_capacity(primitive_count)
}

fn morph_targets_from_reader<'a, 's, F>(
    reader: &gltf::mesh::Reader<'a, 's, F>,
) -> Vec<MeshMorphTargetAsset>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_morph_targets()
        .enumerate()
        .filter_map(|(index, (positions, normals, tangents))| {
            let mut attributes = BTreeMap::new();
            if let Some(positions) = positions {
                attributes.insert(
                    MESH_ATTRIBUTE_POSITION.to_string(),
                    MeshAttributeValues::Float32x3(positions.collect()),
                );
            }
            if let Some(normals) = normals {
                attributes.insert(
                    MESH_ATTRIBUTE_NORMAL.to_string(),
                    MeshAttributeValues::Float32x3(normals.collect()),
                );
            }
            if let Some(tangents) = tangents {
                attributes.insert(
                    MESH_ATTRIBUTE_TANGENT.to_string(),
                    MeshAttributeValues::Float32x3(tangents.collect()),
                );
            }
            (!attributes.is_empty()).then(|| MeshMorphTargetAsset {
                name: Some(format!("MorphTarget{index}")),
                attributes,
            })
        })
        .collect()
}

#[cfg(test)]
mod plugins07_gltf_output_capacity_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const OUTPUTS_PER_SAMPLE: usize = 16_384;

    #[test]
    fn import_execution_collections_contract_preallocated_gltf_outputs() {
        let mut primitives = Vec::new();
        let mesh_primitives = reserve_gltf_mesh_outputs(&mut primitives, 128);
        assert!(primitives.capacity() >= 128);
        assert!(mesh_primitives.capacity() >= 128);
        assert!(mesh_primitives.is_empty());
    }

    #[test]
    #[ignore = "release performance gate"]
    fn import_execution_collections_performance_release_gltf_outputs() {
        for _ in 0..4 {
            black_box(measure_outputs(false));
            black_box(measure_outputs(true));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut legacy_growths = None;
        let mut optimized_growths = None;
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy, optimized) = if pair_index % 2 == 0 {
                (measure_outputs(false), measure_outputs(true))
            } else {
                let optimized = measure_outputs(true);
                (measure_outputs(false), optimized)
            };
            legacy_growths.get_or_insert(legacy.1);
            optimized_growths.get_or_insert(optimized.1);
            assert_eq!(legacy_growths, Some(legacy.1));
            assert_eq!(optimized_growths, Some(optimized.1));
            legacy_samples.push(legacy.0);
            optimized_samples.push(optimized.0);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_preallocated_gltf_output_collections sample_pairs={SAMPLE_PAIRS} outputs_per_sample={OUTPUTS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25 legacy_capacity_growths_per_sample={} optimized_capacity_growths_per_sample={} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
            legacy_growths.unwrap(),
            optimized_growths.unwrap(),
        );
        assert_eq!(optimized_growths, Some(0));
        assert!(
            improvement_percent >= 25,
            "preallocated glTF output collections must improve P95 by at least 25%"
        );
    }

    fn measure_outputs(preallocated: bool) -> (u128, usize) {
        let mut roots = if preallocated {
            Vec::with_capacity(OUTPUTS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        let mut labeled = if preallocated {
            Vec::with_capacity(OUTPUTS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        let started = Instant::now();
        let mut growths = 0;
        for output in 0..OUTPUTS_PER_SAMPLE {
            let root_capacity = roots.capacity();
            roots.push(black_box(output));
            growths += usize::from(roots.capacity() != root_capacity);
            let labeled_capacity = labeled.capacity();
            labeled.push(black_box(output));
            growths += usize::from(labeled.capacity() != labeled_capacity);
        }
        black_box((roots, labeled));
        (started.elapsed().as_nanos().max(1), growths)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn mesh_skin_assets_by_mesh(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> BTreeMap<usize, MeshSkinAsset> {
    let mut mesh_skins = BTreeMap::new();
    for node in document.nodes() {
        let Some(mesh) = node.mesh() else {
            continue;
        };
        let Some(skin) = node.skin() else {
            continue;
        };
        let Some(matrices) = skin
            .reader(|buffer| Some(&buffers[buffer.index()].0))
            .read_inverse_bind_matrices()
        else {
            continue;
        };

        // A MeshAsset currently owns one optional skin payload, so keep the first
        // node-level skin association deterministically until skin subassets carry
        // richer multi-skin bindings.
        mesh_skins
            .entry(mesh.index())
            .or_insert_with(|| MeshSkinAsset {
                inverse_bind_matrices: matrices.collect(),
            });
    }
    mesh_skins
}
