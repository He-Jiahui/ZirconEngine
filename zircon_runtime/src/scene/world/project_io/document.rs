use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::asset::assets::ProjectDocumentError;
use crate::asset::importer::AssetImportError;
use crate::asset::AssetUuid;
use crate::core::resource::io::atomic_write;
use crate::core::resource::{ResourceId, ResourceLocator};
use crate::scene::components::{
    AmbientLight, CameraComponent, DirectionalLight, MeshRenderer, Mobility, NodeKind, PointLight,
    RectLight, RenderLayerMask, SpotLight,
};
use crate::scene::ecs::Schedule;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use thiserror::Error;

use super::super::transform_validation::validate_persisted_transforms;
use super::super::World;
use super::BUILTIN_CUBE;

const PROJECT_FORMAT_VERSION: u32 = 2;

#[derive(Debug, Error)]
pub enum SceneProjectError {
    #[error("project I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("project parse failed: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("asset import failed: {0}")]
    Asset(#[from] AssetImportError),
    #[error(transparent)]
    ProjectDocument(#[from] ProjectDocumentError),
    #[error(transparent)]
    Scene(#[from] crate::scene::SceneError),
    #[error("scene asset error: {0}")]
    SceneAsset(String),
    #[error("unsupported project format version {actual}; expected {expected}")]
    UnsupportedProjectFormatVersion { expected: u32, actual: u32 },
    #[error("dangling asset reference {uuid} at {locator}")]
    DanglingAssetReference {
        uuid: AssetUuid,
        locator: ResourceLocator,
    },
}

#[derive(Deserialize)]
struct BorrowedProjectDocument<'document> {
    format_version: u32,
    #[serde(borrow)]
    world: &'document RawValue,
}

#[derive(Serialize)]
struct ProjectDocumentRef<'world> {
    format_version: u32,
    world: &'world World,
}

impl World {
    pub fn save_project_to_path(&self, path: impl AsRef<Path>) -> Result<(), SceneProjectError> {
        let path = path.as_ref();
        let bytes = self.project_document_bytes(usize::MAX)?;
        atomic_write(path, &bytes)?;
        Ok(())
    }

    pub(crate) fn project_document_bytes(
        &self,
        max_bytes: usize,
    ) -> Result<Vec<u8>, SceneProjectError> {
        validate_persisted_transforms(self)?;
        let document = ProjectDocumentRef {
            format_version: PROJECT_FORMAT_VERSION,
            world: self,
        };
        let mut writer = BoundedDocumentWriter::new(max_bytes);
        serde_json::to_writer_pretty(&mut writer, &document)?;
        Ok(writer.finish())
    }

    pub fn load_project_from_path(path: impl AsRef<Path>) -> Result<Self, SceneProjectError> {
        let json = fs::read_to_string(path)?;
        let document: BorrowedProjectDocument<'_> = serde_json::from_str(&json)?;
        if document.format_version != PROJECT_FORMAT_VERSION {
            return Err(SceneProjectError::UnsupportedProjectFormatVersion {
                expected: PROJECT_FORMAT_VERSION,
                actual: document.format_version,
            });
        }
        let mut world: World = serde_json::from_str(document.world.get())?;
        validate_persisted_transforms(&world)?;
        world.normalize_after_load();
        Ok(world)
    }

    pub(super) fn normalize_scene_asset_after_load(&mut self) {
        self.normalize_loaded_state(false);
    }

    fn normalize_after_load(&mut self) {
        self.normalize_loaded_state(true);
    }

    fn normalize_loaded_state(&mut self, ensure_default_nodes: bool) {
        self.schedule = Schedule::default();
        if self.kinds.len() != self.entities.len() {
            self.kinds.clear();
            for entity in &self.entities {
                let kind = if self.contains_component::<CameraComponent>(*entity) {
                    NodeKind::Camera
                } else if self.contains_component::<AmbientLight>(*entity) {
                    NodeKind::AmbientLight
                } else if self.contains_component::<DirectionalLight>(*entity) {
                    NodeKind::DirectionalLight
                } else if self.contains_component::<PointLight>(*entity) {
                    NodeKind::PointLight
                } else if self.contains_component::<RectLight>(*entity) {
                    NodeKind::RectLight
                } else if self.contains_component::<SpotLight>(*entity) {
                    NodeKind::SpotLight
                } else if self.contains_component::<MeshRenderer>(*entity) {
                    let is_cube = self.get::<MeshRenderer>(*entity).is_some_and(|mesh| {
                        mesh.model.id() == ResourceId::from_stable_label(BUILTIN_CUBE)
                    });
                    if is_cube {
                        NodeKind::Cube
                    } else {
                        NodeKind::Mesh
                    }
                } else {
                    NodeKind::Empty
                };
                self.kinds.insert(*entity, kind);
            }
        }
        self.rebuild_node_kind_ordinals();
        self.next_id = self.entities.iter().copied().max().unwrap_or(0) + 1;
        if ensure_default_nodes && self.camera_count() == 0 {
            self.spawn_node(NodeKind::Camera);
        }
        if !self.contains_component::<CameraComponent>(self.active_camera) {
            self.active_camera = self
                .entities
                .iter()
                .copied()
                .find(|entity| self.contains_component::<CameraComponent>(*entity))
                .unwrap_or(0);
        }
        if ensure_default_nodes
            && self
                .registered_component_id::<DirectionalLight>()
                .map_or(true, |component_id| {
                    self.component_count_for_id(component_id) == 0
                })
        {
            self.spawn_node(NodeKind::DirectionalLight);
        }
        for entity_index in 0..self.entities.len() {
            let entity = self.entities[entity_index];
            let mut row = self.begin_component_row(entity);
            let mut changed = false;
            if !self.contains_component::<crate::scene::components::ActiveSelf>(entity) {
                self.stage_component_row_value(
                    &mut row,
                    crate::scene::components::ActiveSelf::default(),
                );
                changed = true;
            }
            if !self.contains_component::<RenderLayerMask>(entity) {
                self.stage_component_row_value(&mut row, RenderLayerMask::default());
                changed = true;
            }
            if !self.contains_component::<Mobility>(entity) {
                self.stage_component_row_value(&mut row, Mobility::default());
                changed = true;
            }
            if changed {
                self.commit_component_row(entity, row, true);
            }
        }
        self.rebuild_typed_component_presence();
        self.mark_derived_state_dirty();
        self.flush_scene_systems_now();
    }
}

struct BoundedDocumentWriter {
    bytes: Vec<u8>,
    max_bytes: usize,
}

impl BoundedDocumentWriter {
    fn new(max_bytes: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(max_bytes.min(64 * 1024)),
            max_bytes,
        }
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

impl Write for BoundedDocumentWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.len() > self.max_bytes.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other(format!(
                "scene artifact exceeds {} byte limit",
                self.max_bytes
            )));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
