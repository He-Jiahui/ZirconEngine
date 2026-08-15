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
) -> Result<SceneAuthoringDocument<B>, ReferenceResolutionError> {
    let mut entities = Vec::with_capacity(document.entities.len());
    for entity in document.entities {
        let mesh = match entity.mesh {
            Some(mesh) => Some(map_scene_mesh(mesh, &mut map)?),
            None => None,
        };
        entities.push(SceneEntityDocument {
            camera: entity
                .camera
                .map(|camera| map_scene_camera(camera, &mut map))
                .transpose()?,
            mesh,
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

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
    use zircon_runtime_interface::resource::ResourceScheme;

    use super::*;
    use crate::asset::assets::{
        SceneEntityAsset, SceneMeshInstanceAsset, SceneMobilityAsset, TransformAsset,
    };
    use crate::asset::{AssetUri, AssetUuid};

    #[test]
    fn formal_scene_writer_reader_round_trips_project_builtin_and_subasset_references() {
        let model_guid: AssetUuid = "fa111111-2222-4333-8444-555555555555".parse().unwrap();
        let mesh_guid: AssetUuid = "fb111111-2222-4333-8444-555555555555".parse().unwrap();
        let model = AssetReference::new(
            model_guid,
            AssetUri::parse("res://models/hero.glb").unwrap(),
        );
        let mesh = AssetReference::new(
            mesh_guid,
            AssetUri::parse("res://models/hero.glb#Mesh0").unwrap(),
        );
        let material =
            AssetReference::from_locator(AssetUri::parse("builtin://material/default").unwrap());
        let scene = SceneAsset {
            entities: vec![SceneEntityAsset {
                entity: 1,
                name: "Roundtrip".to_owned(),
                parent: None,
                transform: TransformAsset::default(),
                active: true,
                render_layer_mask: 1,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: model.clone(),
                    mesh: Some(mesh.clone()),
                    material: material.clone(),
                    render_queue: 0,
                    material_queue: 0,
                    order_in_layer: 0,
                    depth_bias: 0.0,
                    morph_weights: Vec::new(),
                    primitives: Vec::new(),
                    lods: Vec::new(),
                }),
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
                script_bindings: Vec::new(),
            }],
        };

        let document = serialize_scene(&scene, |reference| {
            if reference.locator.scheme() == ResourceScheme::Builtin {
                return Ok(PersistedAssetReference::builtin(reference.locator.clone()));
            }
            Ok(PersistedAssetReference::project(
                AssetRef::try_new(
                    reference.uuid,
                    RelPath::parse("models/hero.glb").unwrap(),
                    reference.locator.label().map(str::to_owned),
                )
                .unwrap(),
            ))
        })
        .unwrap();
        let reloaded = deserialize_scene(&document, |reference| {
            if let Some(locator) = reference.builtin_locator() {
                return Ok(AssetReference::from_locator(locator.clone()));
            }
            let reference = reference.project_ref().expect("project reference");
            let mut locator = format!("res://{}", reference.path_hint());
            if let Some(sub) = reference.sub() {
                locator.push('#');
                locator.push_str(sub);
            }
            Ok(AssetReference::new(
                reference.guid(),
                AssetUri::parse(&locator).unwrap(),
            ))
        })
        .unwrap();

        assert_eq!(reloaded, scene);
    }
}
