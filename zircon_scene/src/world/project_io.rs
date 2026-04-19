use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_asset::AssetReference;
use zircon_asset::assets::{
    ImportedAsset, SceneAsset, SceneDirectionalLightAsset, SceneEntityAsset,
    SceneMeshInstanceAsset, SceneMobilityAsset, TransformAsset,
};
use zircon_asset::importer::AssetImportError;
use zircon_asset::project::ProjectManager;
use zircon_resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceLocator, ResourceScheme,
};

use super::World;
use crate::components::{Mobility, NodeKind, Schedule};

const PROJECT_FORMAT_VERSION: u32 = 2;
const BUILTIN_CUBE: &str = "builtin://cube";
const BUILTIN_DEFAULT_MATERIAL: &str = "builtin://material/default";
const BUILTIN_MISSING_MODEL: &str = "builtin://missing-model";
const BUILTIN_MISSING_MATERIAL: &str = "builtin://missing-material";

#[derive(Debug, Error)]
pub enum SceneProjectError {
    #[error("project I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("project parse failed: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("asset import failed: {0}")]
    Asset(#[from] AssetImportError),
    #[error("scene asset error: {0}")]
    SceneAsset(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ProjectDocument {
    format_version: u32,
    world: World,
}

impl World {
    pub fn load_scene_from_uri(
        project: &ProjectManager,
        uri: &ResourceLocator,
    ) -> Result<Self, SceneProjectError> {
        let ImportedAsset::Scene(scene) = project.load_artifact(uri)? else {
            return Err(SceneProjectError::SceneAsset(format!(
                "asset {uri} is not a scene"
            )));
        };
        Self::from_scene_asset(project, &scene)
    }

    pub fn from_scene_asset(
        project: &ProjectManager,
        scene: &SceneAsset,
    ) -> Result<Self, SceneProjectError> {
        let mut world = Self::empty();

        for entity in &scene.entities {
            let kind = if entity.camera.is_some() {
                NodeKind::Camera
            } else if entity.directional_light.is_some() {
                NodeKind::DirectionalLight
            } else if entity
                .mesh
                .as_ref()
                .is_some_and(|mesh| mesh.model.to_string() == BUILTIN_CUBE)
            {
                NodeKind::Cube
            } else if entity.mesh.is_some() {
                NodeKind::Mesh
            } else {
                continue;
            };

            let mesh = entity.mesh.as_ref().map(|mesh| {
                crate::components::MeshRenderer::from_handles(
                    model_handle_for_reference(project, &mesh.model),
                    material_handle_for_reference(project, &mesh.material),
                )
            });

            world
                .insert_node_record(crate::components::NodeRecord {
                    id: entity.entity,
                    name: entity.name.clone(),
                    kind,
                    parent: entity.parent,
                    transform: zircon_math::Transform {
                        translation: zircon_math::Vec3::from_array(entity.transform.translation),
                        rotation: zircon_math::Quat::from_array(entity.transform.rotation),
                        scale: zircon_math::Vec3::from_array(entity.transform.scale),
                    },
                    camera: entity.camera.clone().map(|camera| {
                        crate::components::CameraComponent {
                            fov_y_radians: camera.fov_y_radians,
                            z_near: camera.z_near,
                            z_far: camera.z_far,
                        }
                    }),
                    mesh,
                    directional_light: entity.directional_light.clone().map(|light| {
                        crate::components::DirectionalLight {
                            direction: zircon_math::Vec3::from_array(light.direction),
                            color: zircon_math::Vec3::from_array(light.color),
                            intensity: light.intensity,
                        }
                    }),
                    active: entity.active,
                    render_layer_mask: entity.render_layer_mask,
                    mobility: match entity.mobility {
                        SceneMobilityAsset::Dynamic => Mobility::Dynamic,
                        SceneMobilityAsset::Static => Mobility::Static,
                    },
                })
                .map_err(SceneProjectError::SceneAsset)?;
        }

        world.normalize_after_load();
        Ok(world)
    }

    pub fn to_scene_asset(
        &self,
        project: &ProjectManager,
    ) -> Result<SceneAsset, SceneProjectError> {
        let entities = self
            .entities
            .iter()
            .copied()
            .filter_map(|entity| self.node_record(entity))
            .map(|record| {
                let mesh = record
                    .mesh
                    .map(|mesh| {
                        Ok::<SceneMeshInstanceAsset, SceneProjectError>(SceneMeshInstanceAsset {
                            model: reference_for_model_handle(project, mesh.model)?,
                            material: reference_for_material_handle(project, mesh.material)?,
                        })
                    })
                    .transpose()?;

                Ok(SceneEntityAsset {
                    entity: record.id,
                    name: record.name,
                    parent: record.parent,
                    transform: TransformAsset {
                        translation: record.transform.translation.to_array(),
                        rotation: record.transform.rotation.to_array(),
                        scale: record.transform.scale.to_array(),
                    },
                    active: record.active,
                    render_layer_mask: record.render_layer_mask,
                    mobility: match record.mobility {
                        Mobility::Dynamic => SceneMobilityAsset::Dynamic,
                        Mobility::Static => SceneMobilityAsset::Static,
                    },
                    camera: record
                        .camera
                        .map(|camera| zircon_asset::assets::SceneCameraAsset {
                            fov_y_radians: camera.fov_y_radians,
                            z_near: camera.z_near,
                            z_far: camera.z_far,
                        }),
                    mesh,
                    directional_light: record.directional_light.map(|light| {
                        SceneDirectionalLightAsset {
                            direction: light.direction.to_array(),
                            color: light.color.to_array(),
                            intensity: light.intensity,
                        }
                    }),
                })
            })
            .collect::<Result<Vec<_>, SceneProjectError>>()?;

        Ok(SceneAsset { entities })
    }

    pub fn save_project_to_path(&self, path: impl AsRef<Path>) -> Result<(), SceneProjectError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let document = ProjectDocument {
            format_version: PROJECT_FORMAT_VERSION,
            world: self.clone(),
        };
        fs::write(path, serde_json::to_string_pretty(&document)?)?;
        Ok(())
    }

    pub fn load_project_from_path(path: impl AsRef<Path>) -> Result<Self, SceneProjectError> {
        let json = fs::read_to_string(path)?;
        let mut document: ProjectDocument = serde_json::from_str(&json)?;
        document.world.normalize_after_load();
        Ok(document.world)
    }

    fn normalize_after_load(&mut self) {
        self.schedule = Schedule::default();
        if self.kinds.len() != self.entities.len() {
            self.kinds.clear();
            for entity in &self.entities {
                let kind = if self.cameras.contains_key(entity) {
                    NodeKind::Camera
                } else if self.directional_lights.contains_key(entity) {
                    NodeKind::DirectionalLight
                } else if self.mesh_renderers.contains_key(entity) {
                    let is_cube = self.mesh_renderers.get(entity).is_some_and(|mesh| {
                        mesh.model.id() == ResourceId::from_stable_label(BUILTIN_CUBE)
                    });
                    if is_cube {
                        NodeKind::Cube
                    } else {
                        NodeKind::Mesh
                    }
                } else {
                    continue;
                };
                self.kinds.insert(*entity, kind);
            }
        }
        self.next_id = self.entities.iter().copied().max().unwrap_or(0) + 1;
        if self.cameras.is_empty() {
            self.spawn_node(NodeKind::Camera);
        }
        if !self.cameras.contains_key(&self.active_camera) {
            self.active_camera = *self.cameras.keys().next().expect("camera exists");
        }
        if self.directional_lights.is_empty() {
            self.spawn_node(NodeKind::DirectionalLight);
        }
        for entity in self.entities.iter().copied().collect::<Vec<_>>() {
            self.active_self.entry(entity).or_default();
            self.render_layer_masks.entry(entity).or_default();
            self.mobility.entry(entity).or_default();
        }
        self.rebuild_derived_state();
    }
}

fn model_handle_for_reference(
    project: &ProjectManager,
    reference: &AssetReference,
) -> ResourceHandle<ModelMarker> {
    let locator = &reference.locator;
    if locator.scheme() == ResourceScheme::Builtin {
        return ResourceHandle::new(ResourceId::from_locator(locator));
    }

    project
        .asset_id_for_uuid(reference.uuid)
        .or_else(|| project.asset_id_for_uri(locator))
        .map(ResourceHandle::new)
        .unwrap_or_else(|| {
            ResourceHandle::new(ResourceId::from_stable_label(BUILTIN_MISSING_MODEL))
        })
}

fn material_handle_for_reference(
    project: &ProjectManager,
    reference: &AssetReference,
) -> ResourceHandle<MaterialMarker> {
    let locator = &reference.locator;
    if locator.scheme() == ResourceScheme::Builtin {
        return ResourceHandle::new(ResourceId::from_locator(locator));
    }

    project
        .asset_id_for_uuid(reference.uuid)
        .or_else(|| project.asset_id_for_uri(locator))
        .map(ResourceHandle::new)
        .unwrap_or_else(|| {
            ResourceHandle::new(ResourceId::from_stable_label(BUILTIN_MISSING_MATERIAL))
        })
}

fn reference_for_model_handle(
    project: &ProjectManager,
    handle: ResourceHandle<ModelMarker>,
) -> Result<AssetReference, SceneProjectError> {
    reference_for_handle(project, handle.id(), "model")
}

fn reference_for_material_handle(
    project: &ProjectManager,
    handle: ResourceHandle<MaterialMarker>,
) -> Result<AssetReference, SceneProjectError> {
    reference_for_handle(project, handle.id(), "material")
}

fn reference_for_handle(
    project: &ProjectManager,
    id: ResourceId,
    label: &str,
) -> Result<AssetReference, SceneProjectError> {
    if let Some(reference) = project.asset_reference_for_id(id) {
        return Ok(reference);
    }
    if let Some(locator) = builtin_locator_for_id(id) {
        return Ok(AssetReference::from_locator(locator));
    }
    Err(SceneProjectError::SceneAsset(format!(
        "missing persistent locator for {label} resource {id}"
    )))
}

fn builtin_locator_for_id(id: ResourceId) -> Option<ResourceLocator> {
    for locator_text in [
        BUILTIN_CUBE,
        BUILTIN_DEFAULT_MATERIAL,
        BUILTIN_MISSING_MODEL,
        BUILTIN_MISSING_MATERIAL,
    ] {
        let locator = ResourceLocator::parse(locator_text).expect("builtin locator");
        if ResourceId::from_locator(&locator) == id {
            return Some(locator);
        }
    }
    None
}
