use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::{AssetUri, project::ProjectManager};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::resource::ResourceScheme;

use super::filesystem::{ScenePathGuard, protect_scene_path, reject_linked_components};
use super::{ProjectAuthority, ProjectAuthorityError};

static NEXT_SCENE_PUBLICATION: AtomicU64 = AtomicU64::new(1);

/// A selected project-owned scene. UI pickers turn their result into this request before the
/// project authority resolves or loads any source path.
#[derive(Clone, Debug)]
pub struct SceneOpenRequest {
    scene_uri: AssetUri,
}

impl SceneOpenRequest {
    pub fn new(scene_uri: AssetUri) -> Self {
        Self { scene_uri }
    }

    pub fn scene_uri(&self) -> &AssetUri {
        &self.scene_uri
    }
}

/// A new scene destination selected by the user within the active project.
#[derive(Clone, Debug)]
pub struct SceneCreateRequest {
    scene_uri: AssetUri,
}

impl SceneCreateRequest {
    pub fn new(scene_uri: AssetUri) -> Self {
        Self { scene_uri }
    }

    pub fn scene_uri(&self) -> &AssetUri {
        &self.scene_uri
    }
}

/// The persisted scene identity and world accepted by the project authority.
#[derive(Clone, Debug)]
pub struct ProjectSceneDocument {
    scene_uri: AssetUri,
    source_path: PathBuf,
    world: Scene,
}

impl ProjectSceneDocument {
    pub fn scene_uri(&self) -> &AssetUri {
        &self.scene_uri
    }

    pub(crate) fn source_path(&self) -> &std::path::Path {
        &self.source_path
    }

    pub(crate) fn world(&self) -> &Scene {
        &self.world
    }
}

/// Owns an unpublished scene staging file until the document route either commits or aborts it.
pub(crate) struct PreparedSceneCreation {
    document: ProjectSceneDocument,
    staging_path: PathBuf,
    _path_guard: ScenePathGuard,
    published: bool,
}

impl PreparedSceneCreation {
    pub(crate) fn document(&self) -> &ProjectSceneDocument {
        &self.document
    }

    fn publish(&mut self) -> Result<(), ProjectAuthorityError> {
        let result = fs::hard_link(&self.staging_path, &self.document.source_path);
        match result {
            Ok(()) => {
                self.published = true;
                Ok(())
            }
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(ProjectAuthorityError::SceneAlreadyExists {
                    path: self.document.source_path.clone(),
                })
            }
            Err(source) => Err(ProjectAuthorityError::io(
                "publish new scene asset",
                &self.document.source_path,
                source,
            )),
        }
    }

    /// Makes the target source visible only after its staging sibling is gone, so a catalog scan
    /// cannot register the transient source as a project asset.
    pub(crate) fn publish_and_discard_staging(&mut self) -> Result<(), ProjectAuthorityError> {
        self.publish()?;
        if let Err(cleanup) = self.remove_staging() {
            return match self.rollback() {
                Ok(()) => Err(cleanup),
                Err(rollback) => Err(ProjectAuthorityError::SceneStagingCleanupRollback {
                    cleanup: Box::new(cleanup),
                    rollback: Box::new(rollback),
                }),
            };
        }
        Ok(())
    }

    pub(crate) fn rollback(&mut self) -> Result<(), ProjectAuthorityError> {
        if self.published {
            fs::remove_file(&self.document.source_path).map_err(|source| {
                ProjectAuthorityError::io(
                    "remove rejected scene asset",
                    &self.document.source_path,
                    source,
                )
            })?;
            self.published = false;
        }
        self.remove_staging()
    }

    pub(crate) fn finish(mut self) -> ProjectSceneDocument {
        self.published = false;
        self.document.clone()
    }

    fn remove_staging(&mut self) -> Result<(), ProjectAuthorityError> {
        match fs::remove_file(&self.staging_path) {
            Ok(()) => Ok(()),
            Err(ref error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(ProjectAuthorityError::io(
                "remove scene staging file",
                &self.staging_path,
                source,
            )),
        }
    }
}

impl Drop for PreparedSceneCreation {
    fn drop(&mut self) {
        if self.published {
            let _ = fs::remove_file(&self.document.source_path);
        }
        let _ = fs::remove_file(&self.staging_path);
    }
}

impl ProjectAuthority {
    /// Opens one existing scene from the active project generation without reopening its manifest.
    pub fn open_scene(
        &self,
        project: &ProjectManager,
        request: SceneOpenRequest,
    ) -> Result<ProjectSceneDocument, ProjectAuthorityError> {
        let source_path = existing_scene_source_path(project, request.scene_uri())?;
        let _path_guard = protect_scene_path(project.paths().root(), &source_path, true)?;
        let world = Scene::load_scene_from_uri(project, request.scene_uri())?;
        Ok(ProjectSceneDocument {
            scene_uri: request.scene_uri,
            source_path,
            world,
        })
    }

    /// Creates an empty scene at a project-owned URI. Existing sources are never overwritten.
    pub fn create_scene(
        &self,
        project: &mut ProjectManager,
        request: SceneCreateRequest,
    ) -> Result<ProjectSceneDocument, ProjectAuthorityError> {
        let mut creation = self.prepare_scene_creation(project, request)?;
        creation.publish_and_discard_staging()?;
        if let Err(catalog) = project.scan_and_import() {
            return match creation.rollback() {
                Ok(()) => match project.scan_and_import() {
                    Ok(_) => Err(ProjectAuthorityError::SceneCatalog { source: catalog }),
                    Err(reconcile) => {
                        Err(ProjectAuthorityError::SceneCatalogReconcile { catalog, reconcile })
                    }
                },
                Err(rollback) => Err(ProjectAuthorityError::SceneCatalogRollback {
                    catalog,
                    rollback: Box::new(rollback),
                }),
            };
        }
        let document = creation.finish();
        Ok(document)
    }

    pub(crate) fn prepare_scene_creation(
        &self,
        project: &ProjectManager,
        request: SceneCreateRequest,
    ) -> Result<PreparedSceneCreation, ProjectAuthorityError> {
        validate_scene_uri(request.scene_uri())?;
        let source_path = project.primary_project_source_path_for_uri(request.scene_uri())?;
        let parent = source_path
            .parent()
            .ok_or_else(|| ProjectAuthorityError::SceneTarget {
                uri: request.scene_uri().to_string(),
                reason: "scene asset path has no parent directory",
            })?;
        reject_linked_components(&source_path)?;
        if !parent.is_dir() {
            return Err(ProjectAuthorityError::SceneTarget {
                uri: request.scene_uri().to_string(),
                reason: "scene asset parent directory must already exist within the project",
            });
        }
        let path_guard = protect_scene_path(project.paths().root(), &source_path, false)?;
        let world = Scene::default();
        let staging_uri = scene_staging_uri(request.scene_uri());
        let staging_path = project.primary_project_source_path_for_uri(&staging_uri)?;
        if let Err(error) = world.save_scene_to_project(project, &staging_uri) {
            let _ = fs::remove_file(&staging_path);
            return Err(error.into());
        }
        Ok(PreparedSceneCreation {
            document: ProjectSceneDocument {
                scene_uri: request.scene_uri,
                source_path,
                world,
            },
            staging_path,
            _path_guard: path_guard,
            published: false,
        })
    }
}

fn existing_scene_source_path(
    project: &ProjectManager,
    scene_uri: &AssetUri,
) -> Result<PathBuf, ProjectAuthorityError> {
    validate_scene_uri(scene_uri)?;
    let source_path = project.source_path_for_uri(scene_uri)?;
    reject_linked_components(&source_path)?;
    Ok(source_path)
}

fn validate_scene_uri(scene_uri: &AssetUri) -> Result<(), ProjectAuthorityError> {
    if scene_uri.scheme() != ResourceScheme::Res {
        return Err(ProjectAuthorityError::SceneTarget {
            uri: scene_uri.to_string(),
            reason: "scene assets must use a project-owned res:// URI",
        });
    }
    if !scene_uri.path().ends_with(".scene.toml") {
        return Err(ProjectAuthorityError::SceneTarget {
            uri: scene_uri.to_string(),
            reason: "scene asset URI must end in .scene.toml",
        });
    }
    if scene_uri.label().is_some() {
        return Err(ProjectAuthorityError::SceneTarget {
            uri: scene_uri.to_string(),
            reason: "scene asset URI cannot target a sub-asset label",
        });
    }
    Ok(())
}

fn scene_staging_uri(scene_uri: &AssetUri) -> AssetUri {
    let sequence = NEXT_SCENE_PUBLICATION.fetch_add(1, Ordering::Relaxed);
    let path = scene_uri.path();
    let (directory, file_name) = path.rsplit_once('/').unwrap_or(("", path));
    let staged_name = format!(
        ".{file_name}.zircon-scene-staging-{}-{sequence}.scene.toml",
        std::process::id()
    );
    let staged_path = if directory.is_empty() {
        staged_name
    } else {
        format!("{directory}/{staged_name}")
    };
    AssetUri::parse(&format!("res://{staged_path}"))
        .expect("generated project scene staging URI must be valid")
}
