mod lifecycle;
mod scene_route;

pub use lifecycle::{
    DocumentLifecycleAuthority, DocumentLifecycleRetentionSnapshot, ProjectSessionActivation,
    ProjectSessionId, SceneDocumentActivation, SceneDocumentLifecycleError, ScenePickerTicket,
};
pub use scene_route::{
    AuthoringSceneInstaller, SceneAssetCatalog, SceneDocumentRoute, SceneDocumentRouteActivation,
    SceneDocumentRouteError, SceneDocumentRouteResult,
};

#[cfg(test)]
mod scene_route_tests;
