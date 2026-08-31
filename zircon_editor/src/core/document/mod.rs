mod lifecycle;
mod scene_reload;
mod scene_route;

pub(super) use lifecycle::SceneDocumentActivationReservation;

pub use lifecycle::{
    ActiveSceneDocumentIdentity, DocumentLifecycleAuthority, DocumentLifecycleRetentionSnapshot,
    ProjectSessionActivation, ProjectSessionId, SceneDocumentActivation,
    SceneDocumentActivationBindingError, SceneDocumentLifecycleError, ScenePickerTicket,
};
pub use scene_reload::{
    ActiveSceneReloader, SceneDocumentReloadCoordinator, SceneDocumentReloadError,
    SceneDocumentReloadOutcome,
};
pub use scene_route::{
    AuthoringSceneInstaller, SceneAssetCatalog, SceneDocumentRoute, SceneDocumentRouteActivation,
    SceneDocumentRouteError, SceneDocumentRouteResult,
};

#[cfg(test)]
mod scene_reload_tests;
#[cfg(test)]
mod scene_route_tests;
