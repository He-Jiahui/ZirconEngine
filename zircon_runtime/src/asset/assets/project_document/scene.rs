//! 场景项目文档负责作者场景与持久引用的往返；World 保存经 ProjectManager 校验引用身份，加载时把缺失引用转为诊断。

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::PersistedAssetReference;

use crate::asset::{AssetReference, ReferenceResolutionError};

use super::codec::{decode_document, encode_document, ProjectDocumentArtifact};
use crate::asset::assets::{ProjectDocumentError, SceneAsset};

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneAuthoringDocument<R> {
    entities: Vec<SceneEntityDocument<R>>,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneEntityDocument<R> {
    #[serde(default)]
    camera: Option<SceneCameraDocument<R>>,
    #[serde(default)]
    mesh: Option<SceneMeshDocument<R>>,
    #[serde(default)]
    components: Vec<SceneComponentDocument<R>>,
    /// Read-only migration inputs for the pre-registry two-field format.
    #[serde(default, rename = "sprite_2d", skip_serializing_if = "Option::is_none")]
    legacy_sprite_2d: Option<SceneSprite2dDocument<R>>,
    #[serde(default, rename = "mesh_2d", skip_serializing_if = "Option::is_none")]
    legacy_mesh_2d: Option<SceneMesh2dDocument<R>>,
    #[serde(default)]
    collider: Option<SceneColliderDocument<R>>,
    #[serde(default)]
    animation_skeleton: Option<SceneSkeletonDocument<R>>,
    #[serde(default)]
    animation_player: Option<SceneAnimationPlayerDocument<R>>,
    #[serde(default)]
    animation_sequence_player: Option<SceneAnimationSequenceDocument<R>>,
    #[serde(default)]
    animation_graph_player: Option<SceneAnimationGraphDocument<R>>,
    #[serde(default)]
    animation_state_machine_player: Option<SceneAnimationStateMachineDocument<R>>,
    #[serde(default)]
    terrain: Option<SceneTerrainDocument<R>>,
    #[serde(default)]
    tilemap: Option<SceneTilemapDocument<R>>,
    #[serde(default)]
    prefab_instance: Option<ScenePrefabDocument<R>>,
    #[serde(flatten)]
    _rest: toml::Table,
}

/// Registry-owned component row.  Payload stays opaque while the reference table is
/// remapped by the project resolver, so adding a provider does not require editing this DTO.
#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneComponentDocument<R> {
    type_id: String,
    schema_id: String,
    schema_version: u32,
    provider_id: String,
    #[serde(with = "json_payload_as_string")]
    payload: serde_json::Value,
    #[serde(default)]
    references: Vec<R>,
}

mod json_payload_as_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<S>(value: &serde_json::Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&serde_json::to_string(value).map_err(serde::ser::Error::custom)?)
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        serde_json::from_str(&encoded).map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneSprite2dDocument<R> {
    image: R,
    #[serde(default)]
    material: Option<R>,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneMesh2dDocument<R> {
    model: R,
    material: R,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneCameraDocument<R> {
    #[serde(default)]
    target: Option<SceneCameraTargetDocument<R>>,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
enum SceneCameraTargetDocument<R> {
    PrimarySurface,
    Texture { texture: R },
    Headless { size: [u32; 2] },
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneColliderDocument<R> {
    #[serde(default)]
    material: Option<R>,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneSkeletonDocument<R> {
    skeleton: R,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneTerrainDocument<R> {
    terrain: R,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneTilemapDocument<R> {
    tilemap: R,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneAnimationPlayerDocument<R> {
    clip: R,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneAnimationSequenceDocument<R> {
    sequence: R,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneAnimationGraphDocument<R> {
    graph: R,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneAnimationStateMachineDocument<R> {
    state_machine: R,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct ScenePrefabDocument<R> {
    prefab: R,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneMeshDocument<R> {
    model: R,
    #[serde(default)]
    mesh: Option<R>,
    material: R,
    #[serde(default)]
    primitives: Vec<ScenePrimitiveDocument<R>>,
    #[serde(default)]
    lods: Vec<SceneLodDocument<R>>,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct ScenePrimitiveDocument<R> {
    mesh: R,
    material: R,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct SceneLodDocument<R> {
    model: R,
    #[serde(default)]
    mesh: Option<R>,
    material: R,
    #[serde(default)]
    primitives: Vec<ScenePrimitiveDocument<R>>,
    #[serde(flatten)]
    _rest: toml::Table,
}

pub(in crate::asset::assets) fn deserialize_scene(
    document: &str,
    resolver: impl FnMut(&PersistedAssetReference) -> Result<AssetReference, ReferenceResolutionError>,
) -> Result<SceneAsset, ProjectDocumentError> {
    deserialize_scene_artifact(ProjectDocumentArtifact::parse(document)?, resolver)
}

pub(in crate::asset) fn deserialize_scene_artifact(
    document: ProjectDocumentArtifact,
    mut resolver: impl FnMut(
        &PersistedAssetReference,
    ) -> Result<AssetReference, ReferenceResolutionError>,
) -> Result<SceneAsset, ProjectDocumentError> {
    let document = document.into_document::<SceneAuthoringDocument<PersistedAssetReference>>()?;
    let document = map_scene_references(document, |reference| resolver(&reference))?;
    decode_document(document)
}

pub(in crate::asset::assets) fn serialize_scene(
    value: &SceneAsset,
    mut resolver: impl FnMut(
        &AssetReference,
    ) -> Result<PersistedAssetReference, ReferenceResolutionError>,
) -> Result<String, ProjectDocumentError> {
    let document = encode_document::<_, SceneAuthoringDocument<AssetReference>>(value)?;
    let document = map_scene_references(document, |reference| resolver(&reference))?;
    Ok(toml::to_string_pretty(&document)?)
}

fn map_scene_references<A, B>(
    document: SceneAuthoringDocument<A>,
    mut map: impl FnMut(A) -> Result<B, ReferenceResolutionError>,
) -> Result<SceneAuthoringDocument<B>, ReferenceResolutionError>
where
    B: Serialize,
{
    let mut entities = Vec::with_capacity(document.entities.len());
    for entity in document.entities {
        let mesh = match entity.mesh {
            Some(mesh) => Some(map_scene_mesh(mesh, &mut map)?),
            None => None,
        };
        let mut components = entity
            .components
            .into_iter()
            .map(|component| map_scene_component(component, &mut map))
            .collect::<Result<Vec<_>, ReferenceResolutionError>>()?;
        if let Some(sprite) = entity.legacy_sprite_2d {
            components.push(map_legacy_sprite2d(sprite, &mut map)?);
        }
        if let Some(mesh) = entity.legacy_mesh_2d {
            components.push(map_legacy_mesh2d(mesh, &mut map)?);
        }
        entities.push(SceneEntityDocument {
            camera: entity
                .camera
                .map(|camera| map_scene_camera(camera, &mut map))
                .transpose()?,
            mesh,
            components,
            legacy_sprite_2d: None,
            legacy_mesh_2d: None,
            collider: entity
                .collider
                .map(|collider| -> Result<_, ReferenceResolutionError> {
                    Ok(SceneColliderDocument {
                        material: collider.material.map(&mut map).transpose()?,
                        _rest: collider._rest,
                    })
                })
                .transpose()?,
            animation_skeleton: entity
                .animation_skeleton
                .map(|value| -> Result<_, ReferenceResolutionError> {
                    Ok(SceneSkeletonDocument {
                        skeleton: map(value.skeleton)?,
                    })
                })
                .transpose()?,
            animation_player: entity
                .animation_player
                .map(|value| -> Result<_, ReferenceResolutionError> {
                    Ok(SceneAnimationPlayerDocument {
                        clip: map(value.clip)?,
                        _rest: value._rest,
                    })
                })
                .transpose()?,
            animation_sequence_player: entity
                .animation_sequence_player
                .map(|value| -> Result<_, ReferenceResolutionError> {
                    Ok(SceneAnimationSequenceDocument {
                        sequence: map(value.sequence)?,
                        _rest: value._rest,
                    })
                })
                .transpose()?,
            animation_graph_player: entity
                .animation_graph_player
                .map(|value| -> Result<_, ReferenceResolutionError> {
                    Ok(SceneAnimationGraphDocument {
                        graph: map(value.graph)?,
                        _rest: value._rest,
                    })
                })
                .transpose()?,
            animation_state_machine_player: entity
                .animation_state_machine_player
                .map(|value| -> Result<_, ReferenceResolutionError> {
                    Ok(SceneAnimationStateMachineDocument {
                        state_machine: map(value.state_machine)?,
                        _rest: value._rest,
                    })
                })
                .transpose()?,
            terrain: entity
                .terrain
                .map(|value| -> Result<_, ReferenceResolutionError> {
                    Ok(SceneTerrainDocument {
                        terrain: map(value.terrain)?,
                    })
                })
                .transpose()?,
            tilemap: entity
                .tilemap
                .map(|value| -> Result<_, ReferenceResolutionError> {
                    Ok(SceneTilemapDocument {
                        tilemap: map(value.tilemap)?,
                    })
                })
                .transpose()?,
            prefab_instance: entity
                .prefab_instance
                .map(|value| -> Result<_, ReferenceResolutionError> {
                    Ok(ScenePrefabDocument {
                        prefab: map(value.prefab)?,
                        _rest: value._rest,
                    })
                })
                .transpose()?,
            _rest: entity._rest,
        });
    }
    Ok(SceneAuthoringDocument { entities })
}

fn map_scene_camera<A, B>(
    camera: SceneCameraDocument<A>,
    map: &mut impl FnMut(A) -> Result<B, ReferenceResolutionError>,
) -> Result<SceneCameraDocument<B>, ReferenceResolutionError> {
    let target = camera
        .target
        .map(|target| -> Result<_, ReferenceResolutionError> {
            match target {
                SceneCameraTargetDocument::PrimarySurface => {
                    Ok(SceneCameraTargetDocument::PrimarySurface)
                }
                SceneCameraTargetDocument::Texture { texture } => {
                    Ok(SceneCameraTargetDocument::Texture {
                        texture: map(texture)?,
                    })
                }
                SceneCameraTargetDocument::Headless { size } => {
                    Ok(SceneCameraTargetDocument::Headless { size })
                }
            }
        })
        .transpose()?;
    Ok(SceneCameraDocument {
        target,
        _rest: camera._rest,
    })
}

fn map_scene_mesh<A, B>(
    mesh: SceneMeshDocument<A>,
    map: &mut impl FnMut(A) -> Result<B, ReferenceResolutionError>,
) -> Result<SceneMeshDocument<B>, ReferenceResolutionError> {
    Ok(SceneMeshDocument {
        model: map(mesh.model)?,
        mesh: mesh.mesh.map(&mut *map).transpose()?,
        material: map(mesh.material)?,
        primitives: mesh
            .primitives
            .into_iter()
            .map(|primitive| {
                Ok(ScenePrimitiveDocument {
                    mesh: map(primitive.mesh)?,
                    material: map(primitive.material)?,
                })
            })
            .collect::<Result<_, ReferenceResolutionError>>()?,
        lods: mesh
            .lods
            .into_iter()
            .map(|lod| {
                Ok(SceneLodDocument {
                    model: map(lod.model)?,
                    mesh: lod.mesh.map(&mut *map).transpose()?,
                    material: map(lod.material)?,
                    primitives: lod
                        .primitives
                        .into_iter()
                        .map(|primitive| {
                            Ok(ScenePrimitiveDocument {
                                mesh: map(primitive.mesh)?,
                                material: map(primitive.material)?,
                            })
                        })
                        .collect::<Result<_, ReferenceResolutionError>>()?,
                    _rest: lod._rest,
                })
            })
            .collect::<Result<_, ReferenceResolutionError>>()?,
        _rest: mesh._rest,
    })
}

fn map_scene_component<A, B>(
    component: SceneComponentDocument<A>,
    map: &mut impl FnMut(A) -> Result<B, ReferenceResolutionError>,
) -> Result<SceneComponentDocument<B>, ReferenceResolutionError> {
    Ok(SceneComponentDocument {
        type_id: component.type_id,
        schema_id: component.schema_id,
        schema_version: component.schema_version,
        provider_id: component.provider_id,
        payload: component.payload,
        references: component
            .references
            .into_iter()
            .map(&mut *map)
            .collect::<Result<_, ReferenceResolutionError>>()?,
    })
}

fn map_legacy_sprite2d<A, B>(
    sprite: SceneSprite2dDocument<A>,
    map: &mut impl FnMut(A) -> Result<B, ReferenceResolutionError>,
) -> Result<SceneComponentDocument<B>, ReferenceResolutionError>
where
    B: Serialize,
{
    let image = map(sprite.image)?;
    let material = sprite.material.map(&mut *map).transpose()?;
    let mut payload = serde_json::to_value(SceneSprite2dDocument {
        image: &image,
        material: material.as_ref(),
        _rest: sprite._rest,
    })
    .map_err(|error| ReferenceResolutionError::Registry {
        message: format!("legacy sprite payload encoding failed: {error}"),
    })?;
    let object = payload
        .as_object_mut()
        .ok_or_else(|| ReferenceResolutionError::Registry {
            message: "legacy sprite payload is not an object".to_owned(),
        })?;
    let mut references = vec![image];
    object.insert(
        "image".to_owned(),
        serde_json::json!({"$zircon_scene_asset_reference": 0}),
    );
    if let Some(material) = material {
        let index = references.len();
        references.push(material);
        object.insert(
            "material".to_owned(),
            serde_json::json!({"$zircon_scene_asset_reference": index}),
        );
    }
    Ok(SceneComponentDocument {
        type_id: "zircon.render2d.sprite".to_owned(),
        schema_id: "zircon.render2d.sprite.v1".to_owned(),
        schema_version: 1,
        provider_id: "zircon.runtime.render2d".to_owned(),
        payload,
        references,
    })
}

fn map_legacy_mesh2d<A, B>(
    mesh: SceneMesh2dDocument<A>,
    map: &mut impl FnMut(A) -> Result<B, ReferenceResolutionError>,
) -> Result<SceneComponentDocument<B>, ReferenceResolutionError>
where
    B: Serialize,
{
    let model = map(mesh.model)?;
    let material = map(mesh.material)?;
    let mut payload = serde_json::to_value(SceneMesh2dDocument {
        model: &model,
        material: &material,
        _rest: mesh._rest,
    })
    .map_err(|error| ReferenceResolutionError::Registry {
        message: format!("legacy mesh payload encoding failed: {error}"),
    })?;
    let object = payload
        .as_object_mut()
        .ok_or_else(|| ReferenceResolutionError::Registry {
            message: "legacy mesh payload is not an object".to_owned(),
        })?;
    object.insert(
        "model".to_owned(),
        serde_json::json!({"$zircon_scene_asset_reference": 0}),
    );
    object.insert(
        "material".to_owned(),
        serde_json::json!({"$zircon_scene_asset_reference": 1}),
    );
    Ok(SceneComponentDocument {
        type_id: "zircon.render2d.mesh".to_owned(),
        schema_id: "zircon.render2d.mesh.v1".to_owned(),
        schema_version: 1,
        provider_id: "zircon.runtime.render2d".to_owned(),
        payload,
        references: vec![model, material],
    })
}

#[cfg(test)]
#[path = "tests/scene.rs"]
mod tests;
