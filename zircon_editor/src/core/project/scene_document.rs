//! 项目场景读写只允许项目拥有的完整场景资源；创建先准备暂存文档，再由上游事务发布或回滚，目录链租约必须覆盖该交接期间。
#[cfg(not(target_os = "linux"))]
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::{project::ProjectManager, AssetUri};
use zircon_runtime::scene::world::SceneSourceRevision;
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::resource::ResourceScheme;

use super::filesystem::{protect_scene_path, reject_linked_components, ScenePathGuard};
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
    source_revision: SceneSourceRevision,
}

impl ProjectSceneDocument {
    pub fn source_revision(&self) -> &SceneSourceRevision {
        &self.source_revision
    }

    pub fn scene_uri(&self) -> &AssetUri {
        &self.scene_uri
    }

    pub(crate) fn source_path(&self) -> &std::path::Path {
        &self.source_path
    }

    pub(crate) fn world(&self) -> &Scene {
        &self.world
    }

    pub(crate) fn take_world(&mut self) -> Scene {
        std::mem::replace(&mut self.world, Scene::empty())
    }

    pub(crate) fn into_world(self) -> Scene {
        self.world
    }
}

// 暂存创建的所有者必须持有路径租约直到发布或回滚结束；丢弃会清理尚未交接的文件。
/// Owns an unpublished scene staging file until the document route either commits or aborts it.
pub(crate) struct PreparedSceneCreation {
    document: Option<ProjectSceneDocument>,
    staging_path: PathBuf,
    _path_guard: ScenePathGuard,
    published: bool,
}

impl PreparedSceneCreation {
    pub(crate) fn document(&self) -> &ProjectSceneDocument {
        self.document
            .as_ref()
            .expect("prepared scene document must remain available until finish")
    }

    pub(crate) fn document_mut(&mut self) -> &mut ProjectSceneDocument {
        self.document
            .as_mut()
            .expect("prepared document remains available until finish")
    }

    fn publish(&mut self) -> Result<(), ProjectAuthorityError> {
        #[cfg(target_os = "linux")]
        self._path_guard.publish_scene(&self.staging_path)?;
        #[cfg(not(target_os = "linux"))]
        match fs::hard_link(&self.staging_path, self.document().source_path()) {
            Ok(()) => {}
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(ProjectAuthorityError::SceneAlreadyExists {
                    path: self.document().source_path.clone(),
                });
            }
            Err(source) => {
                return Err(ProjectAuthorityError::io(
                    "publish new scene asset",
                    self.document().source_path(),
                    source,
                ));
            }
        }
        self.published = true;
        Ok(())
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
            self.remove_published()?;
            self.published = false;
        }
        self.remove_staging()
    }

    pub(crate) fn finish(mut self) -> ProjectSceneDocument {
        self.published = false;
        self.document
            .take()
            .expect("prepared scene document must remain available until finish")
    }

    fn remove_staging(&mut self) -> Result<(), ProjectAuthorityError> {
        #[cfg(target_os = "linux")]
        return self
            ._path_guard
            .remove_owned_file(&self.staging_path, self.document().source_revision());
        #[cfg(not(target_os = "linux"))]
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

    fn remove_published(&self) -> Result<(), ProjectAuthorityError> {
        #[cfg(target_os = "linux")]
        return self._path_guard.remove_owned_file(
            self.document().source_path(),
            self.document().source_revision(),
        );
        #[cfg(not(target_os = "linux"))]
        fs::remove_file(self.document().source_path()).map_err(|source| {
            ProjectAuthorityError::io(
                "remove rejected scene asset",
                self.document().source_path(),
                source,
            )
        })
    }
}

impl Drop for PreparedSceneCreation {
    fn drop(&mut self) {
        if self.document.is_some() {
            if self.published {
                let _ = self.remove_published();
            }
            let _ = self.remove_staging();
        }
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
        #[cfg(target_os = "linux")]
        let loaded = Scene::load_scene_document_from_snapshot(
            project,
            request.scene_uri(),
            source_path.clone(),
            _path_guard.read_snapshot()?,
        )?;
        #[cfg(not(target_os = "linux"))]
        let loaded = Scene::load_scene_document(project, request.scene_uri())?;
        #[cfg(target_os = "linux")]
        _path_guard.verify()?;
        let (world, source_revision) = loaded.into_parts();
        Ok(ProjectSceneDocument {
            scene_uri: request.scene_uri,
            source_path,
            world,
            source_revision,
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
        #[cfg(target_os = "linux")]
        let source_path = project.primary_project_source_path_for_no_follow(request.scene_uri())?;
        #[cfg(not(target_os = "linux"))]
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
        #[cfg(target_os = "linux")]
        let mut path_guard = path_guard;
        let world = Scene::default();
        let staging_uri = scene_staging_uri(request.scene_uri());
        #[cfg(target_os = "linux")]
        let staging_path = project.primary_project_source_path_for_no_follow(&staging_uri)?;
        #[cfg(not(target_os = "linux"))]
        let staging_path = project.primary_project_source_path_for_uri(&staging_uri)?;
        #[cfg(target_os = "linux")]
        let source_revision = {
            let document = world
                .to_scene_asset(project)?
                .to_project_toml_string(|reference| project.persist_runtime_reference(reference))
                .map_err(zircon_runtime::scene::world::SceneProjectError::from)?;
            path_guard.stage_scene(&staging_path, document.as_bytes())?
        };
        #[cfg(not(target_os = "linux"))]
        let receipt =
            match world.save_scene_to_project(project, &staging_uri, &SceneSourceRevision::Missing)
            {
                Ok(receipt) => receipt,
                Err(error) => return Err(error.into()),
            };
        #[cfg(not(target_os = "linux"))]
        receipt.ensure_durable()?;
        // The final create-only hard link publishes this exact staged object.
        #[cfg(not(target_os = "linux"))]
        let source_revision = receipt.source_revision().clone();
        Ok(PreparedSceneCreation {
            document: Some(ProjectSceneDocument {
                scene_uri: request.scene_uri,
                source_path,
                world,
                source_revision,
            }),
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
    #[cfg(target_os = "linux")]
    let source_path = project.source_path_for_no_follow(scene_uri)?;
    #[cfg(not(target_os = "linux"))]
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
