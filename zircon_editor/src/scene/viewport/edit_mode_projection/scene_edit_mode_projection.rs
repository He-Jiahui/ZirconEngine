use std::sync::Arc;

use zircon_runtime::scene::{EntityId, WorldInspectionHierarchyRow};

use super::{SceneInspectorField, SceneViewportStats, SceneViewportToolbarState};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneEditModeProjection {
    pub(crate) selected_entity: Option<EntityId>,
    pub(crate) hierarchy_rows: Arc<[WorldInspectionHierarchyRow]>,
    pub(crate) inspector_fields: Vec<SceneInspectorField>,
    pub(crate) toolbar: SceneViewportToolbarState,
    pub(crate) stats: SceneViewportStats,
}
