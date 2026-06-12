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
    prepare_skinned_mesh_asset_primitive, prepare_skinned_model_primitive, SkinnedMeshJointPalette,
    SkinnedMeshPreparedPrimitive,
};

struct DynamicMeshPrimitive {
    primitive: ModelPrimitiveAsset,
    skinned: bool,
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
    let (previous_model_matrix, has_previous_motion_vector_transform) =
        previous_motion_model_matrix(frame, mesh_instance.node_id, model_matrix);
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
                let shared_morphed_source_available = matches!(
                    dynamic_primitive.skinned_gpu_source.as_ref(),
                    Some(PendingSkinnedGpuSource::CpuMorphed(_))
                );
                let previous_skinned_joint_palette = if dynamic_primitive.skinned {
                    previous_skinned_joint_palette(
                        frame,
                        streamer,
                        mesh_instance,
                        shared_morphed_source_available,
                    )
                } else {
                    None
                };
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
                    previous_model_matrix,
                    has_previous_motion_vector_transform,
                    dynamic_primitive.skinned,
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
                    previous_model_matrix,
                    has_previous_motion_vector_transform,
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
    let previous_skinned_joint_palette =
        previous_skinned_joint_palette(frame, streamer, mesh_instance, false);
    let skinned_primitives = frame
        .extract
        .animation_poses
        .iter()
        .find(|entry| entry.entity == mesh_instance.node_id)
        .and_then(|entry| {
            let model_asset = streamer.load_model_asset(mesh_instance.model.id())?;
            let skeleton = streamer.load_animation_skeleton_asset(entry.skeleton)?;
            Some(
                model_asset
                    .primitives
                    .iter()
                    .map(|primitive| {
                        prepare_skinned_model_primitive(primitive, &skeleton, &entry.pose).ok()
                    })
                    .collect::<Vec<_>>(),
            )
        });

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        if let Some(skinned_primitive) = skinned_primitives
            .as_ref()
            .and_then(|primitives| primitives.get(mesh_index))
            .and_then(|primitive| primitive.as_ref())
        {
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
                previous_model_matrix,
                has_previous_motion_vector_transform,
                true,
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
                model_matrix,
                previous_model_matrix,
                has_previous_motion_vector_transform,
                draw_tint,
                skinned: false,
                skinned_joint_palette: None,
                previous_skinned_joint_palette: None,
                skinned_gpu_source: None,
                first_index,
                draw_index_count,
                indirect_draw_ref: None,
            });
        }
    }
}

fn previous_motion_model_matrix(
    frame: &ViewportRenderFrame,
    entity: EntityId,
    fallback_model_matrix: [[f32; 4]; 4],
) -> ([[f32; 4]; 4], bool) {
    let Some(previous_transform) = frame
        .previous_motion_vector_object_history()
        .and_then(|history| history.transform(entity))
    else {
        return (fallback_model_matrix, false);
    };
    (
        render_mat4_or(previous_transform.matrix(), RenderMat4::IDENTITY).to_cols_array_2d(),
        true,
    )
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

fn previous_skinned_joint_palette(
    frame: &ViewportRenderFrame,
    streamer: &ResourceStreamer,
    mesh_instance: &RenderMeshSnapshot,
    shared_morphed_source_available: bool,
) -> Option<SkinnedMeshJointPaletteUniform> {
    let current_pose = frame
        .extract
        .animation_poses
        .iter()
        .find(|entry| entry.entity == mesh_instance.node_id)?;
    let previous_pose = frame
        .previous_motion_vector_object_history()
        .and_then(|history| history.skinned_pose(mesh_instance.node_id))?;
    if previous_pose.skeleton() != current_pose.skeleton {
        return None;
    }
    if !morph_weights_support_previous_palette(
        &mesh_instance.morph_weights,
        previous_pose.morph_weights(),
        shared_morphed_source_available,
    ) {
        return None;
    }

    let skeleton = streamer.load_animation_skeleton_asset(previous_pose.skeleton())?;
    SkinnedMeshJointPalette::from_skeleton_pose(&skeleton, previous_pose.pose())
        .ok()?
        .to_uniform()
        .ok()
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
        .map(|prepared| DynamicMeshPrimitive {
            primitive: prepared.primitive,
            skinned: true,
            skinned_joint_palette: prepared.joint_palette_uniform,
            skinned_gpu_source: direct_skinned_gpu_source(
                prepared.joint_palette_uniform.as_ref(),
                prepared_mesh,
                prepared.shader_skinning_source_primitive,
                &mesh_instance.morph_weights,
            ),
        })
        .or_else(|| {
            morphed_direct_mesh_primitive(streamer, mesh_instance, mesh_id).map(|primitive| {
                DynamicMeshPrimitive {
                    primitive,
                    skinned: false,
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
) -> Option<SkinnedMeshPreparedPrimitive> {
    let pose_entry = frame
        .extract
        .animation_poses
        .iter()
        .find(|entry| entry.entity == mesh_instance.node_id)?;
    let mesh_asset = streamer.mesh_asset(mesh_id)?;
    let skeleton = streamer.load_animation_skeleton_asset(pose_entry.skeleton)?;
    prepare_skinned_mesh_asset_primitive(
        mesh_asset,
        &skeleton,
        &pose_entry.pose,
        &mesh_instance.morph_weights,
    )
    .ok()
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

fn morph_weights_support_previous_palette(
    current_morph_weights: &[f32],
    previous_morph_weights: &[f32],
    shared_morphed_source_available: bool,
) -> bool {
    if !morph_weights_are_finite(current_morph_weights)
        || !morph_weights_are_finite(previous_morph_weights)
    {
        return false;
    }
    if !has_active_morph_weights(current_morph_weights)
        && !has_active_morph_weights(previous_morph_weights)
    {
        return true;
    }
    shared_morphed_source_available
        && morph_weights_match_for_shared_source(current_morph_weights, previous_morph_weights)
}

fn morph_weights_are_finite(morph_weights: &[f32]) -> bool {
    morph_weights.iter().all(|weight| weight.is_finite())
}

fn morph_weights_match_for_shared_source(
    current_morph_weights: &[f32],
    previous_morph_weights: &[f32],
) -> bool {
    let weight_count = current_morph_weights
        .len()
        .max(previous_morph_weights.len());
    (0..weight_count).all(|index| {
        let current_weight = morph_weight_or_zero(current_morph_weights, index);
        let previous_weight = morph_weight_or_zero(previous_morph_weights, index);
        (current_weight - previous_weight).abs() <= f32::EPSILON
    })
}

fn morph_weight_or_zero(morph_weights: &[f32], index: usize) -> f32 {
    morph_weights.get(index).copied().unwrap_or(0.0)
}

fn skinned_gpu_source_candidate_available(
    joint_palette_uniform: Option<&SkinnedMeshJointPaletteUniform>,
) -> bool {
    joint_palette_uniform.is_some()
}

fn direct_skinned_gpu_source(
    joint_palette_uniform: Option<&SkinnedMeshJointPaletteUniform>,
    prepared_mesh: Arc<GpuMeshResource>,
    shader_skinning_source_primitive: ModelPrimitiveAsset,
    morph_weights: &[f32],
) -> Option<PendingSkinnedGpuSource> {
    skinned_gpu_source_candidate_available(joint_palette_uniform).then(|| {
        if has_active_morph_weights(morph_weights) {
            PendingSkinnedGpuSource::CpuMorphed(shader_skinning_source_primitive)
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
    previous_model_matrix: [[f32; 4]; 4],
    has_previous_motion_vector_transform: bool,
    skinned: bool,
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
            model_matrix,
            previous_model_matrix,
            has_previous_motion_vector_transform,
            draw_tint,
            skinned,
            skinned_joint_palette,
            previous_skinned_joint_palette,
            skinned_gpu_source: skinned_gpu_source.clone(),
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
        morph_weights_support_previous_palette, morphed_mesh_asset_primitive,
        skinned_gpu_source_candidate_available,
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
    fn skinned_gpu_source_candidate_requires_palette() {
        let uniform = SkinnedMeshJointPaletteUniform::from_matrices(&[])
            .expect("empty palette should fit the fixed skinned ABI");

        assert!(skinned_gpu_source_candidate_available(Some(&uniform)));
        assert!(
            !skinned_gpu_source_candidate_available(None),
            "a source mesh is not enough without a shader-visible palette"
        );
    }

    #[test]
    fn previous_palette_morph_weights_accept_inactive_weights() {
        assert!(morph_weights_support_previous_palette(&[], &[0.0], false));
    }

    #[test]
    fn previous_palette_morph_weights_accept_matching_active_shared_source_weights() {
        assert!(morph_weights_support_previous_palette(
            &[0.25, 0.0],
            &[0.25],
            true
        ));
    }

    #[test]
    fn previous_palette_morph_weights_reject_active_weights_without_shared_source() {
        assert!(!morph_weights_support_previous_palette(
            &[0.25],
            &[0.25],
            false
        ));
    }

    #[test]
    fn previous_palette_morph_weights_reject_changed_active_weights() {
        assert!(!morph_weights_support_previous_palette(
            &[0.25],
            &[0.5],
            true
        ));
    }

    #[test]
    fn previous_palette_morph_weights_reject_non_finite_weights() {
        assert!(!morph_weights_support_previous_palette(
            &[f32::NAN],
            &[],
            true
        ));
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
    previous_model_matrix: [[f32; 4]; 4],
    has_previous_motion_vector_transform: bool,
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
            model_matrix,
            previous_model_matrix,
            has_previous_motion_vector_transform,
            draw_tint,
            skinned: false,
            skinned_joint_palette: None,
            previous_skinned_joint_palette: None,
            skinned_gpu_source: None,
            first_index,
            draw_index_count,
            indirect_draw_ref: None,
        });
    }
}
