//! Save As publishes current authoring bytes through New's lease without reinstalling a world.
use std::convert::Infallible;

use zircon_runtime::scene::Scene;

use super::{SceneAssetCatalog, SceneCreateRequest, SceneDocumentRoute, SceneDocumentRouteError};
use crate::core::document::ActiveSceneDocumentIdentity;

impl SceneDocumentRoute<'_> {
    /// Preserves the active document/history and commits its new source only after publication
    /// and catalog synchronization succeed. Failed publication leaves the old journal bound.
    pub fn save_as<Catalog: SceneAssetCatalog>(
        &self,
        request: SceneCreateRequest,
        world: &Scene,
        catalog: &Catalog,
    ) -> Result<ActiveSceneDocumentIdentity, SceneDocumentRouteError<Infallible>> {
        self.lifecycle.with_scene_route(|| {
            self.lifecycle
                .validate_scene_picker_ticket_while_routed(
                    &self.ticket,
                    self.project.paths().root(),
                )
                .map_err(SceneDocumentRouteError::Lifecycle)?;
            let reservation = self
                .lifecycle
                .prepare_scene_save_as_while_routed(&self.ticket, &request.scene_uri().to_string())
                .map_err(SceneDocumentRouteError::Lifecycle)?;
            let identity = self
                .lifecycle
                .active_scene_identity_while_routed(self.project.paths().root())
                .expect("reserved active scene");
            let mut creation = self
                .authority
                .prepare_scene_publication(self.project, request, world.clone())
                .map_err(SceneDocumentRouteError::Project)?;
            let binding = self
                .journal
                .prepare_project_document_rebind(
                    identity.document(),
                    creation.document().source_path(),
                )
                .map_err(SceneDocumentRouteError::Journal)?;
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
            binding.commit();
            let document = creation.finish();
            Ok(self
                .lifecycle
                .commit_scene_save_as_while_routed(reservation, document.source_revision().clone()))
        })
    }
}
