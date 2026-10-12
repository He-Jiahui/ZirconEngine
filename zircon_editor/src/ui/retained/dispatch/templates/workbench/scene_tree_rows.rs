use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

pub(super) const SCENE_TREE_STATIC_CONTROLS: &[&str] = &[
    "WorkbenchSceneRootItem",
    "WorkbenchSceneEnvironmentItem",
    "WorkbenchSceneLevelItem",
    "WorkbenchScenePropsItem",
    "WorkbenchScenePlayerItem",
    "WorkbenchSceneAudioItem",
    "WorkbenchSceneSlot07Item",
    "WorkbenchSceneSlot08Item",
    "WorkbenchSceneSlot09Item",
    "WorkbenchSceneSlot10Item",
];
const SCENE_TREE_AUTHORED_ROW_COUNT: usize = SCENE_TREE_STATIC_CONTROLS.len();

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn reconcile_scene_tree_row_capacity(
        &mut self,
        entry_count: usize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        // Native hierarchy projection owns logical rows beyond the authored skeleton.
        let authored_row_count = entry_count.min(SCENE_TREE_AUTHORED_ROW_COUNT);
        debug_assert!(authored_row_count <= SCENE_TREE_STATIC_CONTROLS.len());
        Ok(())
    }

    pub(super) fn scene_tree_control_ids(
        &self,
    ) -> Result<Vec<String>, BuiltinHostWindowTemplateBridgeError> {
        Ok(SCENE_TREE_STATIC_CONTROLS
            .iter()
            .map(|control_id| (*control_id).to_string())
            .collect())
    }

    pub(super) fn is_scene_tree_control(
        &self,
        control_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        Ok(SCENE_TREE_STATIC_CONTROLS.contains(&control_id)
            || self.scene_hierarchy_projection.contains_control(control_id))
    }
}
