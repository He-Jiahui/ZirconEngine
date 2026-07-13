use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::asset::{MeshAsset, ModelPrimitiveAsset};
use crate::core::framework::render::{
    render_mesh_stable_instance_key, DisplayMode, RenderMaterialPropertyUniformPayload,
    RenderMeshLodSelection, RenderMeshSnapshot, RenderMeshStaticState,
};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::{RenderMat4, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId};
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::scene_renderer::mesh::skinning::SkinnedMeshJointPaletteStorage;

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::super::resources::{
    default_pipeline_key, GpuMeshResource, ResourceStreamer,
};
use super::super::super::super::primitives::render_mat4_or;
use super::super::super::mesh_draw::MeshCommandSortInput;
use super::super::raster_draws_for_mesh::raster_draws_for_mesh;
use super::mesh_draw_build_context::MeshDrawBuildContext;
use super::morph_payload_upload::morph_payload_from_mesh_asset;
use super::pending_mesh_draw::{PendingMeshDraw, PendingMeshGeometry, PendingSkinnedGpuSource};
use super::skinning::{
    prepare_skinned_mesh_asset_primitive, prepare_skinned_model_primitive,
    SkinnedMeshPreparedPrimitive,
};

mod material_inputs;

use self::material_inputs::{
    material_cast_shadows, material_disabled_passes, material_receive_shadows,
    material_taa_reactive_mask_strength, material_texture_set, material_tinted,
};

struct DynamicMeshPrimitive {
    primitive: ModelPrimitiveAsset,
    cpu_morphed: bool,
    morph_payload: Option<Arc<super::pending_mesh_draw::PendingMorphPayload>>,
    source_morph_weights: Option<Vec<f32>>,
    skinned: bool,
    skinned_palette_signature: Option<u64>,
    skinned_joint_palette: Option<SkinnedMeshJointPaletteStorage>,
    skinned_gpu_source: Option<PendingSkinnedGpuSource>,
    gpu_morphed_source: Option<Arc<GpuMeshResource>>,
}

pub(super) fn extend_pending_draws_for_mesh_instance(
    pending_draws: &mut Vec<PendingMeshDraw>,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    build_context: &MeshDrawBuildContext,
    gpu_scene: &GpuScene,
    mesh_instance: &RenderMeshSnapshot,
    command_sort_input: MeshCommandSortInput,
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
    let material_uniform_override_payload = frame
        .extract
        .geometry
        .material_property_overrides
        .get(&mesh_instance.node_id)
        .and_then(|overrides| {
            streamer
                .material_uniform_payload_with_overrides(&mesh_instance.material.id(), overrides)
        });
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
                gpu_scene,
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
                    material_uniform_override_payload.clone(),
                    mesh_instance.mobility,
                    command_sort_input,
                    mesh.index_count,
                    &dynamic_primitive.primitive,
                    instance_tint,
                    model_matrix,
                    dynamic_primitive.cpu_morphed,
                    dynamic_primitive.morph_payload,
                    dynamic_primitive.source_morph_weights.clone(),
                    dynamic_primitive.skinned,
                    dynamic_primitive.skinned_palette_signature,
                    dynamic_primitive.skinned_joint_palette,
                    previous_skinned_joint_palette,
                    dynamic_primitive.skinned_gpu_source,
                    dynamic_primitive.gpu_morphed_source,
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
                    material_uniform_override_payload.clone(),
                    mesh_instance.mobility,
                    command_sort_input,
                    mesh,
                    instance_tint,
                    model_matrix,
                    direct_mesh_source_morph_weights(streamer, &mesh_id, mesh_instance),
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
                material_uniform_override_payload.clone(),
                mesh_instance.mobility,
                command_sort_input,
                mesh.index_count,
                &skinned_primitive.primitive,
                instance_tint,
                model_matrix,
                false,
                None,
                None,
                true,
                skinned_palette_signature,
                skinned_primitive.joint_palette_storage,
                previous_skinned_joint_palette,
                skinned_gpu_source_candidate_available(
                    skinned_primitive.joint_palette_storage.as_ref(),
                )
                .then_some(PendingSkinnedGpuSource::Prepared(mesh.clone())),
                None,
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
            let source_draw_ordinal = next_draw_ordinal(&mut draw_ordinal);
            pending_draws.push(PendingMeshDraw {
                mesh: PendingMeshGeometry::Prepared(mesh.clone()),
                source_entity: mesh_instance.node_id,
                source_draw_ordinal,
                transform_revision: mesh_instance.transform_revision,
                mobility: mesh_instance.mobility,
                static_state,
                material_textures: material_texture_set(streamer, material),
                material_uniform: streamer.material_uniform(&mesh_instance.material.id()),
                material_uniform_override_payload: material_uniform_override_payload.clone(),
                standard_material_uniform: streamer
                    .standard_material_uniform(&mesh_instance.material.id()),
                pipeline_key: streamer
                    .material(&mesh_instance.material.id())
                    .map(|material| material.pipeline_key.clone())
                    .unwrap_or_else(default_pipeline_key),
                morph_payload: None,
                source_morph_weights: None,
                morph_payload_slot: None,
                mesh_lod: mesh_instance.mesh_lod,
                cast_shadows: material_cast_shadows(streamer, mesh_instance.material),
                receive_shadows: material_receive_shadows(streamer, mesh_instance.material),
                disabled_passes: material_disabled_passes(material),
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
                command_sort_input: command_sort_input.with_tie_breaker(
                    mesh_order_command_sort_tie_breaker(
                        mesh_instance.node_id,
                        source_draw_ordinal,
                        mesh.indirect_order_signature(),
                    ),
                ),
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

fn dynamic_direct_mesh_primitive(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    gpu_scene: &GpuScene,
    mesh_instance: &RenderMeshSnapshot,
    mesh_id: &ResourceId,
    prepared_mesh: Arc<GpuMeshResource>,
) -> Option<DynamicMeshPrimitive> {
    let previous_morph_weights =
        gpu_scene.previous_morph_weights(render_mesh_stable_instance_key(mesh_instance.node_id, 0));
    let source_morph_weights = direct_mesh_source_morph_weights(streamer, mesh_id, mesh_instance);
    if let Some((prepared, skinned_palette_signature)) =
        skinned_direct_mesh_primitive(streamer, frame, mesh_instance, mesh_id)
    {
        let morph_payload =
            direct_mesh_morph_payload(streamer, mesh_id, mesh_instance, previous_morph_weights);
        let morph_payload_available = morph_payload.is_some();
        return Some(DynamicMeshPrimitive {
            primitive: prepared.primitive,
            cpu_morphed: has_active_morph_weights(&mesh_instance.morph_weights),
            morph_payload,
            source_morph_weights,
            skinned: true,
            skinned_palette_signature: Some(skinned_palette_signature),
            skinned_joint_palette: prepared.joint_palette_storage,
            skinned_gpu_source: direct_skinned_gpu_source(
                prepared.joint_palette_storage.as_ref(),
                *mesh_id,
                prepared_mesh.clone(),
                prepared.shader_skinning_source_primitive,
                morph_payload_available,
                &mesh_instance.morph_weights,
            ),
            gpu_morphed_source: None,
        });
    }

    let morph_payload =
        direct_mesh_morph_payload(streamer, mesh_id, mesh_instance, previous_morph_weights);
    if let Some(primitive) = morphed_direct_mesh_primitive(streamer, mesh_instance, mesh_id) {
        let gpu_morphed_source = morph_payload.as_ref().map(|_| prepared_mesh.clone());
        return Some(DynamicMeshPrimitive {
            primitive,
            cpu_morphed: true,
            morph_payload,
            source_morph_weights,
            skinned: false,
            skinned_palette_signature: None,
            skinned_joint_palette: None,
            skinned_gpu_source: None,
            gpu_morphed_source,
        });
    }

    morph_payload.and_then(|payload| {
        let primitive = streamer.mesh_asset(mesh_id)?.to_model_primitive().ok()?;
        Some(DynamicMeshPrimitive {
            primitive,
            cpu_morphed: false,
            morph_payload: Some(payload),
            source_morph_weights,
            skinned: false,
            skinned_palette_signature: None,
            skinned_joint_palette: None,
            skinned_gpu_source: None,
            gpu_morphed_source: Some(prepared_mesh),
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

fn direct_mesh_morph_payload(
    streamer: &ResourceStreamer,
    mesh_id: &ResourceId,
    mesh_instance: &RenderMeshSnapshot,
    previous_morph_weights: Option<&[f32]>,
) -> Option<Arc<super::pending_mesh_draw::PendingMorphPayload>> {
    let mesh_asset = streamer.mesh_asset(mesh_id)?;
    morph_payload_from_mesh_asset(
        mesh_asset.as_ref(),
        &mesh_instance.morph_weights,
        previous_morph_weights,
    )
}

fn direct_mesh_source_morph_weights(
    streamer: &ResourceStreamer,
    mesh_id: &ResourceId,
    mesh_instance: &RenderMeshSnapshot,
) -> Option<Vec<f32>> {
    let mesh_asset = streamer.mesh_asset(mesh_id)?;
    if mesh_asset.morph_targets.is_empty() {
        return None;
    }
    source_morph_weights(&mesh_instance.morph_weights)
}

fn source_morph_weights(weights: &[f32]) -> Option<Vec<f32>> {
    (!weights.is_empty()).then(|| weights.to_vec())
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
    joint_palette_storage: Option<&SkinnedMeshJointPaletteStorage>,
) -> bool {
    joint_palette_storage.is_some()
}

fn direct_skinned_gpu_source(
    joint_palette_storage: Option<&SkinnedMeshJointPaletteStorage>,
    mesh_id: ResourceId,
    prepared_mesh: Arc<GpuMeshResource>,
    shader_skinning_source_primitive: ModelPrimitiveAsset,
    morph_payload_available: bool,
    morph_weights: &[f32],
) -> Option<PendingSkinnedGpuSource> {
    skinned_gpu_source_candidate_available(joint_palette_storage).then(|| {
        if has_active_morph_weights(morph_weights) && !morph_payload_available {
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
    material_uniform_override_payload: Option<RenderMaterialPropertyUniformPayload>,
    mobility: Mobility,
    command_sort_input: MeshCommandSortInput,
    index_count: u32,
    dynamic_primitive: &ModelPrimitiveAsset,
    instance_tint: Vec4,
    model_matrix: [[f32; 4]; 4],
    cpu_morphed: bool,
    morph_payload: Option<Arc<super::pending_mesh_draw::PendingMorphPayload>>,
    source_morph_weights: Option<Vec<f32>>,
    skinned: bool,
    skinned_palette_signature: Option<u64>,
    skinned_joint_palette: Option<SkinnedMeshJointPaletteStorage>,
    previous_skinned_joint_palette: Option<SkinnedMeshJointPaletteStorage>,
    skinned_gpu_source: Option<PendingSkinnedGpuSource>,
    gpu_morphed_source: Option<Arc<GpuMeshResource>>,
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
        let source_draw_ordinal = next_draw_ordinal(draw_ordinal);
        let mesh = if let Some(gpu_morphed_source) = gpu_morphed_source.clone() {
            PendingMeshGeometry::GpuMorphed(gpu_morphed_source)
        } else if cpu_morphed {
            PendingMeshGeometry::CpuMorphed(dynamic_primitive.clone())
        } else {
            PendingMeshGeometry::Dynamic(dynamic_primitive.clone())
        };
        pending_draws.push(PendingMeshDraw {
            mesh,
            source_entity,
            source_draw_ordinal,
            transform_revision,
            mobility,
            static_state,
            material_textures: material_textures.clone(),
            material_uniform: material_uniform.clone(),
            material_uniform_override_payload: material_uniform_override_payload.clone(),
            standard_material_uniform: standard_material_uniform.clone(),
            pipeline_key: pipeline_key.clone(),
            morph_payload: morph_payload.clone(),
            source_morph_weights: source_morph_weights.clone(),
            morph_payload_slot: None,
            mesh_lod,
            cast_shadows: material
                .map(|material| material.cast_shadows)
                .unwrap_or(true),
            receive_shadows: material
                .map(|material| material.receive_shadows)
                .unwrap_or(true),
            disabled_passes: material_disabled_passes(material),
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
            command_sort_input: command_sort_input.with_tie_breaker(
                dynamic_command_sort_tie_breaker(source_entity, source_draw_ordinal),
            ),
            first_index,
            draw_index_count,
            indirect_draw_ref: None,
        });
    }
}

#[cfg(test)]
mod tests;

fn push_prepared_mesh_draws(
    pending_draws: &mut Vec<PendingMeshDraw>,
    streamer: &ResourceStreamer,
    source_entity: EntityId,
    draw_ordinal: &mut u32,
    transform_revision: u64,
    static_state: RenderMeshStaticState,
    material_id: ResourceHandle<MaterialMarker>,
    material_uniform_override_payload: Option<RenderMaterialPropertyUniformPayload>,
    mobility: Mobility,
    command_sort_input: MeshCommandSortInput,
    mesh: &Arc<GpuMeshResource>,
    instance_tint: Vec4,
    model_matrix: [[f32; 4]; 4],
    source_morph_weights: Option<Vec<f32>>,
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
        let source_draw_ordinal = next_draw_ordinal(draw_ordinal);
        pending_draws.push(PendingMeshDraw {
            mesh: PendingMeshGeometry::Prepared(mesh.clone()),
            source_entity,
            source_draw_ordinal,
            transform_revision,
            mobility,
            static_state,
            material_textures: material_textures.clone(),
            material_uniform: material_uniform.clone(),
            material_uniform_override_payload: material_uniform_override_payload.clone(),
            standard_material_uniform: standard_material_uniform.clone(),
            pipeline_key: pipeline_key.clone(),
            morph_payload: None,
            source_morph_weights: source_morph_weights.clone(),
            morph_payload_slot: None,
            mesh_lod,
            cast_shadows: material
                .map(|material| material.cast_shadows)
                .unwrap_or(true),
            receive_shadows: material
                .map(|material| material.receive_shadows)
                .unwrap_or(true),
            disabled_passes: material_disabled_passes(material),
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
            command_sort_input: command_sort_input.with_tie_breaker(
                mesh_order_command_sort_tie_breaker(
                    source_entity,
                    source_draw_ordinal,
                    mesh.indirect_order_signature(),
                ),
            ),
            first_index,
            draw_index_count,
            indirect_draw_ref: None,
        });
    }
}

fn mesh_order_command_sort_tie_breaker(
    source_entity: EntityId,
    draw_ordinal: u32,
    mesh_order_signature: u64,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    let stable_instance_key = crate::core::framework::render::render_mesh_stable_instance_key(
        source_entity,
        draw_ordinal,
    );
    stable_instance_key.hash(&mut hasher);
    mesh_order_signature.hash(&mut hasher);
    nonzero_hash(hasher)
}

fn dynamic_command_sort_tie_breaker(source_entity: EntityId, draw_ordinal: u32) -> u64 {
    crate::core::framework::render::render_mesh_stable_instance_key(source_entity, draw_ordinal)
}
