use crate::asset::assets::{
    SceneMeshInstanceAsset, SceneMeshLodLevelAsset, SceneMeshPrimitiveBindingAsset,
};
use crate::asset::project::ProjectManager;
use crate::core::resource::{MaterialMarker, MeshMarker};
use crate::scene::components::{MeshRenderer, MeshRendererLodLevel, MeshRendererPrimitiveBinding};

use super::SceneProjectError;
use super::references::{
    handle_for_reference, material_handle_for_reference, model_handle_for_reference,
    reference_for_material_handle, reference_for_mesh_handle, reference_for_model_handle,
};

pub(super) fn mesh_from_asset(
    project: &ProjectManager,
    mesh: Option<&SceneMeshInstanceAsset>,
) -> Result<Option<MeshRenderer>, SceneProjectError> {
    let Some(mesh) = mesh else {
        return Ok(None);
    };
    let mut renderer = MeshRenderer::from_handles(
        model_handle_for_reference(project, &mesh.model)?,
        material_handle_for_reference(project, &mesh.material)?,
    );
    renderer.mesh = mesh
        .mesh
        .as_ref()
        .map(|reference| handle_for_reference::<MeshMarker>(project, reference))
        .transpose()?;
    renderer.render_queue = mesh.render_queue;
    renderer.material_queue = mesh.material_queue;
    renderer.order_in_layer = mesh.order_in_layer;
    renderer.depth_bias = mesh.depth_bias;
    renderer.morph_weights = mesh.morph_weights.clone();
    renderer.primitives = mesh
        .primitives
        .iter()
        .map(|primitive| {
            Ok(MeshRendererPrimitiveBinding {
                mesh: handle_for_reference::<MeshMarker>(project, &primitive.mesh)?,
                material: handle_for_reference::<MaterialMarker>(project, &primitive.material)?,
            })
        })
        .collect::<Result<Vec<_>, SceneProjectError>>()?;
    renderer.lods = mesh
        .lods
        .iter()
        .map(|lod| {
            Ok(MeshRendererLodLevel {
                min_distance: lod.min_distance,
                model: model_handle_for_reference(project, &lod.model)?,
                mesh: lod
                    .mesh
                    .as_ref()
                    .map(|reference| handle_for_reference::<MeshMarker>(project, reference))
                    .transpose()?,
                material: material_handle_for_reference(project, &lod.material)?,
                primitives: lod
                    .primitives
                    .iter()
                    .map(|primitive| {
                        Ok(MeshRendererPrimitiveBinding {
                            mesh: handle_for_reference::<MeshMarker>(project, &primitive.mesh)?,
                            material: handle_for_reference::<MaterialMarker>(
                                project,
                                &primitive.material,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, SceneProjectError>>()?,
            })
        })
        .collect::<Result<Vec<_>, SceneProjectError>>()?;
    Ok(Some(renderer))
}

pub(super) fn mesh_to_asset(
    project: &ProjectManager,
    mesh: Option<MeshRenderer>,
) -> Result<Option<SceneMeshInstanceAsset>, SceneProjectError> {
    mesh.map(|mesh| {
        Ok::<SceneMeshInstanceAsset, SceneProjectError>(SceneMeshInstanceAsset {
            model: reference_for_model_handle(project, mesh.model)?,
            mesh: mesh
                .mesh
                .map(|mesh| reference_for_mesh_handle(project, mesh))
                .transpose()?,
            material: reference_for_material_handle(project, mesh.material)?,
            render_queue: mesh.render_queue,
            material_queue: mesh.material_queue,
            order_in_layer: mesh.order_in_layer,
            depth_bias: mesh.depth_bias,
            morph_weights: mesh.morph_weights,
            primitives: mesh
                .primitives
                .into_iter()
                .map(|primitive| {
                    Ok::<SceneMeshPrimitiveBindingAsset, SceneProjectError>(
                        SceneMeshPrimitiveBindingAsset {
                            mesh: reference_for_mesh_handle(project, primitive.mesh)?,
                            material: reference_for_material_handle(project, primitive.material)?,
                        },
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
            lods: mesh
                .lods
                .into_iter()
                .map(|lod| {
                    Ok::<SceneMeshLodLevelAsset, SceneProjectError>(SceneMeshLodLevelAsset {
                        min_distance: lod.min_distance,
                        model: reference_for_model_handle(project, lod.model)?,
                        mesh: lod
                            .mesh
                            .map(|mesh| reference_for_mesh_handle(project, mesh))
                            .transpose()?,
                        material: reference_for_material_handle(project, lod.material)?,
                        primitives: lod
                            .primitives
                            .into_iter()
                            .map(|primitive| {
                                Ok::<SceneMeshPrimitiveBindingAsset, SceneProjectError>(
                                    SceneMeshPrimitiveBindingAsset {
                                        mesh: reference_for_mesh_handle(project, primitive.mesh)?,
                                        material: reference_for_material_handle(
                                            project,
                                            primitive.material,
                                        )?,
                                    },
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    })
    .transpose()
}
