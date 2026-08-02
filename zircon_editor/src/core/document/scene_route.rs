use std::fmt::{self, Display, Formatter};

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::scene::Scene;

use crate::core::editor_message::DocumentId;
use crate::core::project::{
    ProjectAuthority, ProjectAuthorityError, ProjectSceneDocument, SceneCreateRequest,
    SceneOpenRequest,
};

use super::{
    DocumentLifecycleAuthority, SceneDocumentActivation, SceneDocumentLifecycleError,
    ScenePickerTicket,
};

/// Host port that atomically installs a scene only after its project identity has been resolved.
pub trait AuthoringSceneInstaller {
    type Error;

    /// Installs a resolved scene atomically, or returns without mutating the current world.
    fn install_scene(&mut self, scene: &Scene) -> Result<(), Self::Error>;
}

/// Synchronizes the authoritative asset catalog around a newly published scene source.
///
/// The implementation must make a successful import visible before authoring-world installation.
/// When a later installation fails, `remove_scene` must reconcile the catalog with the rolled-back
/// source path before the route reports the failure.
pub trait SceneAssetCatalog {
    fn import_scene(
        &self,
        scene_uri: &zircon_runtime::asset::AssetUri,
    ) -> Result<(), ProjectAuthorityError>;

    fn remove_scene(
        &self,
        scene_uri: &zircon_runtime::asset::AssetUri,
    ) -> Result<(), ProjectAuthorityError>;
}

/// Core-only coordinator for scene authority, authoring-world installation, and document facts.
///
/// The route deliberately owns no picker or UI state. A picker supplies a typed project scene
/// request, while the host supplies the only adapter allowed to replace the authoring world.
pub struct SceneDocumentRoute<'a> {
    project: &'a ProjectManager,
    lifecycle: &'a DocumentLifecycleAuthority,
    ticket: ScenePickerTicket,
    authority: ProjectAuthority,
}

impl<'a> SceneDocumentRoute<'a> {
    pub fn new(
        project: &'a ProjectManager,
        lifecycle: &'a DocumentLifecycleAuthority,
        ticket: ScenePickerTicket,
    ) -> Self {
        Self {
            project,
            lifecycle,
            ticket,
            authority: ProjectAuthority,
        }
    }

    pub fn open<Installer>(
        &self,
        request: SceneOpenRequest,
        installer: &mut Installer,
    ) -> Result<SceneDocumentRouteResult, SceneDocumentRouteError<Installer::Error>>
    where
        Installer: AuthoringSceneInstaller,
    {
        self.lifecycle
            .with_scene_route(|| self.open_while_routed(request, installer))
    }

    fn open_while_routed<Installer>(
        &self,
        request: SceneOpenRequest,
        installer: &mut Installer,
    ) -> Result<SceneDocumentRouteResult, SceneDocumentRouteError<Installer::Error>>
    where
        Installer: AuthoringSceneInstaller,
    {
        self.lifecycle
            .validate_scene_picker_ticket_while_routed(&self.ticket, self.project.paths().root())
            .map_err(SceneDocumentRouteError::Lifecycle)?;
        if let Some(document) = self
            .lifecycle
            .active_scene_document_while_routed(
                self.ticket.session(),
                self.project.paths().root(),
                &request.scene_uri().to_string(),
            )
            .map_err(SceneDocumentRouteError::Lifecycle)?
        {
            return Ok(SceneDocumentRouteResult::AlreadyActive { document });
        }

        let document = self
            .authority
            .open_scene(self.project, request)
            .map_err(SceneDocumentRouteError::Project)?;
        self.install_and_activate(document, installer)
    }

    pub fn create<Installer, Catalog>(
        &self,
        request: SceneCreateRequest,
        installer: &mut Installer,
        catalog: &Catalog,
    ) -> Result<SceneDocumentRouteResult, SceneDocumentRouteError<Installer::Error>>
    where
        Installer: AuthoringSceneInstaller,
        Catalog: SceneAssetCatalog,
    {
        self.lifecycle
            .with_scene_route(|| self.create_while_routed(request, installer, catalog))
    }

    fn create_while_routed<Installer, Catalog>(
        &self,
        request: SceneCreateRequest,
        installer: &mut Installer,
        catalog: &Catalog,
    ) -> Result<SceneDocumentRouteResult, SceneDocumentRouteError<Installer::Error>>
    where
        Installer: AuthoringSceneInstaller,
        Catalog: SceneAssetCatalog,
    {
        self.lifecycle
            .validate_scene_picker_ticket_while_routed(&self.ticket, self.project.paths().root())
            .map_err(SceneDocumentRouteError::Lifecycle)?;
        let mut creation = self
            .authority
            .prepare_scene_creation(self.project, request)
            .map_err(SceneDocumentRouteError::Project)?;
        creation
            .publish_and_discard_staging()
            .map_err(SceneDocumentRouteError::Project)?;
        let scene_uri = creation.document().scene_uri().clone();
        if let Err(catalog_error) = catalog.import_scene(&scene_uri) {
            return match creation.rollback() {
                Ok(()) => match catalog.remove_scene(&scene_uri) {
                    Ok(()) => Err(SceneDocumentRouteError::Project(catalog_error)),
                    Err(cleanup) => Err(SceneDocumentRouteError::CatalogImportRollback {
                        import: catalog_error,
                        cleanup,
                    }),
                },
                Err(rollback) => Err(SceneDocumentRouteError::CatalogRollback {
                    catalog: catalog_error,
                    rollback,
                }),
            };
        }
        if let Err(install) = installer.install_scene(creation.document().world()) {
            return match creation.rollback() {
                Ok(()) => match catalog.remove_scene(&scene_uri) {
                    Ok(()) => Err(SceneDocumentRouteError::Install(install)),
                    Err(catalog) => {
                        Err(SceneDocumentRouteError::InstallCatalogRollback { install, catalog })
                    }
                },
                Err(rollback) => {
                    Err(SceneDocumentRouteError::InstallRollback { install, rollback })
                }
            };
        }
        self.activate_document(creation.finish())
    }

    fn install_and_activate<Installer>(
        &self,
        document: ProjectSceneDocument,
        installer: &mut Installer,
    ) -> Result<SceneDocumentRouteResult, SceneDocumentRouteError<Installer::Error>>
    where
        Installer: AuthoringSceneInstaller,
    {
        installer
            .install_scene(document.world())
            .map_err(SceneDocumentRouteError::Install)?;
        self.activate_document(document)
    }

    fn activate_document<InstallerError>(
        &self,
        document: ProjectSceneDocument,
    ) -> Result<SceneDocumentRouteResult, SceneDocumentRouteError<InstallerError>> {
        let activation = self
            .lifecycle
            .activate_scene_while_routed(
                self.ticket.session(),
                self.project.paths().root(),
                &document.scene_uri().to_string(),
            )
            .map_err(SceneDocumentRouteError::Lifecycle)?;
        if activation.already_active {
            return Ok(SceneDocumentRouteResult::AlreadyActive {
                document: activation.document,
            });
        }
        Ok(SceneDocumentRouteResult::Activated(
            SceneDocumentRouteActivation {
                document,
                activation,
            },
        ))
    }
}

#[derive(Clone, Debug)]
pub struct SceneDocumentRouteActivation {
    pub document: ProjectSceneDocument,
    pub activation: SceneDocumentActivation,
}

#[derive(Clone, Debug)]
pub enum SceneDocumentRouteResult {
    Activated(SceneDocumentRouteActivation),
    AlreadyActive { document: DocumentId },
}

#[derive(Debug)]
pub enum SceneDocumentRouteError<InstallerError> {
    Project(ProjectAuthorityError),
    Lifecycle(SceneDocumentLifecycleError),
    Install(InstallerError),
    InstallRollback {
        install: InstallerError,
        rollback: ProjectAuthorityError,
    },
    CatalogRollback {
        catalog: ProjectAuthorityError,
        rollback: ProjectAuthorityError,
    },
    CatalogImportRollback {
        import: ProjectAuthorityError,
        cleanup: ProjectAuthorityError,
    },
    InstallCatalogRollback {
        install: InstallerError,
        catalog: ProjectAuthorityError,
    },
}

impl<InstallerError: Display> Display for SceneDocumentRouteError<InstallerError> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Project(error) => write!(formatter, "scene project authority failed: {error}"),
            Self::Lifecycle(error) => write!(formatter, "scene document lifecycle failed: {error}"),
            Self::Install(error) => {
                write!(formatter, "scene authoring installation failed: {error}")
            }
            Self::InstallRollback { install, rollback } => write!(
                formatter,
                "scene authoring installation failed: {install}; scene asset rollback also failed: {rollback}"
            ),
            Self::CatalogRollback { catalog, rollback } => write!(
                formatter,
                "scene catalog synchronization failed: {catalog}; scene source rollback also failed: {rollback}"
            ),
            Self::CatalogImportRollback { import, cleanup } => write!(
                formatter,
                "scene catalog synchronization failed: {import}; catalog reconciliation after source rollback also failed: {cleanup}"
            ),
            Self::InstallCatalogRollback { install, catalog } => write!(
                formatter,
                "scene authoring installation failed: {install}; scene catalog rollback also failed: {catalog}"
            ),
        }
    }
}

impl<InstallerError> From<ProjectAuthorityError> for SceneDocumentRouteError<InstallerError> {
    fn from(error: ProjectAuthorityError) -> Self {
        Self::Project(error)
    }
}
