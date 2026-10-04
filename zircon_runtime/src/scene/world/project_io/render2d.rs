use crate::asset::assets::{
    SceneComponentAssetRecord, SceneEntityAsset, SceneMesh2dAsset, SceneSprite2dAsset,
};
use crate::asset::project::ProjectManager;
use crate::core::math::{Vec2, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, TextureMarker};
use crate::scene::components::{Mesh2dComponent, NodeRecord, Sprite2dComponent};
use crate::scene::world::World;
use serde_json::Value;

use super::components::{
    SceneComponentSerializer, SceneComponentSerializerRegistry, BUILTIN_PROVIDER_ID,
    MESH_SCHEMA_ID, MESH_TYPE_ID, SCHEMA_VERSION, SPRITE_SCHEMA_ID, SPRITE_TYPE_ID,
};
use super::references::{handle_for_reference, reference_for_handle};
use super::SceneProjectError;

pub(super) fn capture_sprite2d(
    project: &ProjectManager,
    _world: &World,
    record: &NodeRecord,
) -> Result<Option<SceneComponentAssetRecord>, SceneProjectError> {
    let Some(asset) = sprite2d_to_asset(project, record.sprite_2d.clone())? else {
        return Ok(None);
    };
    SceneComponentAssetRecord::from_typed(
        SPRITE_TYPE_ID,
        SPRITE_SCHEMA_ID,
        SCHEMA_VERSION,
        BUILTIN_PROVIDER_ID,
        &asset,
    )
    .map(Some)
    .map_err(SceneProjectError::SceneAsset)
}

pub(super) fn capture_mesh2d(
    project: &ProjectManager,
    _world: &World,
    record: &NodeRecord,
) -> Result<Option<SceneComponentAssetRecord>, SceneProjectError> {
    let Some(asset) = mesh2d_to_asset(project, record.mesh_2d.clone())? else {
        return Ok(None);
    };
    SceneComponentAssetRecord::from_typed(
        MESH_TYPE_ID,
        MESH_SCHEMA_ID,
        SCHEMA_VERSION,
        BUILTIN_PROVIDER_ID,
        &asset,
    )
    .map(Some)
    .map_err(SceneProjectError::SceneAsset)
}

pub(super) fn instantiate_sprite2d(
    project: &ProjectManager,
    row: &SceneComponentAssetRecord,
    record: &mut NodeRecord,
) -> Result<Option<(String, Value)>, SceneProjectError> {
    let asset: SceneSprite2dAsset = row.decode_typed().map_err(SceneProjectError::SceneAsset)?;
    record.sprite_2d = Some(sprite2d_from_asset(project, Some(&asset))?.ok_or_else(|| {
        SceneProjectError::SceneAsset("sprite2d row decoded without a value".into())
    })?);
    Ok(None)
}

pub(super) fn instantiate_mesh2d(
    project: &ProjectManager,
    row: &SceneComponentAssetRecord,
    record: &mut NodeRecord,
) -> Result<Option<(String, Value)>, SceneProjectError> {
    let asset: SceneMesh2dAsset = row.decode_typed().map_err(SceneProjectError::SceneAsset)?;
    record.mesh_2d = Some(mesh2d_from_asset(project, Some(&asset))?.ok_or_else(|| {
        SceneProjectError::SceneAsset("mesh2d row decoded without a value".into())
    })?);
    Ok(None)
}

fn sprite2d_from_asset(
    project: &ProjectManager,
    asset: Option<&SceneSprite2dAsset>,
) -> Result<Option<Sprite2dComponent>, SceneProjectError> {
    let Some(asset) = asset else {
        return Ok(None);
    };
    Ok(Some(Sprite2dComponent {
        image: handle_for_reference::<TextureMarker>(project, &asset.image)?,
        material: asset
            .material
            .as_ref()
            .map(|reference| handle_for_reference::<MaterialMarker>(project, reference))
            .transpose()?,
        atlas_region: asset.atlas_region,
        rect: asset.rect,
        flip_x: asset.flip_x,
        flip_y: asset.flip_y,
        anchor: asset.anchor,
        custom_size: asset.custom_size.map(Vec2::from_array),
        image_mode: asset.image_mode,
        color: Vec4::from_array(asset.color),
        z_order: asset.z_order,
        material_alpha_mode: asset.material_alpha_mode,
    }))
}

fn sprite2d_to_asset(
    project: &ProjectManager,
    sprite: Option<Sprite2dComponent>,
) -> Result<Option<SceneSprite2dAsset>, SceneProjectError> {
    sprite
        .map(|sprite| {
            Ok(SceneSprite2dAsset {
                image: reference_for_handle(project, sprite.image.id(), "texture")?,
                material: sprite
                    .material
                    .map(|material| reference_for_handle(project, material.id(), "material"))
                    .transpose()?,
                atlas_region: sprite.atlas_region,
                rect: sprite.rect,
                flip_x: sprite.flip_x,
                flip_y: sprite.flip_y,
                anchor: sprite.anchor,
                custom_size: sprite.custom_size.map(|size| size.to_array()),
                image_mode: sprite.image_mode,
                color: sprite.color.to_array(),
                z_order: sprite.z_order,
                material_alpha_mode: sprite.material_alpha_mode,
            })
        })
        .transpose()
}

fn mesh2d_from_asset(
    project: &ProjectManager,
    asset: Option<&SceneMesh2dAsset>,
) -> Result<Option<Mesh2dComponent>, SceneProjectError> {
    let Some(asset) = asset else {
        return Ok(None);
    };
    Ok(Some(Mesh2dComponent {
        mesh: handle_for_reference::<ModelMarker>(project, &asset.model)?,
        material: handle_for_reference::<MaterialMarker>(project, &asset.material)?,
        color: Vec4::from_array(asset.color),
        z_order: asset.z_order,
        material_alpha_mode: asset.material_alpha_mode,
    }))
}

fn mesh2d_to_asset(
    project: &ProjectManager,
    mesh: Option<Mesh2dComponent>,
) -> Result<Option<SceneMesh2dAsset>, SceneProjectError> {
    mesh.map(|mesh| {
        Ok(SceneMesh2dAsset {
            model: reference_for_handle(project, mesh.mesh.id(), "model")?,
            material: reference_for_handle(project, mesh.material.id(), "material")?,
            color: mesh.color.to_array(),
            z_order: mesh.z_order,
            material_alpha_mode: mesh.material_alpha_mode,
        })
    })
    .transpose()
}

#[cfg(test)]
#[path = "tests/render2d.rs"]
mod tests;
