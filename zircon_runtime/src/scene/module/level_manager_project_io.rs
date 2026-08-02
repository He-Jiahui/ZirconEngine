use std::path::Path;
use std::sync::Arc;

use crate::asset::project::ProjectManager;
use crate::core::framework::scene::{SceneArtifactTicket, WorldHandle};
use crate::core::resource::io::atomic_file::atomic_write;
use crate::core::resource::ResourceLocator;
use crate::core::runtime::BoundedKeyedIoFailure;

use super::level_display_name::display_name_for_level;
use super::scene_artifact_io::MAX_SCENE_ARTIFACT_BYTES;
use super::DefaultLevelManager;
use crate::scene::{
    serializer::SceneAssetSerializer,
    world::{SceneProjectError, World},
    LevelMetadata, LevelSystem,
};

impl DefaultLevelManager {
    pub fn save_world(
        &self,
        handle: WorldHandle,
        path: impl AsRef<Path>,
    ) -> Result<Arc<dyn SceneArtifactTicket>, SceneProjectError> {
        let level = self.level(handle).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "world handle not found")
        })?;
        let world = level.snapshot();
        let path = path.as_ref().to_path_buf();
        let key = format!("world://{}", path.to_string_lossy());
        self.scene_artifact_io().submit(
            key,
            Box::new(move || {
                let document = world
                    .project_document_bytes(MAX_SCENE_ARTIFACT_BYTES)
                    .map_err(|error| scene_artifact_failure("world", error))?;
                atomic_write(&path, &document)
                    .map_err(SceneProjectError::from)
                    .map_err(|error| scene_artifact_failure("world", error))
            }),
        )
    }

    pub fn load_world(&self, path: impl AsRef<Path>) -> Result<LevelSystem, SceneProjectError> {
        let world = World::load_project_from_path(path)?;
        self.try_create_level(world, LevelMetadata::default())
            .map_err(|error| SceneProjectError::SceneAsset(error.to_string()))
    }

    pub fn load_level(
        &self,
        project: &ProjectManager,
        uri: &ResourceLocator,
    ) -> Result<LevelSystem, SceneProjectError> {
        let world = SceneAssetSerializer::load_world(project, uri)?;
        self.try_create_level(
            world,
            LevelMetadata {
                project_root: Some(project.paths().root().to_string_lossy().into_owned()),
                asset_uri: Some(uri.to_string()),
                display_name: display_name_for_level(uri),
            },
        )
        .map_err(|error| SceneProjectError::SceneAsset(error.to_string()))
    }

    pub fn save_level(
        &self,
        handle: WorldHandle,
        project: &ProjectManager,
        uri: &ResourceLocator,
    ) -> Result<Arc<dyn SceneArtifactTicket>, SceneProjectError> {
        let level = self.level(handle).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "world handle not found")
        })?;
        let world = level.snapshot();
        let project = project.clone();
        let uri = uri.clone();
        let key = format!(
            "project://{}/{}",
            project.paths().root().to_string_lossy(),
            uri
        );
        self.scene_artifact_io().submit(
            key,
            Box::new(move || persist_scene_artifact(&world, &project, &uri)),
        )
    }
}

fn persist_scene_artifact(
    world: &World,
    project: &ProjectManager,
    uri: &ResourceLocator,
) -> Result<(), BoundedKeyedIoFailure> {
    let result = (|| {
        let scene = SceneAssetSerializer::serialize_world(project, world)?;
        let document = scene
            .to_project_toml_string(|reference| project.persist_runtime_reference(reference))?;
        if document.len() > MAX_SCENE_ARTIFACT_BYTES {
            return Err(SceneProjectError::SceneAsset(format!(
                "scene artifact contains {} bytes, exceeding the {} byte limit",
                document.len(),
                MAX_SCENE_ARTIFACT_BYTES
            )));
        }
        let path = project.existing_or_primary_project_source_path_for_uri(uri)?;
        atomic_write(&path, document.as_bytes())?;
        Ok::<(), SceneProjectError>(())
    })();
    result.map_err(|error| scene_artifact_failure("scene", error))
}

fn scene_artifact_failure(_kind: &'static str, _error: SceneProjectError) -> BoundedKeyedIoFailure {
    #[cfg(feature = "diagnostic-log")]
    crate::diagnostic_log::write_log(
        "scene_artifact_io",
        format!("{_kind} artifact persistence failed: {_error}"),
    );
    BoundedKeyedIoFailure::new("scene_artifact_persistence_failed")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use crate::asset::project::{ProjectManager, ProjectManifest};
    use crate::asset::AssetUri;
    use crate::core::framework::scene::{SceneArtifactTerminal, SceneArtifactWaitResult};
    use crate::core::resource::ResourceLocator;
    use crate::scene::{DefaultLevelManager, LevelMetadata, World};

    #[test]
    fn scene_save_returns_a_ticket_and_persists_on_the_bounded_io_lane() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon_scene_artifact_ticket_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(root.join("assets/scenes")).unwrap();
        ProjectManifest::new(
            "Scene Artifact Ticket Fixture",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(root.join("zircon-project.toml"))
        .unwrap();
        let project = ProjectManager::open(&root).unwrap();
        let manager = DefaultLevelManager::default();
        let level = manager.create_level(World::empty(), LevelMetadata::default());
        let uri = ResourceLocator::parse("res://scenes/main.scene.toml").unwrap();

        let ticket = manager.save_level(level.handle(), &project, &uri).unwrap();

        assert_eq!(
            ticket.wait_until(Instant::now() + Duration::from_secs(10)),
            SceneArtifactWaitResult::Terminal(SceneArtifactTerminal::Succeeded)
        );
        assert!(root.join("assets/scenes/main.scene.toml").is_file());

        drop(manager);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn world_project_document_rejects_bytes_beyond_the_lane_quote() {
        let error = World::empty().project_document_bytes(1).unwrap_err();

        assert!(error
            .to_string()
            .contains("scene artifact exceeds 1 byte limit"));
    }
}
