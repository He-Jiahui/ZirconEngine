use std::sync::Arc;

use crate::core::framework::navigation::{
    NAVIGATION_BAKE_SCENE_OPERATION, NAVIGATION_BAKE_SURFACE_OPERATION,
    NAVIGATION_CLEAR_SURFACE_OPERATION, NAVIGATION_RESTORE_BAKE_OPERATION,
};
use crate::operation::{RuntimeOperationService, RuntimeOperationServiceError};

use super::handler::{NavigationOperationHandler, NavigationOperationKind};

pub fn register_navigation_operation_handlers(
    service: &mut RuntimeOperationService,
) -> Result<(), RuntimeOperationServiceError> {
    for (operation_id, kind) in [
        (
            NAVIGATION_BAKE_SCENE_OPERATION,
            NavigationOperationKind::BakeScene,
        ),
        (
            NAVIGATION_BAKE_SURFACE_OPERATION,
            NavigationOperationKind::BakeSurface,
        ),
        (
            NAVIGATION_CLEAR_SURFACE_OPERATION,
            NavigationOperationKind::ClearSurface,
        ),
        (
            NAVIGATION_RESTORE_BAKE_OPERATION,
            NavigationOperationKind::RestoreSnapshot,
        ),
    ] {
        service.register_handler(
            operation_id,
            Arc::new(NavigationOperationHandler::new(kind)),
        )?;
    }
    Ok(())
}
