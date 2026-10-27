#[path = "lifecycle.rs"]
mod lifecycle;
#[path = "scene_reload.rs"]
mod scene_reload;
#[path = "scene_route.rs"]
mod scene_route;
#[path = "scene_save.rs"]
mod scene_save;

pub(crate) use lifecycle::SceneDocumentActivationReservation;

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
pub use scene_save::{OverwriteLease, SceneSaveError, SceneSaveOutcome};

#[cfg(test)]
#[path = "tests/scene_reload_tests.rs"]
mod scene_reload_tests;
#[cfg(test)]
#[path = "tests/scene_route_tests.rs"]
mod scene_route_tests;
#[cfg(all(test, any(windows, target_os = "linux")))]
#[path = "tests/scene_save_as.rs"]
mod scene_save_as_tests;
#[cfg(test)]
#[path = "tests/scene_save_tests.rs"]
mod scene_save_tests;
