use std::sync::Arc;

use zircon_runtime::scene::WorldInspectionHierarchyRow;

use super::super::{RetainedEditorHost, ViewContentKind};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn prepare_hierarchy_pointer_target(
        &mut self,
        width: f32,
        height: f32,
        focus_source_window: bool,
    ) -> Arc<[WorldInspectionHierarchyRow]> {
        self.use_committed_pointer_layout();
        let generation = self.ui.get_host_presentation_generation();
        let authored_metrics = crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_hierarchy(generation.structure()).map(|hierarchy| hierarchy.metrics);
        drop(generation);
        let row_metrics_changed = self
            .hierarchy_pointer_bridge
            .set_authored_row_metrics(authored_metrics);
        let target_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.hierarchy_pointer_size,
            ViewContentKind::Hierarchy,
        );
        let scene_entries = Arc::clone(&self.hierarchy_scene_entries);
        if self.hierarchy_pointer_size != target_size || row_metrics_changed {
            self.hierarchy_pointer_size = target_size;
            self.sync_hierarchy_pointer_layout(Arc::clone(&scene_entries));
        }
        if focus_source_window {
            self.focus_callback_source_window();
        }
        scene_entries
    }
}

#[cfg(test)]
#[path = "tests/target_performance_tests.rs"]
mod performance_tests;
