use std::fmt::{self, Display, Formatter};

use crate::core::editor_message::{DocumentId, DocumentMessage};
use crate::core::project::{
    ProjectAuthority, ProjectAuthorityError, ProjectSceneDocument, SceneCreateRequest,
    SceneOpenRequest,
};
use crate::core::recovery::{DocumentJournalCoordinator, DocumentJournalCoordinatorError};
use zircon_runtime::asset::project::ProjectManager;

use super::{
    DocumentLifecycleAuthority, SceneDocumentActivation, SceneDocumentActivationReservation,
    SceneDocumentLifecycleError, ScenePickerTicket,
};

/// Host port that atomically installs a scene only after its project identity has been resolved.
pub trait AuthoringSceneInstaller {
    type Error;

    /// Refuses a scene replacement before the route resolves or publishes a new source.
    ///
    /// Hosts use this admission point to preserve dirty editor state. It is deliberately separate
    /// from installation so a rejected transition cannot clear world/history/selection state.
    fn prepare_scene_transition(&mut self) -> Result<(), Self::Error>;

    /// Installs a resolved scene atomically, or returns without mutating the current world.
    fn install_scene(&mut self, document: &ProjectSceneDocument) -> Result<(), Self::Error>;
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
    journal: &'a DocumentJournalCoordinator,
    ticket: ScenePickerTicket,
    authority: ProjectAuthority,
}

impl<'a> SceneDocumentRoute<'a> {
    pub fn new(
        project: &'a ProjectManager,
        lifecycle: &'a DocumentLifecycleAuthority,
        journal: &'a DocumentJournalCoordinator,
        ticket: ScenePickerTicket,
    ) -> Self {
        Self {
            project,
            lifecycle,
            journal,
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

        installer
            .prepare_scene_transition()
            .map_err(SceneDocumentRouteError::Transition)?;

        let document = self
            .authority
            .open_scene(self.project, request)
            .map_err(SceneDocumentRouteError::Project)?;
        let reservation = self
            .lifecycle
            .prepare_scene_activation_while_routed(
                self.ticket.session(),
                self.project.paths().root(),
                &document.scene_uri().to_string(),
            )
            .map_err(SceneDocumentRouteError::Lifecycle)?;
        self.install_and_commit(document, reservation, installer)
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
        installer
            .prepare_scene_transition()
            .map_err(SceneDocumentRouteError::Transition)?;
        let mut creation = self
            .authority
            .prepare_scene_creation(self.project, request)
            .map_err(SceneDocumentRouteError::Project)?;
        let reservation = self
            .lifecycle
            .prepare_scene_activation_while_routed(
                self.ticket.session(),
                self.project.paths().root(),
                &creation.document().scene_uri().to_string(),
            )
            .map_err(SceneDocumentRouteError::Lifecycle)?;
        self.bind_document(creation.document(), &reservation)
            .map_err(SceneDocumentRouteError::Journal)?;
        if let Err(error) = creation.publish_and_discard_staging() {
            self.release_uncommitted_document_journal(&reservation);
            return Err(SceneDocumentRouteError::Project(error));
        }
        let scene_uri = creation.document().scene_uri().clone();
        if let Err(catalog_error) = catalog.import_scene(&scene_uri) {
            self.release_uncommitted_document_journal(&reservation);
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
        if let Err(install) = installer.install_scene(creation.document()) {
            self.release_uncommitted_document_journal(&reservation);
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
        self.commit_document(creation.finish(), reservation)
    }

    fn install_and_commit<Installer>(
        &self,
        document: ProjectSceneDocument,
        reservation: SceneDocumentActivationReservation,
        installer: &mut Installer,
    ) -> Result<SceneDocumentRouteResult, SceneDocumentRouteError<Installer::Error>>
    where
        Installer: AuthoringSceneInstaller,
    {
        self.bind_document(&document, &reservation)
            .map_err(SceneDocumentRouteError::Journal)?;
        if let Err(error) = installer.install_scene(&document) {
            self.release_uncommitted_document_journal(&reservation);
            return Err(SceneDocumentRouteError::Install(error));
        }
        Ok(self.commit_document(document, reservation))
    }

    fn bind_document(
        &self,
        document: &ProjectSceneDocument,
        reservation: &SceneDocumentActivationReservation,
    ) -> Result<(), DocumentJournalCoordinatorError> {
        self.journal
            .bind_project_document(reservation.document(), document.source_path())
    }

    fn commit_document(
        &self,
        document: ProjectSceneDocument,
        reservation: SceneDocumentActivationReservation,
    ) -> SceneDocumentRouteResult {
        let activation = self
            .lifecycle
            .commit_scene_activation_while_routed(reservation);
        if activation.already_active {
            return SceneDocumentRouteResult::AlreadyActive {
                document: activation.document,
            };
        }
        self.release_closed_document_journals(&activation.messages);
        SceneDocumentRouteResult::Activated(SceneDocumentRouteActivation {
            document,
            activation,
        })
    }

    fn release_closed_document_journals(&self, messages: &[DocumentMessage]) {
        for message in messages {
            if let DocumentMessage::Closed { doc } = message {
                self.journal.unbind_document(*doc);
            }
        }
    }

    fn release_uncommitted_document_journal(
        &self,
        reservation: &SceneDocumentActivationReservation,
    ) {
        self.journal.unbind_document(reservation.document());
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
    Journal(DocumentJournalCoordinatorError),
    Transition(InstallerError),
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
            Self::Journal(error) => {
                write!(formatter, "scene document journal binding failed: {error}")
            }
            Self::Transition(error) => {
                write!(
                    formatter,
                    "scene document transition was not admitted: {error}"
                )
            }
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
