use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::asset::{MeshAsset, ModelPrimitiveAsset};
use crate::core::framework::render::{
    DisplayMode, RenderMeshLodSelection, RenderMeshSnapshot, RenderMeshStaticState,
};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::{RenderMat4, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId};
use crate::graphics::scene::scene_renderer::mesh::skinning::SkinnedMeshJointPaletteUniform;

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::super::resources::{
    default_pipeline_key, GpuMeshResource, MaterialRuntime, ResourceStreamer,
};
use super::super::super::super::primitives::render_mat4_or;
use super::super::super::mesh_draw::MaterialTextureSet;
use super::super::raster_draws_for_mesh::raster_draws_for_mesh;
use super::mesh_draw_build_context::MeshDrawBuildContext;
use super::pending_mesh_draw::{PendingMeshDraw, PendingMeshGeometry, PendingSkinnedGpuSource};
use super::skinning::{
    prepare_skinned_mesh_asset_primitive, prepare_skinned_model_primitive,
    SkinnedMeshPreparedPrimitive,
};

struct DynamicMeshPrimitive {
    primitive: ModelPrimitiveAsset,
    skinned: bool,
    skinned_palette_signature: Option<u64>,
    skinned_joint_palette: Option<SkinnedMeshJointPaletteUniform>,
    skinned_gpu_source: Option<PendingSkinnedGpuSource>,
}

pub(super) fn extend_pending_draws_for_mesh_instance(
    pending_draws: &mut Vec<PendingMeshDraw>,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    build_context: &MeshDrawBuildContext,
    mesh_instance: &RenderMeshSnapshot,
) {
    if let Some(allowed_entities) = build_context.allowed_virtual_geometry_entities.as_ref() {
        if !allowed_entities.contains(&mesh_instance.node_id) {
            return;
        }
    }

    let instance_tint = if build_context.selection.contains(&mesh_instance.node_id)
        && frame.overlays().display_mode != DisplayMode::WireOnly
    {
        mesh_instance.tint * Vec4::new(1.0, 0.94, 0.72, 1.0)
    } else {
        mesh_instance.tint
    };
    let model_matrix =
        render_mat4_or(mesh_instance.transform.matrix(), RenderMat4::IDENTITY).to_cols_array_2d();
    let material_revision = streamer.material_revision(&mesh_instance.material.id());
    let mut draw_ordinal = 0_u32;
    // Direct mesh snapshots bypass the model-primitive loop, so mirror the same CPU skinning path here.
    if let Some(mesh_handle) = mesh_instance.mesh.as_ref() {
        let mesh_id = mesh_handle.id();
        if let Some(mesh) = streamer.mesh(&mesh_id) {
            let static_state = mesh_draw_static_state(
                mesh_instance,
                mesh_id,
                streamer.mesh_revision(&mesh_id),
                material_revision,
            );
            if let Some(dynamic_primitive) = dynamic_direct_mesh_primitive(
                streamer,
                frame,
                mesh_instance,
                &mesh_id,
                mesh.clone(),
            ) {
                let previous_skinned_joint_palette = None;
                push_dynamic_mesh_draws(
                    pending_draws,
                    streamer,
                    mesh_instance.node_id,
                    &mut draw_ordinal,
                    mesh_instance.transform_revision,
                    static_state,
                    mesh_instance.material,
                    mesh_instance.mobility,
                    mesh.index_count,
                    &dynamic_primitive.primitive,
                    instance_tint,
                    model_matrix,
                    dynamic_primitive.skinned,
                    dynamic_primitive.skinned_palette_signature,
                    dynamic_primitive.skinned_joint_palette,
                    previous_skinned_joint_palette,
                    dynamic_primitive.skinned_gpu_source,
                    mesh_instance.mesh_lod,
                );
            } else {
                push_prepared_mesh_draws(
                    pending_draws,
                    streamer,
                    mesh_instance.node_id,
                    &mut draw_ordinal,
                    mesh_instance.transform_revision,
                    static_state,
                    mesh_instance.material,
                    mesh_instance.mobility,
                    mesh,
                    instance_tint,
                    model_matrix,
                    mesh_instance.mesh_lod,
                );
            }
            return;
        }
    }

    let model_id = mesh_instance.model.id();
    let Some(model) = streamer.model(&model_id) else {
        return;
    };
    let static_state = mesh_draw_static_state(
        mesh_instance,
        model_id,
        streamer.model_revision(&model_id),
        material_revision,
    );
    let previous_skinned_joint_palette = None;
    let skinned_primitives = frame
        .extract
        .animation_poses
        .iter()
        .find(|entry| entry.entity == mesh_instance.node_id)
        .and_then(|entry| {
            let skinned_palette_signature = skinned_palette_signature(entry.skeleton);
            let model_asset = streamer.load_model_asset(mesh_instance.model.id())?;
            let skeleton = streamer.load_animation_skeleton_asset(entry.skeleton)?;
            Some((
                skinned_palette_signature,
                model_asset
                    .primitives
                    .iter()
                    .map(|primitive| {
                        prepare_skinned_model_primitive(primitive, &skeleton, &entry.pose).ok()
                    })
                    .collect::<Vec<_>>(),
            ))
        });

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        if let Some(skinned_primitive) = skinned_primitives
            .as_ref()
            .and_then(|(_, primitives)| primitives.get(mesh_index))
            .and_then(|primitive| primitive.as_ref())
        {
            let skinned_palette_signature =
                skinned_primitives.as_ref().map(|(signature, _)| *signature);
            push_dynamic_mesh_draws(
                pending_draws,
                streamer,
                mesh_instance.node_id,
                &mut draw_ordinal,
                mesh_instance.transform_revision,
                static_state,
                mesh_instance.material,
                mesh_instance.mobility,
                mesh.index_count,
                &skinned_primitive.primitive,
                instance_tint,
                model_matrix,
                true,
                skinned_palette_signature,
                skinned_primitive.joint_palette_uniform,
                previous_skinned_joint_palette,
                skinned_gpu_source_candidate_available(
                    skinned_primitive.joint_palette_uniform.as_ref(),
                )
                .then_some(PendingSkinnedGpuSource::Prepared(mesh.clone())),
                mesh_instance.mesh_lod,
            );
            continue;
        }

        let raster_draws = raster_draws_for_mesh(
            mesh.index_count,
            material_tinted(streamer, mesh_instance.material, instance_tint),
        );
        if raster_draws.is_empty() {
            continue;
        }

        for (first_index, draw_index_count, draw_tint) in raster_draws {
            let material = streamer.material(&mesh_instance.material.id());
            pending_draws.push(PendingMeshDraw {
                mesh: PendingMeshGeometry::Prepared(mesh.clone()),
                source_entity: mesh_instance.node_id,
                source_draw_ordinal: next_draw_ordinal(&mut draw_ordinal),
                transform_revision: mesh_instance.transform_revision,
                mobility: mesh_instance.mobility,
                static_state,
                material_textures: material_texture_set(streamer, material),
                material_uniform: streamer.material_uniform(&mesh_instance.material.id()),
                standard_material_uniform: streamer
                    .standard_material_uniform(&mesh_instance.material.id()),
                pipeline_key: streamer
                    .material(&mesh_instance.material.id())
                    .map(|material| material.pipeline_key.clone())
                    .unwrap_or_else(default_pipeline_key),
                mesh_lod: mesh_instance.mesh_lod,
                cast_shadows: material_cast_shadows(streamer, mesh_instance.material),
                receive_shadows: material_receive_shadows(streamer, mesh_instance.material),
                taa_reactive_mask_strength: material_taa_reactive_mask_strength(material),
                model_matrix,
                draw_tint,
                skinned: false,
                skinned_palette_signature: None,
                skinned_joint_palette: None,
                previous_skinned_joint_palette: None,
                skinned_gpu_source: None,
                resolved_skinned_gpu_source: None,
                previous_skinned_gpu_source: None,
                first_index,
                draw_index_count,
                indirect_draw_ref: None,
            });
        }
    }
}

fn mesh_draw_static_state(
    mesh_instance: &RenderMeshSnapshot,
    geometry_id: ResourceId,
    geometry_revision: Option<u64>,
    material_revision: Option<u64>,
) -> RenderMeshStaticState {
    let geometry_revision =
        geometry_revision.unwrap_or(mesh_instance.static_state.geometry_revision);
    let material_revision =
        material_revision.unwrap_or(mesh_instance.static_state.material_revision);
    RenderMeshStaticState::new(
        mesh_instance.static_state.transform_static && mesh_instance.mobility == Mobility::Static,
        geometry_revision_signature(geometry_id, geometry_revision, mesh_instance.mesh_lod),
        resource_revision_signature(mesh_instance.material.id(), material_revision),
    )
}

fn geometry_revision_signature(
    resource_id: ResourceId,
    revision: u64,
    mesh_lod: Option<RenderMeshLodSelection>,
) -> u64 {
    if revision == 0 {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    resource_id.hash(&mut hasher);
    revision.hash(&mut hasher);
    if let Some(mesh_lod) = mesh_lod {
        mesh_lod.level_index.hash(&mut hasher);
        mesh_lod.min_distance.to_bits().hash(&mut hasher);
    }
    nonzero_hash(hasher)
}

fn resource_revision_signature(resource_id: ResourceId, revision: u64) -> u64 {
    if revision == 0 {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    resource_id.hash(&mut hasher);
    revision.hash(&mut hasher);
    nonzero_hash(hasher)
}

fn nonzero_hash(hasher: DefaultHasher) -> u64 {
    let signature = hasher.finish();
    if signature == 0 {
        1
    } else {
        signature
    }
}

fn next_draw_ordinal(draw_ordinal: &mut u32) -> u32 {
    let current = *draw_ordinal;
    *draw_ordinal = draw_ordinal.saturating_add(1);
    current
}

fn material_tinted(
    streamer: &ResourceStreamer,
    material: ResourceHandle<MaterialMarker>,
    instance_tint: Vec4,
) -> Vec4 {
    let material_tint = streamer
        .material(&material.id())
        .map(|material| material.base_color)
        .unwrap_or(Vec4::ONE);
    instance_tint * material_tint
}

fn material_receive_shadows(
    streamer: &ResourceStreamer,
    material: ResourceHandle<MaterialMarker>,
) -> bool {
    streamer
        .material(&material.id())
        .map(|material| material.receive_shadows)
        .unwrap_or(true)
}

fn material_cast_shadows(
    streamer: &ResourceStreamer,
    material: ResourceHandle<MaterialMarker>,
) -> bool {
    streamer
        .material(&material.id())
        .map(|material| material.cast_shadows)
        .unwrap_or(true)
}

fn material_taa_reactive_mask_strength(material: Option<&MaterialRuntime>) -> f32 {
    material
        .map(|material| material.taa_reactive_mask_strength)
        .filter(|strength| strength.is_finite())
        .unwrap_or_default()
        .clamp(0.0, 1.0)
}

fn material_texture_set(
    streamer: &ResourceStreamer,
    material: Option<&MaterialRuntime>,
) -> MaterialTextureSet {
    MaterialTextureSet::new(
        streamer.texture(material.and_then(|material| material.base_color_texture)),
        streamer.normal_texture(material.and_then(|material| material.normal_texture)),
        streamer.texture(material.and_then(|material| material.metallic_roughness_texture)),
        streamer.texture(material.and_then(|material| material.occlusion_texture)),
        streamer.texture(material.and_then(|material| material.emissive_texture)),
    )
}

fn dynamic_direct_mesh_primitive(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    mesh_instance: &RenderMeshSnapshot,
    mesh_id: &ResourceId,
    prepared_mesh: Arc<GpuMeshResource>,
) -> Option<DynamicMeshPrimitive> {
    skinned_direct_mesh_primitive(streamer, frame, mesh_instance, mesh_id)
        .map(
            |(prepared, skinned_palette_signature)| DynamicMeshPrimitive {
                primitive: prepared.primitive,
                skinned: true,
                skinned_palette_signature: Some(skinned_palette_signature),
                skinned_joint_palette: prepared.joint_palette_uniform,
                skinned_gpu_source: direct_skinned_gpu_source(
                    prepared.joint_palette_uniform.as_ref(),
                    *mesh_id,
                    prepared_mesh,
                    prepared.shader_skinning_source_primitive,
                    &mesh_instance.morph_weights,
                ),
            },
        )
        .or_else(|| {
            morphed_direct_mesh_primitive(streamer, mesh_instance, mesh_id).map(|primitive| {
                DynamicMeshPrimitive {
                    primitive,
                    skinned: false,
                    skinned_palette_signature: None,
                    skinned_joint_palette: None,
                    skinned_gpu_source: None,
                }
            })
        })
}

fn skinned_direct_mesh_primitive(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    mesh_instance: &RenderMeshSnapshot,
    mesh_id: &ResourceId,
) -> Option<(SkinnedMeshPreparedPrimitive, u64)> {
    let pose_entry = frame
        .extract
        .animation_poses
        .iter()
        .find(|entry| entry.entity == mesh_instance.node_id)?;
    let skinned_palette_signature = skinned_palette_signature(pose_entry.skeleton);
    let mesh_asset = streamer.mesh_asset(mesh_id)?;
    let skeleton = streamer.load_animation_skeleton_asset(pose_entry.skeleton)?;
    prepare_skinned_mesh_asset_primitive(
        mesh_asset,
        &skeleton,
        &pose_entry.pose,
        &mesh_instance.morph_weights,
    )
    .ok()
    .map(|prepared| (prepared, skinned_palette_signature))
}

fn morphed_direct_mesh_primitive(
    streamer: &ResourceStreamer,
    mesh_instance: &RenderMeshSnapshot,
    mesh_id: &ResourceId,
) -> Option<ModelPrimitiveAsset> {
    let mesh_asset = streamer.mesh_asset(mesh_id)?;
    morphed_mesh_asset_primitive(mesh_asset.as_ref(), &mesh_instance.morph_weights)
}

fn morphed_mesh_asset_primitive(
    mesh_asset: &MeshAsset,
    morph_weights: &[f32],
) -> Option<ModelPrimitiveAsset> {
    if !has_active_morph_weights(morph_weights) {
        return None;
    }
    mesh_asset.to_morphed_model_primitive(morph_weights).ok()
}

fn has_active_morph_weights(morph_weights: &[f32]) -> bool {
    morph_weights
        .iter()
        .any(|weight| weight.abs() > f32::EPSILON)
}

fn skinned_palette_signature(skeleton_id: ResourceId) -> u64 {
    let mut hasher = DefaultHasher::new();
    skeleton_id.hash(&mut hasher);
    nonzero_hash(hasher)
}

fn morph_shape_signature(mesh_id: ResourceId, morph_weights: &[f32]) -> u64 {
    let mut hasher = DefaultHasher::new();
    mesh_id.hash(&mut hasher);
    morph_weights.len().hash(&mut hasher);
    for weight in morph_weights {
        weight.to_bits().hash(&mut hasher);
    }
    nonzero_hash(hasher)
}

fn skinned_gpu_source_candidate_available(
    joint_palette_uniform: Option<&SkinnedMeshJointPaletteUniform>,
) -> bool {
    joint_palette_uniform.is_some()
}

fn direct_skinned_gpu_source(
    joint_palette_uniform: Option<&SkinnedMeshJointPaletteUniform>,
    mesh_id: ResourceId,
    prepared_mesh: Arc<GpuMeshResource>,
    shader_skinning_source_primitive: ModelPrimitiveAsset,
    morph_weights: &[f32],
) -> Option<PendingSkinnedGpuSource> {
    skinned_gpu_source_candidate_available(joint_palette_uniform).then(|| {
        if has_active_morph_weights(morph_weights) {
            PendingSkinnedGpuSource::CpuMorphed {
                primitive: shader_skinning_source_primitive,
                morph_shape_signature: morph_shape_signature(mesh_id, morph_weights),
            }
        } else {
            PendingSkinnedGpuSource::Prepared(prepared_mesh)
        }
    })
}

fn push_dynamic_mesh_draws(
    pending_draws: &mut Vec<PendingMeshDraw>,
    streamer: &ResourceStreamer,
    source_entity: EntityId,
    draw_ordinal: &mut u32,
    transform_revision: u64,
    static_state: RenderMeshStaticState,
    material_id: ResourceHandle<MaterialMarker>,
    mobility: Mobility,
    index_count: u32,
    dynamic_primitive: &ModelPrimitiveAsset,
    instance_tint: Vec4,
    model_matrix: [[f32; 4]; 4],
    skinned: bool,
    skinned_palette_signature: Option<u64>,
    skinned_joint_palette: Option<SkinnedMeshJointPaletteUniform>,
    previous_skinned_joint_palette: Option<SkinnedMeshJointPaletteUniform>,
    skinned_gpu_source: Option<PendingSkinnedGpuSource>,
    mesh_lod: Option<RenderMeshLodSelection>,
) {
    let material = streamer.material(&material_id.id());
    let material_textures = material_texture_set(streamer, material);
    let material_uniform = streamer.material_uniform(&material_id.id());
    let standard_material_uniform = streamer.standard_material_uniform(&material_id.id());
    let pipeline_key = material
        .map(|material| material.pipeline_key.clone())
        .unwrap_or_else(default_pipeline_key);
    for (first_index, draw_index_count, draw_tint) in raster_draws_for_mesh(
        index_count,
        material_tinted(streamer, material_id, instance_tint),
    ) {
        pending_draws.push(PendingMeshDraw {
            mesh: PendingMeshGeometry::Dynamic(dynamic_primitive.clone()),
            source_entity,
            source_draw_ordinal: next_draw_ordinal(draw_ordinal),
            transform_revision,
            mobility,
            static_state,
            material_textures: material_textures.clone(),
            material_uniform: material_uniform.clone(),
            standard_material_uniform: standard_material_uniform.clone(),
            pipeline_key: pipeline_key.clone(),
            mesh_lod,
            cast_shadows: material
                .map(|material| material.cast_shadows)
                .unwrap_or(true),
            receive_shadows: material
                .map(|material| material.receive_shadows)
                .unwrap_or(true),
            taa_reactive_mask_strength: material_taa_reactive_mask_strength(material),
            model_matrix,
            draw_tint,
            skinned,
            skinned_palette_signature,
            skinned_joint_palette,
            previous_skinned_joint_palette,
            skinned_gpu_source: skinned_gpu_source.clone(),
            resolved_skinned_gpu_source: None,
            previous_skinned_gpu_source: None,
            first_index,
            draw_index_count,
            indirect_draw_ref: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        morph_shape_signature, morphed_mesh_asset_primitive, skinned_gpu_source_candidate_available,
    };
    use crate::asset::{
        AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MeshMorphTargetAsset,
        MESH_ATTRIBUTE_POSITION,
    };
    use crate::core::framework::render::RenderMeshTopology;
    use crate::core::math::Vec3;
    use crate::graphics::scene::scene_renderer::mesh::skinning::SkinnedMeshJointPaletteUniform;

    #[test]
    fn morphed_mesh_asset_primitive_ignores_zero_weights_for_static_direct_mesh_fallback() {
        let mesh = morph_test_mesh();

        assert!(morphed_mesh_asset_primitive(&mesh, &[0.0]).is_none());
    }

    #[test]
    fn morphed_mesh_asset_primitive_applies_nonzero_weights_for_dynamic_direct_mesh() {
        let mesh = morph_test_mesh();

        let primitive = morphed_mesh_asset_primitive(&mesh, &[0.5]).expect("morphed primitive");

        assert!(Vec3::from_array(primitive.vertices[0].position)
            .abs_diff_eq(Vec3::new(1.0, 0.0, 0.5), 1.0e-6));
        assert_eq!(primitive.indices, vec![0, 1, 2]);
    }

    #[test]
    fn morph_shape_signature_tracks_mesh_and_weights() {
        let mesh_a = crate::core::resource::ResourceId::from_stable_label("mesh-a");
        let mesh_b = crate::core::resource::ResourceId::from_stable_label("mesh-b");
        let first = morph_shape_signature(mesh_a, &[0.25, 0.0]);

        assert_eq!(first, morph_shape_signature(mesh_a, &[0.25, 0.0]));
        assert_ne!(first, morph_shape_signature(mesh_a, &[0.5, 0.0]));
        assert_ne!(first, morph_shape_signature(mesh_b, &[0.25, 0.0]));
    }

    #[test]
    fn skinned_gpu_source_candidate_requires_palette() {
        let uniform = SkinnedMeshJointPaletteUniform::from_matrices(&[])
            .expect("empty palette should fit the fixed skinned ABI");

        assert!(skinned_gpu_source_candidate_available(Some(&uniform)));
        assert!(
            !skinned_gpu_source_candidate_available(None),
            "a source mesh is not enough without a shader-visible palette"
        );
    }

    fn morph_test_mesh() -> MeshAsset {
        let mut mesh = MeshAsset::new(
            AssetUri::parse("res://meshes/direct-morph.zmesh").unwrap(),
            RenderMeshTopology::TriangleList,
            BTreeMap::from([(
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [1.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                ]),
            )]),
            Some(MeshIndices::U32(vec![0, 1, 2])),
        )
        .unwrap();
        mesh.morph_targets = vec![MeshMorphTargetAsset {
            name: Some("Lift".to_string()),
            attributes: BTreeMap::from([(
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
            )]),
        }];
        mesh
    }
}

fn push_prepared_mesh_draws(
    pending_draws: &mut Vec<PendingMeshDraw>,
    streamer: &ResourceStreamer,
    source_entity: EntityId,
    draw_ordinal: &mut u32,
    transform_revision: u64,
    static_state: RenderMeshStaticState,
    material_id: ResourceHandle<MaterialMarker>,
    mobility: Mobility,
    mesh: &Arc<GpuMeshResource>,
    instance_tint: Vec4,
    model_matrix: [[f32; 4]; 4],
    mesh_lod: Option<RenderMeshLodSelection>,
) {
    let material = streamer.material(&material_id.id());
    let material_textures = material_texture_set(streamer, material);
    let material_uniform = streamer.material_uniform(&material_id.id());
    let standard_material_uniform = streamer.standard_material_uniform(&material_id.id());
    let pipeline_key = material
        .map(|material| material.pipeline_key.clone())
        .unwrap_or_else(default_pipeline_key);
    for (first_index, draw_index_count, draw_tint) in raster_draws_for_mesh(
        mesh.index_count,
        material_tinted(streamer, material_id, instance_tint),
    ) {
        pending_draws.push(PendingMeshDraw {
            mesh: PendingMeshGeometry::Prepared(mesh.clone()),
            source_entity,
            source_draw_ordinal: next_draw_ordinal(draw_ordinal),
            transform_revision,
            mobility,
            static_state,
            material_textures: material_textures.clone(),
            material_uniform: material_uniform.clone(),
            standard_material_uniform: standard_material_uniform.clone(),
            pipeline_key: pipeline_key.clone(),
            mesh_lod,
            cast_shadows: material
                .map(|material| material.cast_shadows)
                .unwrap_or(true),
            receive_shadows: material
                .map(|material| material.receive_shadows)
                .unwrap_or(true),
            taa_reactive_mask_strength: material_taa_reactive_mask_strength(material),
            model_matrix,
            draw_tint,
            skinned: false,
            skinned_palette_signature: None,
            skinned_joint_palette: None,
            previous_skinned_joint_palette: None,
            skinned_gpu_source: None,
            resolved_skinned_gpu_source: None,
            previous_skinned_gpu_source: None,
            first_index,
            draw_index_count,
            indirect_draw_ref: None,
        });
    }
}
